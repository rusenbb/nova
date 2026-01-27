//! Frecency-based ranking for search results.
//!
//! Frecency combines frequency (how often) and recency (how recently) to rank
//! results. This implementation uses Firefox's exponential decay algorithm:
//!
//! ```text
//! score = frequency_score × decay_factor
//! decay_factor = e^(-λ × age_days)
//! λ = ln(2) / half_life_days
//! ```
//!
//! With a 14-day half-life, an item used 14 days ago has half the recency weight
//! of an item used today.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Half-life in days for the exponential decay function.
/// After 14 days, recency score is halved.
const HALF_LIFE_DAYS: f64 = 14.0;

/// Decay constant: λ = ln(2) / half_life
const LAMBDA: f64 = std::f64::consts::LN_2 / HALF_LIFE_DAYS;

/// Maximum age in days before an entry is pruned.
const MAX_AGE_DAYS: u64 = 90;

/// Debounce interval for saving (in number of updates).
const SAVE_DEBOUNCE_COUNT: u32 = 5;

/// Kind-specific weight multipliers.
///
/// Different result types have different usage patterns:
/// - Scripts are often reused (boosted)
/// - Files change frequently (penalized)
/// - Clipboard items are ephemeral (not tracked)
fn kind_weight(kind: ResultKind) -> f64 {
    match kind {
        ResultKind::App => 1.0,       // Standard weighting
        ResultKind::Script => 1.2,    // Boost scripts (reuse pattern)
        ResultKind::Alias => 0.9,     // Slightly less weight
        ResultKind::Quicklink => 0.7, // Decay faster
        ResultKind::Command => 0.8,   // System commands
        ResultKind::Extension => 1.0, // Similar to apps
        ResultKind::File => 0.5,      // Files change often
        ResultKind::Clipboard => 0.0, // Don't track
    }
}

/// Type of result for categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResultKind {
    App,
    Alias,
    Quicklink,
    Script,
    Extension,
    File,
    Command,
    Clipboard,
}

/// A single usage entry tracking frequency and recency.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageEntry {
    /// Unique identifier for the result.
    pub id: String,

    /// Type of result.
    pub kind: ResultKind,

    /// Number of times this result was used.
    pub count: u32,

    /// Unix timestamp of last use.
    pub last_used: u64,

    /// Unix timestamp of first use.
    pub first_used: u64,
}

impl UsageEntry {
    /// Create a new entry with current timestamp.
    fn new(id: String, kind: ResultKind) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            kind,
            count: 1,
            last_used: now,
            first_used: now,
        }
    }

    /// Record a new usage.
    fn record_usage(&mut self) {
        self.count += 1;
        self.last_used = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Get the age in days since last use.
    fn age_days(&self) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let age_secs = now.saturating_sub(self.last_used);
        age_secs as f64 / 86400.0
    }
}

/// Frecency data store with persistence.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrecencyData {
    /// Map of result ID to usage entry.
    entries: HashMap<String, UsageEntry>,

    /// Number of updates since last save.
    #[serde(skip)]
    updates_since_save: u32,

    /// Path to the data file.
    #[serde(skip)]
    data_path: Option<PathBuf>,
}

impl FrecencyData {
    /// Create a new empty frecency store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load frecency data from the default location.
    ///
    /// Returns empty data if file doesn't exist or is corrupted.
    pub fn load() -> Self {
        let data_path = Self::default_path();

        let mut data = if let Some(ref path) = data_path {
            if path.exists() {
                match fs::read_to_string(path) {
                    Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
                    Err(_) => Self::default(),
                }
            } else {
                Self::default()
            }
        } else {
            Self::default()
        };

        data.data_path = data_path;
        data.prune_old();
        data
    }

