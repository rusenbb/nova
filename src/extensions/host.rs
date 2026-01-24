//! Extension host - manages all extension isolates.
//!
//! The ExtensionHost is responsible for:
//! - Scanning the extensions directory and loading manifests
//! - Indexing commands for search
//! - Loading/unloading isolates on demand
//! - LRU eviction when max isolates reached
//! - Cleanup of idle isolates

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::platform::Platform;

use super::error::{ExtensionError, ExtensionResult};
use super::ipc::NovaContext;
use super::isolate::{ExtensionIsolate, IsolateState};
use super::manifest::{CommandConfig, ExtensionManifest};
use super::storage::ExtensionStorage;
use super::{CommandId, ExtensionId};

/// Configuration for the extension host.
#[derive(Clone)]
pub struct ExtensionHostConfig {
    /// Directory containing extensions.
    pub extensions_dir: PathBuf,

    /// Maximum number of isolates to keep loaded.
    pub max_isolates: usize,

    /// How long to keep an isolate loaded after last use.
    pub idle_timeout: Duration,

    /// Maximum execution time for a command.
    pub execution_timeout: Duration,

    /// Platform trait for system operations (optional for testing).
    pub platform: Option<Arc<dyn Platform>>,
}

impl Default for ExtensionHostConfig {
    fn default() -> Self {
        Self {
            extensions_dir: default_extensions_dir(),
            max_isolates: 10,
            idle_timeout: Duration::from_secs(30),
            execution_timeout: Duration::from_secs(30),
            platform: None,
        }
    }
}

impl std::fmt::Debug for ExtensionHostConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionHostConfig")
            .field("extensions_dir", &self.extensions_dir)
            .field("max_isolates", &self.max_isolates)
            .field("idle_timeout", &self.idle_timeout)
            .field("execution_timeout", &self.execution_timeout)
            .field("platform", &self.platform.is_some())
            .finish()
    }
}

fn default_extensions_dir() -> PathBuf {
    dirs::data_dir()
        .map(|d| d.join("nova").join("extensions"))
        .unwrap_or_else(|| PathBuf::from("~/.nova/extensions"))
}

/// A command that matched a search query.
#[derive(Debug, Clone)]
pub struct ExtensionCommandMatch {
    pub extension_id: ExtensionId,
    pub command_id: CommandId,
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: Option<String>,
    pub keywords: Vec<String>,
    pub score: i64,
}

/// Indexed command for fast lookup.
#[derive(Debug, Clone)]
struct IndexedCommand {
    extension_id: ExtensionId,
    command: CommandConfig,
    extension_title: String,
    extension_icon: Option<String>,
}

/// The extension host manages all extension isolates.
pub struct ExtensionHost {
    /// Configuration.
    config: ExtensionHostConfig,

    /// Platform trait for system operations.
    platform: Option<Arc<dyn Platform>>,

    /// Loaded manifests by extension ID.
    manifests: HashMap<ExtensionId, ExtensionManifest>,

    /// Extension directories by ID.
    extension_dirs: HashMap<ExtensionId, PathBuf>,

    /// Loaded isolates by extension ID.
    isolates: HashMap<ExtensionId, ExtensionIsolate>,

    /// All commands indexed for search.
    command_index: Vec<IndexedCommand>,

    /// Order of isolate loading (for LRU eviction).
    load_order: Vec<ExtensionId>,

    /// Fuzzy matcher for search.
    matcher: SkimMatcherV2,

    /// User preferences per extension (loaded from config).
    extension_preferences: HashMap<ExtensionId, HashMap<String, serde_json::Value>>,
}

impl ExtensionHost {
    /// Create a new extension host and scan the extensions directory.
    pub fn new(config: ExtensionHostConfig) -> ExtensionResult<Self> {
        let platform = config.platform.clone();

        let mut host = Self {
            config,
            platform,
            manifests: HashMap::new(),
            extension_dirs: HashMap::new(),
            isolates: HashMap::new(),
            command_index: Vec::new(),
            load_order: Vec::new(),
            matcher: SkimMatcherV2::default(),
            extension_preferences: HashMap::new(),
        };

        host.scan_extensions()?;
        Ok(host)
    }

    /// Create a NovaContext for an extension.
    fn create_context(&self, ext_id: &ExtensionId) -> ExtensionResult<NovaContext> {
        let manifest = self
            .manifests
            .get(ext_id)
            .ok_or_else(|| ExtensionError::ExtensionNotFound(ext_id.clone()))?;

        let extension_dir = self
            .extension_dirs
            .get(ext_id)
            .ok_or_else(|| ExtensionError::ExtensionNotFound(ext_id.clone()))?;

        let platform = self.platform.clone().ok_or_else(|| {
            ExtensionError::ExecutionError("Platform not configured for extension host".to_string())
        })?;

        let storage_dir = extension_dir.join("storage");
        let storage = ExtensionStorage::new(ext_id, storage_dir);

        let preferences = self
            .extension_preferences
            .get(ext_id)
            .cloned()
            .unwrap_or_default();

        Ok(NovaContext::new(
            ext_id.clone(),
            platform,
            storage,
            manifest.permissions.clone(),
            preferences,
        ))
    }

