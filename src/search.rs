//! Search provider trait and engine for unified search across sources
//!
//! This module provides a trait-based abstraction for search providers,
//! allowing new search sources to be added cleanly.

// Allow dead code - used by FFI layer but not by GTK binary.
#![allow(dead_code)]

/// A search provider that can contribute results for a query
pub trait SearchProvider {
    /// Unique name of this provider (e.g., "apps", "calculator", "clipboard")
    fn name(&self) -> &str;

    /// Check if this provider should handle the given query
    /// Returns true if the provider wants to contribute results
    fn should_search(&self, query: &str) -> bool;

    /// Priority for this provider (higher = results shown first)
    /// Default priorities:
    /// - Aliases/Quicklinks: 100 (exact keyword match)
    /// - Calculator: 90
    /// - File search: 80
    /// - Apps: 70
    /// - System commands: 60
    fn priority(&self) -> i32 {
        50
    }
}

/// Context passed to search providers
pub struct SearchContext<'a> {
    /// The full query string
    pub query: &'a str,
    /// Query converted to lowercase
    pub query_lower: String,
    /// First word of query (keyword)
    pub keyword: String,
    /// Text after the keyword (if any)
    pub remaining: Option<String>,
    /// Maximum results to return
    pub max_results: usize,
}

impl<'a> SearchContext<'a> {
    pub fn new(query: &'a str, max_results: usize) -> Self {
        let query_lower = query.to_lowercase();
        let parts: Vec<&str> = query.splitn(2, ' ').collect();
        let keyword = parts[0].to_lowercase();
        let remaining = parts.get(1).map(|s| s.to_string());

        Self {
            query,
            query_lower,
            keyword,
            remaining,
            max_results,
        }
    }

    /// Check if query starts with any of the given prefixes
    pub fn starts_with_any(&self, prefixes: &[&str]) -> bool {
        prefixes.iter().any(|p| self.query_lower.starts_with(p))
    }

    /// Check if query contains the given substring (case-insensitive)
    pub fn contains(&self, needle: &str) -> bool {
        self.query_lower.contains(&needle.to_lowercase())
    }
}

/// Result from a search provider with its priority
#[derive(Debug)]
pub struct PrioritizedResult<T> {
    pub result: T,
    pub priority: i32,
    pub provider: String,
}

impl<T> PrioritizedResult<T> {
    pub fn new(result: T, priority: i32, provider: impl Into<String>) -> Self {
        Self {
            result,
            priority,
            provider: provider.into(),
        }
    }
}