    /// Get the default path for frecency data.
    fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("nova").join("frecency.json"))
    }

    /// Calculate the frecency score for a result.
    ///
    /// Returns 0.0 if the result has never been used.
    /// Higher scores indicate more frequently/recently used items.
    /// Scores are weighted by result kind (scripts boosted, files penalized).
    pub fn calculate(&self, id: &str) -> f64 {
        let Some(entry) = self.entries.get(id) else {
            return 0.0;
        };

        // Clipboard items are not tracked
        let weight = kind_weight(entry.kind);
        if weight == 0.0 {
            return 0.0;
        }

        let age_days = entry.age_days();

        // Frequency component: logarithmic scaling
        // log(1) = 0, log(2) ≈ 0.69, log(10) ≈ 2.3
        let freq_score = (entry.count as f64 + 1.0).ln();

        // Recency component: exponential decay
        // At age 0: 1.0, at half_life: 0.5, at 2×half_life: 0.25
        let recency_score = (-LAMBDA * age_days).exp();

        // Combined score: 40% frequency, 60% recency
        // Multiply by 10 to get reasonable score magnitudes (0-100 range)
        // Apply kind-specific weight
        let base_score = 0.4 * freq_score * 10.0 + 0.6 * recency_score * 100.0;
        base_score * weight
    }

    /// Record that a result was used.
    ///
    /// Updates the usage count and last-used timestamp.
    /// Triggers a debounced save to disk.
    pub fn log_usage(&mut self, id: &str, kind: ResultKind) {
        self.entries
            .entry(id.to_string())
            .and_modify(|e| e.record_usage())
            .or_insert_with(|| UsageEntry::new(id.to_string(), kind));

        self.updates_since_save += 1;

        // Debounced save
        if self.updates_since_save >= SAVE_DEBOUNCE_COUNT {
            self.save();
        }
    }

    /// Remove entries not used in MAX_AGE_DAYS.
    pub fn prune_old(&mut self) {
        let cutoff_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            - (MAX_AGE_DAYS * 86400);

        let before_count = self.entries.len();
        self.entries.retain(|_, e| e.last_used > cutoff_secs);

        if self.entries.len() != before_count {
            self.save();
        }
    }

    /// Save data to disk.
    pub fn save(&mut self) {
        self.updates_since_save = 0;

        let Some(ref path) = self.data_path else {
            return;
        };

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    /// Force an immediate save (e.g., on shutdown).
    pub fn flush(&mut self) {
        if self.updates_since_save > 0 {
            self.save();
        }
    }

    /// Get the number of tracked entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if there are no tracked entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the top N items by frecency score.
    ///
    /// Returns a vector of (id, score) pairs, sorted by score descending.
    pub fn top_by_score(&self, limit: usize) -> Vec<(String, f64)> {
        let mut scored: Vec<_> = self
            .entries
            .keys()
            .map(|id| {
                let score = self.calculate(id);
                (id.clone(), score)
            })
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored.truncate(limit);
        scored
    }

    /// Clear all frecency data.
    ///
    /// This resets the learning history. Useful for testing or user preference.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.save();
    }

    /// Boost an item's score by a multiplier.
    ///
    /// Increases the usage count to effectively boost the frecency score.
    /// A multiplier of 2.0 doubles the frequency component.
    pub fn boost(&mut self, id: &str, multiplier: f64) {
        if let Some(entry) = self.entries.get_mut(id) {
            // Boost by increasing count (affects log-scaled frequency)
            // To double the log score, we need exp(2 * ln(count)) = count^2
            // For a simpler boost, we multiply count directly
            let new_count = ((entry.count as f64) * multiplier).round() as u32;
            entry.count = new_count.max(1);
            self.updates_since_save += 1;
        }
    }

    /// Penalize an item's score by a divisor.
    ///
    /// Decreases the usage count to effectively reduce the frecency score.
    /// A divisor of 2.0 halves the frequency component.
    pub fn penalize(&mut self, id: &str, divisor: f64) {
        if divisor <= 0.0 {
            return;
        }
        if let Some(entry) = self.entries.get_mut(id) {
            let new_count = ((entry.count as f64) / divisor).round() as u32;
            entry.count = new_count.max(1);
            self.updates_since_save += 1;
        }
    }

    /// Get a specific entry for inspection.
    pub fn get_entry(&self, id: &str) -> Option<&UsageEntry> {
        self.entries.get(id)
    }

    /// Get usage statistics for debugging.
    pub fn stats(&self) -> FrecencyStats {
        let mut total_count = 0u64;
        let mut max_count = 0u32;
        let mut by_kind: HashMap<ResultKind, usize> = HashMap::new();

        for entry in self.entries.values() {
            total_count += entry.count as u64;
            max_count = max_count.max(entry.count);
            *by_kind.entry(entry.kind).or_insert(0) += 1;
        }

        FrecencyStats {
            total_entries: self.entries.len(),
            total_usage_count: total_count,
            max_usage_count: max_count,
            entries_by_kind: by_kind,
        }
    }
}

/// Statistics about frecency data.
#[derive(Debug)]
pub struct FrecencyStats {
    pub total_entries: usize,
    pub total_usage_count: u64,
    pub max_usage_count: u32,
    pub entries_by_kind: HashMap<ResultKind, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_new_entry() {
        let entry = UsageEntry::new("test".into(), ResultKind::App);
        assert_eq!(entry.id, "test");
        assert_eq!(entry.count, 1);
        assert!(entry.last_used > 0);
        assert_eq!(entry.first_used, entry.last_used);
    }

    #[test]
    fn test_record_usage() {
        let mut entry = UsageEntry::new("test".into(), ResultKind::App);
        let first_used = entry.last_used;

        std::thread::sleep(Duration::from_millis(10));
        entry.record_usage();

        assert_eq!(entry.count, 2);
        assert!(entry.last_used >= first_used);
    }