    /// Scan the extensions directory and load all manifests.
    pub fn scan_extensions(&mut self) -> ExtensionResult<()> {
        self.manifests.clear();
        self.extension_dirs.clear();
        self.command_index.clear();

        let extensions_dir = &self.config.extensions_dir;

        if !extensions_dir.exists() {
            // No extensions directory - that's fine, just no extensions
            return Ok(());
        }

        let entries = std::fs::read_dir(extensions_dir)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            match ExtensionManifest::load(&path) {
                Ok(manifest) => {
                    if let Err(e) = manifest.validate() {
                        eprintln!(
                            "Warning: Invalid manifest in {}: {}",
                            path.display(),
                            e
                        );
                        continue;
                    }

                    let ext_id = manifest.extension.name.clone();

                    // Index commands
                    for cmd in &manifest.commands {
                        self.command_index.push(IndexedCommand {
                            extension_id: ext_id.clone(),
                            command: cmd.clone(),
                            extension_title: manifest.extension.title.clone(),
                            extension_icon: manifest.extension.icon.clone(),
                        });
                    }

                    self.extension_dirs.insert(ext_id.clone(), path);
                    self.manifests.insert(ext_id, manifest);
                }
                Err(ExtensionError::ManifestNotFound(_)) => {
                    // Not an extension directory, skip
                    continue;
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load extension from {}: {}", path.display(), e);
                    continue;
                }
            }
        }

