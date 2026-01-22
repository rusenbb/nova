use freedesktop_desktop_entry::DesktopEntry;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
}

impl AppEntry {
    /// Launch this application
    pub fn launch(&self) -> Result<(), String> {
        // Parse exec command - remove field codes like %f, %u, %F, %U
        let exec = self
            .exec
            .replace("%f", "")
            .replace("%F", "")
            .replace("%u", "")
            .replace("%U", "")
            .replace("%i", "")
            .replace("%c", "")
            .replace("%k", "");

        let parts: Vec<&str> = exec.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty exec command".to_string());
        }

        let program = parts[0];
        let args = &parts[1..];

        Command::new(program)
            .args(args)
            .spawn()
            .map_err(|e| format!("Failed to launch {}: {}", self.name, e))?;

        Ok(())
    }
}

pub struct AppIndex {
    entries: Vec<AppEntry>,
    matcher: SkimMatcherV2,
}

impl AppIndex {
    /// Create a new app index by scanning XDG directories
    pub fn new() -> Self {
        let mut entries = Vec::new();
        let matcher = SkimMatcherV2::default();

        // Standard XDG application directories
        let mut dirs_to_scan: Vec<PathBuf> = vec![
            PathBuf::from("/usr/share/applications"),
            PathBuf::from("/usr/local/share/applications"),
        ];

        // User local applications
        if let Some(data_home) = dirs::data_local_dir() {
            dirs_to_scan.push(data_home.join("applications"));
        }

        // Flatpak applications
        if let Some(home) = dirs::home_dir() {
            dirs_to_scan.push(home.join(".local/share/flatpak/exports/share/applications"));
        }

        // Snap applications
        dirs_to_scan.push(PathBuf::from("/var/lib/snapd/desktop/applications"));

        for dir in dirs_to_scan {
            if dir.exists() {
                Self::scan_directory(&dir, &mut entries);
            }
        }

        // Sort by name for consistent ordering
        entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        println!("[Nova] Indexed {} applications", entries.len());

        Self { entries, matcher }
    }

    fn scan_directory(dir: &PathBuf, entries: &mut Vec<AppEntry>) {
        for entry in WalkDir::new(dir)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "desktop") {
                if let Some(app_entry) = Self::parse_desktop_file(path.to_path_buf()) {
                    // Skip duplicates by ID
                    if !entries.iter().any(|e| e.id == app_entry.id) {
                        entries.push(app_entry);
                    }
                }
            }
        }
    }

    fn parse_desktop_file(path: PathBuf) -> Option<AppEntry> {
        let content = std::fs::read_to_string(&path).ok()?;
        let entry = DesktopEntry::from_str(&path, &content, Some(&["en"])).ok()?;

        // Skip entries that shouldn't be shown
        if entry.no_display() || entry.hidden() {
            return None;
        }

        // Use empty locale list to get default (untranslated) values
        let locales: &[&str] = &[];

        // Skip entries without a name or exec
        let name = entry.name(locales)?.to_string();
        let exec = entry.exec()?.to_string();

        // Use filename as ID
        let id = path.file_stem()?.to_string_lossy().to_string();

        let icon = entry.icon().map(|s| s.to_string());
        let description = entry.comment(locales).map(|s| s.to_string());

        // Collect keywords
        let mut keywords: Vec<String> = entry
            .keywords(locales)
            .map(|kw| kw.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default();

        // Add name words as keywords for better matching
        keywords.extend(name.split_whitespace().map(|s| s.to_lowercase()));

        Some(AppEntry {
            id,
            name,
            exec,
            icon,
            description,
            keywords,
        })
    }

    /// Search for apps matching the query
    pub fn search(&self, query: &str) -> Vec<&AppEntry> {
        if query.is_empty() {
            // Return first 8 apps when no query
            return self.entries.iter().take(8).collect();
        }

        let query_lower = query.to_lowercase();
        let mut scored: Vec<(&AppEntry, i64)> = self
            .entries
            .iter()
            .filter_map(|entry| {
                // Match against name
                let name_score = self
                    .matcher
                    .fuzzy_match(&entry.name.to_lowercase(), &query_lower);

                // Match against keywords
                let keyword_score = entry
                    .keywords
                    .iter()
                    .filter_map(|kw| self.matcher.fuzzy_match(&kw.to_lowercase(), &query_lower))
                    .max();

                // Match against description
                let desc_score = entry
                    .description
                    .as_ref()
                    .and_then(|d| self.matcher.fuzzy_match(&d.to_lowercase(), &query_lower))
                    .map(|s| s / 2); // Weight description matches lower

                // Get best score
                let best_score = [name_score, keyword_score, desc_score]
                    .into_iter()
                    .flatten()
                    .max()?;

                // Boost exact prefix matches
                let prefix_boost = if entry.name.to_lowercase().starts_with(&query_lower) {
                    100
                } else {
                    0
                };

                Some((entry, best_score + prefix_boost))
            })
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.1.cmp(&a.1));

        // Return top 8 results
        scored.into_iter().take(8).map(|(entry, _)| entry).collect()
    }

    /// Get total number of indexed apps
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the index is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for AppIndex {
    fn default() -> Self {
        Self::new()
    }
}