    #[test]
    fn test_calculate_new_item() {
        let data = FrecencyData::new();

        // Unknown items have score 0
        assert_eq!(data.calculate("unknown"), 0.0);
    }

    #[test]
    fn test_calculate_used_item() {
        let mut data = FrecencyData::new();
        data.log_usage("test", ResultKind::App);

        let score = data.calculate("test");

        // Score should be positive for recently used item
        assert!(score > 0.0, "Score should be positive: {}", score);
        // Score should be in reasonable range (0-100ish)
        assert!(score < 100.0, "Score should be < 100: {}", score);
    }

    #[test]
    fn test_frequent_use_increases_score() {
        let mut data = FrecencyData::new();

        data.log_usage("item1", ResultKind::App);
        let score1 = data.calculate("item1");

        // Use it many more times
        for _ in 0..10 {
            data.log_usage("item1", ResultKind::App);
        }
        let score2 = data.calculate("item1");

        assert!(
            score2 > score1,
            "More usage should increase score: {} > {}",
            score2,
            score1
        );
    }

    #[test]
    fn test_stats() {
        let mut data = FrecencyData::new();
        data.log_usage("app1", ResultKind::App);
        data.log_usage("app2", ResultKind::App);
        data.log_usage("alias1", ResultKind::Alias);

        let stats = data.stats();
        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.entries_by_kind.get(&ResultKind::App), Some(&2));
        assert_eq!(stats.entries_by_kind.get(&ResultKind::Alias), Some(&1));
    }

    #[test]
    fn test_serialize_deserialize() {
        let mut data = FrecencyData::new();
        data.log_usage("test", ResultKind::Script);

        let json = serde_json::to_string(&data).unwrap();
        let restored: FrecencyData = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.entries.len(), 1);
        assert!(restored.entries.contains_key("test"));
    }

    #[test]
    fn test_kind_weights() {
        let mut data = FrecencyData::new();

        // Log same usage for different kinds
        data.log_usage("app1", ResultKind::App);
        data.log_usage("script1", ResultKind::Script);
        data.log_usage("file1", ResultKind::File);

        let app_score = data.calculate("app1");
        let script_score = data.calculate("script1");
        let file_score = data.calculate("file1");

        // Scripts should score higher than apps (1.2 vs 1.0)
        assert!(
            script_score > app_score,
            "Script score {} should be > app score {}",
            script_score,
            app_score
        );

        // Files should score lower than apps (0.5 vs 1.0)
        assert!(
            file_score < app_score,
            "File score {} should be < app score {}",
            file_score,
            app_score
        );
    }

    #[test]
    fn test_clipboard_not_tracked() {
        let mut data = FrecencyData::new();
        data.log_usage("clip1", ResultKind::Clipboard);

        // Clipboard items have zero weight
        let score = data.calculate("clip1");
        assert_eq!(score, 0.0, "Clipboard score should be 0");
    }

    #[test]
    fn test_top_by_score() {
        let mut data = FrecencyData::new();

        // Create items with different usage counts
        data.log_usage("low", ResultKind::App);
        for _ in 0..5 {
            data.log_usage("medium", ResultKind::App);
        }
        for _ in 0..20 {
            data.log_usage("high", ResultKind::App);
        }

        let top = data.top_by_score(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "high");
        assert_eq!(top[1].0, "medium");
    }

    #[test]
    fn test_clear() {
        let mut data = FrecencyData::new();
        data.log_usage("item1", ResultKind::App);
        data.log_usage("item2", ResultKind::App);
        assert_eq!(data.len(), 2);

        data.clear();
        assert!(data.is_empty());
    }

    #[test]
    fn test_boost() {
        let mut data = FrecencyData::new();
        data.log_usage("item", ResultKind::App);

        let score_before = data.calculate("item");
        data.boost("item", 3.0); // Triple the count
        let score_after = data.calculate("item");

        assert!(
            score_after > score_before,
            "Boosted score {} should be > original {}",
            score_after,
            score_before
        );
    }

    #[test]
    fn test_penalize() {
        let mut data = FrecencyData::new();
        for _ in 0..10 {
            data.log_usage("item", ResultKind::App);
        }

        let score_before = data.calculate("item");
        data.penalize("item", 2.0); // Halve the count
        let score_after = data.calculate("item");

        assert!(
            score_after < score_before,
            "Penalized score {} should be < original {}",
            score_after,
            score_before
        );
    }

    #[test]
    fn test_get_entry() {
        let mut data = FrecencyData::new();
        data.log_usage("test", ResultKind::Script);

        let entry = data.get_entry("test");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().kind, ResultKind::Script);
        assert_eq!(entry.unwrap().count, 1);

        assert!(data.get_entry("nonexistent").is_none());
    }
}
