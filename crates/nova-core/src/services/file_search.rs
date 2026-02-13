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

    let (base_path, search_term) = parse_query(query);

    let Some(base_path) = base_path else {
        return Vec::new();
    };

    if !base_path.exists() {
        return Vec::new();
    }

    let matcher = SkimMatcherV2::default();
    let mut results: Vec<(i64, FileEntry)> = Vec::new();

    let max_depth = if search_term.is_empty() { 1 } else { 4 };

    for entry in WalkDir::new(&base_path)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path() == base_path {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy();
        if file_name.starts_with('.') && !search_term.starts_with('.') {
            continue;
        }

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

    results.sort_by(|a, b| b.0.cmp(&a.0));
    results
        .into_iter()
        .take(max_results)
        .map(|(_, entry)| entry)
        .collect()
}

/// Parse query into base path and search term
fn parse_query(query: &str) -> (Option<PathBuf>, String) {
    if query.is_empty() {
        return (None, String::new());
    }

    let expanded = if let Some(rest) = query.strip_prefix('~') {
        if let Some(home) = dirs::home_dir() {
            if rest.is_empty() {
                home.to_string_lossy().to_string()
            } else if let Some(path_rest) = rest.strip_prefix('/') {
                format!("{}/{}", home.display(), path_rest)
            } else {
                return (Some(home), rest.to_string());
            }
        } else {
            return (None, String::new());
        }
    } else if query.starts_with('/') {
        query.to_string()
    } else {
        if let Some(home) = dirs::home_dir() {
            return (Some(home), query.to_string());
        }
        return (None, String::new());
    };

    let path = Path::new(&expanded);

    if path.is_dir() {
        return (Some(path.to_path_buf()), String::new());
    }

    if let (Some(parent), Some(file_name)) = (path.parent(), path.file_name()) {
        if parent.is_dir() {
            return (
                Some(parent.to_path_buf()),
                file_name.to_string_lossy().to_string(),
            );
        }
    }

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
        let (path, _term) = parse_query("~/Doc");
        assert!(path.is_some());
        assert!(path.unwrap().exists());
    }

    #[test]
    fn test_parse_query_absolute() {
        let (path, term) = parse_query("/tmp");
        assert_eq!(path, Some(PathBuf::from("/tmp")));
        assert!(term.is_empty());
    }
}
