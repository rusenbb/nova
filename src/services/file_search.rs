//! File search module for finding files and folders

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A file search result
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

impl FileEntry {
    /// Get display name (filename only)
    pub fn display_name(&self) -> &str {
        &self.name
    }

    /// Get full path as string
    pub fn path_string(&self) -> String {
        self.path.display().to_string()
    }

    /// Get path with ~ for home directory
    pub fn display_path(&self) -> String {
        if let Some(home) = dirs::home_dir() {
            if let Ok(suffix) = self.path.strip_prefix(&home) {
                return format!("~/{}", suffix.display());
            }
        }
        self.path_string()
    }
}

/// Search for files matching a query
pub fn search_files(query: &str, max_results: usize) -> Vec<FileEntry> {
    let query = query.trim();

    // Determine base path and search term
    let (base_path, search_term) = parse_query(query);

    let Some(base_path) = base_path else {
        return Vec::new();
    };

    if !base_path.exists() {
        return Vec::new();
    }

    let matcher = SkimMatcherV2::default();
    let mut results: Vec<(i64, FileEntry)> = Vec::new();

    // Walk directory with depth limit
    let max_depth = if search_term.is_empty() { 1 } else { 4 };

    for entry in WalkDir::new(&base_path)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        // Skip the base directory itself
        if entry.path() == base_path {
            continue;
        }

        // Skip hidden files unless query starts with .
        let file_name = entry.file_name().to_string_lossy();
        if file_name.starts_with('.') && !search_term.starts_with('.') {
            continue;
        }

        // If no search term, just list directory contents
        if search_term.is_empty() {
            results.push((
                0,
                FileEntry {
                    name: file_name.to_string(),
                    path: entry.path().to_path_buf(),
                    is_dir: entry.file_type().is_dir(),
                },
            ));
            if results.len() >= max_results {
                break;
            }
            continue;
        }

        // Fuzzy match filename
        if let Some(score) = matcher.fuzzy_match(&file_name, &search_term) {
            results.push((
                score,
                FileEntry {
                    name: file_name.to_string(),
                    path: entry.path().to_path_buf(),
                    is_dir: entry.file_type().is_dir(),
                },
            ));
        }
    }

    // Sort by score (descending) and take top results
    results.sort_by(|a, b| b.0.cmp(&a.0));
    results
        .into_iter()
        .take(max_results)
        .map(|(_, entry)| entry)
        .collect()
}

/// Parse query into base path and search term
/// Examples:
///   "~" -> (home_dir, "")
///   "~/Doc" -> (home_dir, "Doc")
///   "~/Documents/re" -> (home_dir/Documents, "re")
///   "/etc/pas" -> (/etc, "pas")
fn parse_query(query: &str) -> (Option<PathBuf>, String) {
    if query.is_empty() {
        return (None, String::new());
    }

    // Expand ~ to home directory
    let expanded = if query.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            if query == "~" {
                home.to_string_lossy().to_string()
            } else if query.starts_with("~/") {
                format!("{}{}", home.display(), &query[1..])
            } else {
                // ~something without / - treat ~ as home, rest as search
                return (Some(home), query[1..].to_string());
            }
        } else {
            return (None, String::new());
        }
    } else if query.starts_with('/') {
        query.to_string()
    } else {
        // No path prefix - search from home
        if let Some(home) = dirs::home_dir() {
            return (Some(home), query.to_string());
        }
        return (None, String::new());
    };

    let path = Path::new(&expanded);

    // If path exists and is a directory, list its contents
    if path.is_dir() {
        return (Some(path.to_path_buf()), String::new());
    }

    // Otherwise, use parent as base and filename as search term
    if let (Some(parent), Some(file_name)) = (path.parent(), path.file_name()) {
        if parent.is_dir() {
            return (
                Some(parent.to_path_buf()),
                file_name.to_string_lossy().to_string(),
            );
        }
    }

    // Fallback: try to find the longest existing parent path
    let mut current = path.to_path_buf();
    let mut search_parts = Vec::new();

    while !current.exists() {
        if let Some(file_name) = current.file_name() {
            search_parts.push(file_name.to_string_lossy().to_string());
        }
        if !current.pop() {
            break;
        }
    }

    if current.exists() && current.is_dir() {
        search_parts.reverse();
        let search_term = search_parts.join("/");
        return (Some(current), search_term);
    }

    (None, String::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_home() {
        let (path, term) = parse_query("~");
        assert!(path.is_some());
        assert!(term.is_empty());
    }

    #[test]
    fn test_parse_query_home_subdir() {
        let (path, term) = parse_query("~/Doc");
        assert!(path.is_some());
        // Either Documents exists and term is empty, or home exists and term is "Doc"
        assert!(path.unwrap().exists());
    }

    #[test]
    fn test_parse_query_absolute() {
        let (path, term) = parse_query("/tmp");
        assert_eq!(path, Some(PathBuf::from("/tmp")));
        assert!(term.is_empty());
    }
}
