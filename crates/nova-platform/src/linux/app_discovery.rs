use freedesktop_desktop_entry::DesktopEntry;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use nova_core::{NovaError, NovaResult, PlatformAppEntry};
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

use crate::traits::AppDiscovery;

pub struct LinuxAppDiscovery {
    entries: Vec<PlatformAppEntry>,
    matcher: SkimMatcherV2,
}

impl Default for LinuxAppDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl LinuxAppDiscovery {
    pub fn new() -> Self {
        let mut entries = Vec::new();
        let matcher = SkimMatcherV2::default();

        let mut dirs_to_scan: Vec<PathBuf> = vec![
            PathBuf::from("/usr/share/applications"),
            PathBuf::from("/usr/local/share/applications"),
        ];

        if let Some(data_home) = dirs::data_local_dir() {
            dirs_to_scan.push(data_home.join("applications"));
        }

        if let Some(home) = dirs::home_dir() {
            dirs_to_scan.push(home.join(".local/share/flatpak/exports/share/applications"));
        }

        dirs_to_scan.push(PathBuf::from("/var/lib/snapd/desktop/applications"));

        for dir in dirs_to_scan {
            if dir.exists() {
                Self::scan_directory(&dir, &mut entries);
            }
        }

        entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        println!("[Nova] Indexed {} applications", entries.len());

        Self { entries, matcher }
    }

    fn scan_directory(dir: &PathBuf, entries: &mut Vec<PlatformAppEntry>) {
        for entry in WalkDir::new(dir)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "desktop") {
                if let Some(app_entry) = Self::parse_desktop_file(path.to_path_buf()) {
                    if !entries.iter().any(|e| e.id == app_entry.id) {
                        entries.push(app_entry);
                    }
                }
            }
        }
    }

    fn parse_desktop_file(path: PathBuf) -> Option<PlatformAppEntry> {
        let content = std::fs::read_to_string(&path).ok()?;
        let entry = DesktopEntry::from_str(&path, &content, Some(&["en"])).ok()?;

        if entry.no_display() || entry.hidden() {
            return None;
        }

        let locales: &[&str] = &[];
        let name = entry.name(locales)?.to_string();
        let exec = entry.exec()?.to_string();
        let id = path.file_stem()?.to_string_lossy().to_string();
        let icon = entry.icon().map(|s| s.to_string());
        let description = entry.comment(locales).map(|s| s.to_string());

        let mut keywords: Vec<String> = entry
            .keywords(locales)
            .map(|kw| kw.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default();

        keywords.extend(name.split_whitespace().map(|s| s.to_lowercase()));

        Some(PlatformAppEntry {
            id,
            name,
            exec,
            icon,
            description,
            keywords,
        })
    }

    /// Search for apps matching the query using fuzzy matching
    pub fn search(&self, query: &str) -> Vec<&PlatformAppEntry> {
        if query.is_empty() {
            return self.entries.iter().take(8).collect();
        }

        let query_lower = query.to_lowercase();
        let mut scored: Vec<(&PlatformAppEntry, i64)> = self
            .entries
            .iter()
            .filter_map(|entry| {
                let name_score = self
                    .matcher
                    .fuzzy_match(&entry.name.to_lowercase(), &query_lower);

                let keyword_score = entry
                    .keywords
                    .iter()
                    .filter_map(|kw| self.matcher.fuzzy_match(&kw.to_lowercase(), &query_lower))
                    .max();

                let desc_score = entry
                    .description
                    .as_ref()
                    .and_then(|d| self.matcher.fuzzy_match(&d.to_lowercase(), &query_lower))
                    .map(|s| s / 2);

                let best_score = [name_score, keyword_score, desc_score]
                    .into_iter()
                    .flatten()
                    .max()?;

                let prefix_boost = if entry.name.to_lowercase().starts_with(&query_lower) {
                    100
                } else {
                    0
                };

                Some((entry, best_score + prefix_boost))
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().take(8).map(|(entry, _)| entry).collect()
    }
}

impl AppDiscovery for LinuxAppDiscovery {
    fn discover_apps(&self) -> Vec<PlatformAppEntry> {
        self.entries.clone()
    }

    fn launch_app(&self, app: &PlatformAppEntry) -> NovaResult<()> {
        let exec = app
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
            return Err(NovaError::Launch("Empty exec command".to_string()));
        }

        let program = parts[0];
        let args = &parts[1..];

        Command::new(program)
            .args(args)
            .spawn()
            .map_err(|e| NovaError::Launch(format!("Failed to launch {}: {}", app.name, e)))?;

        Ok(())
    }
}