        Ok(())
    }

    /// Get the number of loaded extensions.
    pub fn extension_count(&self) -> usize {
        self.manifests.len()
    }

    /// Get the number of indexed commands.
    pub fn command_count(&self) -> usize {
        self.command_index.len()
    }

    /// Search for commands matching a query.
    pub fn search_commands(&self, query: &str) -> Vec<ExtensionCommandMatch> {
        if query.is_empty() {
            // Return all commands when query is empty
            return self
                .command_index
                .iter()
                .map(|idx| ExtensionCommandMatch {
                    extension_id: idx.extension_id.clone(),
                    command_id: idx.command.name.clone(),
                    title: idx.command.title.clone(),
                    subtitle: Some(idx.extension_title.clone()),
                    icon: idx.extension_icon.clone(),
                    keywords: idx.command.keywords.clone(),
                    score: 0,
                })
                .collect();
        }

        let query_lower = query.to_lowercase();
        let mut matches: Vec<ExtensionCommandMatch> = Vec::new();

        for idx in &self.command_index {
            // Build searchable text
            let search_text = format!(
                "{} {} {} {}",
                idx.command.title,
                idx.command.description,
                idx.command.keywords.join(" "),
                idx.extension_title
            );

            if let Some(score) = self.matcher.fuzzy_match(&search_text, &query_lower) {
                matches.push(ExtensionCommandMatch {
                    extension_id: idx.extension_id.clone(),
                    command_id: idx.command.name.clone(),
                    title: idx.command.title.clone(),
                    subtitle: Some(idx.extension_title.clone()),
                    icon: idx.extension_icon.clone(),
                    keywords: idx.command.keywords.clone(),
                    score,
                });
            }
        }

        // Sort by score descending
        matches.sort_by(|a, b| b.score.cmp(&a.score));
        matches
    }

    /// Get or load an isolate for an extension.
    fn get_or_load_isolate(
        &mut self,
        ext_id: &ExtensionId,
    ) -> ExtensionResult<&mut ExtensionIsolate> {
        // Check if we need to evict before loading
        if !self.isolates.contains_key(ext_id) && self.isolates.len() >= self.config.max_isolates {
            self.evict_lru_isolate();
        }

        // Create context before loading (if needed)
        let needs_load = if let Some(isolate) = self.isolates.get(ext_id) {
            isolate.state == IsolateState::Unloaded
        } else {
            true
        };

        let context = if needs_load {
            Some(self.create_context(ext_id)?)
        } else {
            None
        };

        // Load if not present
        if !self.isolates.contains_key(ext_id) {
            let manifest = self
                .manifests
                .get(ext_id)
                .ok_or_else(|| ExtensionError::ExtensionNotFound(ext_id.clone()))?
                .clone();

            let extension_dir = self
                .extension_dirs
                .get(ext_id)
                .ok_or_else(|| ExtensionError::ExtensionNotFound(ext_id.clone()))?
                .clone();

            let isolate = ExtensionIsolate::new(ext_id.clone(), manifest, extension_dir);

            self.isolates.insert(ext_id.clone(), isolate);
            self.load_order.push(ext_id.clone());
        } else {
            // Move to end of LRU order
            self.load_order.retain(|id| id != ext_id);
            self.load_order.push(ext_id.clone());
        }

        // Get mutable reference and ensure loaded
        let isolate = self.isolates.get_mut(ext_id).unwrap();

        if isolate.state == IsolateState::Unloaded {
            if let Some(ctx) = context {
                isolate.load(ctx)?;
            }
        }

        Ok(isolate)
    }

    /// Evict the least recently used isolate.
    fn evict_lru_isolate(&mut self) {
        if let Some(ext_id) = self.load_order.first().cloned() {
            if let Some(mut isolate) = self.isolates.remove(&ext_id) {
                isolate.unload();
            }
            self.load_order.retain(|id| id != &ext_id);
        }
    }

    /// Execute a command in an extension.
    pub fn execute_command(
        &mut self,
        ext_id: &ExtensionId,
        command: &str,
        argument: Option<&str>,
    ) -> ExtensionResult<String> {
        // Verify command exists
        let manifest = self
            .manifests
            .get(ext_id)
            .ok_or_else(|| ExtensionError::ExtensionNotFound(ext_id.clone()))?;

        if !manifest.commands.iter().any(|c| c.name == command) {
            return Err(ExtensionError::CommandNotFound {
                extension: ext_id.clone(),
                command: command.to_string(),
            });
        }

        // Get or load isolate
        let isolate = self.get_or_load_isolate(ext_id)?;

        // Execute
        isolate.execute_command(command, argument)
    }

    /// Cleanup idle isolates.
    pub fn cleanup_idle(&mut self) {
        let timeout = self.config.idle_timeout;
        let to_unload: Vec<ExtensionId> = self
            .isolates
            .iter()
            .filter(|(_, iso)| iso.is_idle(timeout) && iso.state == IsolateState::Ready)
            .map(|(id, _)| id.clone())
            .collect();

        for ext_id in to_unload {
            if let Some(isolate) = self.isolates.get_mut(&ext_id) {
                isolate.unload();
            }
            self.load_order.retain(|id| id != &ext_id);
        }
    }

    /// Get the manifest for an extension.
    pub fn get_manifest(&self, ext_id: &ExtensionId) -> Option<&ExtensionManifest> {
        self.manifests.get(ext_id)
    }

    /// Get all loaded manifests.
    pub fn manifests(&self) -> &HashMap<ExtensionId, ExtensionManifest> {
        &self.manifests
    }

    /// Reload all extensions (rescan directory).
    pub fn reload(&mut self) -> ExtensionResult<()> {
        // Unload all isolates
        for isolate in self.isolates.values_mut() {
            isolate.unload();
        }
        self.isolates.clear();
        self.load_order.clear();

        // Rescan
        self.scan_extensions()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_extension(dir: &PathBuf, name: &str) {
        let ext_dir = dir.join(name);
        fs::create_dir_all(&ext_dir).unwrap();

        let manifest = format!(
            r#"
[extension]
name = "{name}"
title = "{name} Extension"
version = "1.0.0"

[[commands]]
name = "test-cmd"
title = "Test Command"
keywords = ["test"]
"#
        );

        fs::write(ext_dir.join("nova.toml"), manifest).unwrap();
    }

    #[test]
    fn test_scan_extensions() {
        let temp = tempdir().unwrap();
        let ext_dir = temp.path().to_path_buf();

        create_test_extension(&ext_dir, "ext1");
        create_test_extension(&ext_dir, "ext2");

        let config = ExtensionHostConfig {
            extensions_dir: ext_dir,
            ..Default::default()
        };

        let host = ExtensionHost::new(config).unwrap();

        assert_eq!(host.extension_count(), 2);
        assert_eq!(host.command_count(), 2);
    }

    #[test]
    fn test_search_commands() {
        let temp = tempdir().unwrap();
        let ext_dir = temp.path().to_path_buf();

        create_test_extension(&ext_dir, "github");
        create_test_extension(&ext_dir, "jira");

        let config = ExtensionHostConfig {
            extensions_dir: ext_dir,
            ..Default::default()
        };

        let host = ExtensionHost::new(config).unwrap();

        // Search for "git"
        let results = host.search_commands("git");
        assert!(!results.is_empty());
        assert!(results[0].extension_id == "github");

        // Empty query returns all
        let all = host.search_commands("");
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_empty_extensions_dir() {
        let temp = tempdir().unwrap();

        let config = ExtensionHostConfig {
            extensions_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        let host = ExtensionHost::new(config).unwrap();
        assert_eq!(host.extension_count(), 0);
    }

    #[test]
    fn test_nonexistent_extensions_dir() {
        let config = ExtensionHostConfig {
            extensions_dir: PathBuf::from("/nonexistent/path"),
            ..Default::default()
        };

        let host = ExtensionHost::new(config).unwrap();
        assert_eq!(host.extension_count(), 0);
    }
}
