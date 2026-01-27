//! Clipboard history module for tracking recent clipboard items.
//!
//! This module is platform-agnostic - it stores clipboard history but does not
//! directly access the clipboard. The caller is responsible for providing
//! clipboard content via the Platform trait.

use std::collections::VecDeque;
use std::time::Instant;

/// Entry in clipboard history.
#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    pub content: String,
    pub timestamp: Instant,
}

impl ClipboardEntry {
    /// Get a preview of the content (first line, limited chars).
    pub fn preview(&self, max_chars: usize) -> String {
        let first_line = self.content.lines().next().unwrap_or(&self.content);
        if first_line.len() > max_chars {
            format!("{}...", &first_line[..max_chars])
        } else {
            first_line.to_string()
        }
    }

    /// Get relative time description.
    pub fn time_ago(&self) -> String {
        let elapsed = self.timestamp.elapsed();
        let secs = elapsed.as_secs();

        if secs < 60 {
            "just now".to_string()
        } else if secs < 3600 {
            format!("{}m ago", secs / 60)
        } else if secs < 86400 {
            format!("{}h ago", secs / 3600)
        } else {
            format!("{}d ago", secs / 86400)
        }
    }
}

/// Manages clipboard history.
///
/// This is a platform-agnostic clipboard history manager. To use it:
/// 1. Periodically call `platform.clipboard_read()` to get clipboard content
/// 2. Pass the content to `poll_with_content()` or `add()` to track history
///
/// Example:
/// ```ignore
/// // In your app's update loop or timer:
/// if let Some(content) = platform.clipboard_read() {
///     clipboard_history.poll_with_content(&content);
/// }
/// ```
pub struct ClipboardHistory {
    items: VecDeque<ClipboardEntry>,
    max_items: usize,
    last_content: String,
}

impl ClipboardHistory {
    /// Create a new clipboard history with max items limit.
    pub fn new(max_items: usize) -> Self {
        Self {
            items: VecDeque::with_capacity(max_items),
            max_items,
            last_content: String::new(),
        }
    }

    /// Poll with new clipboard content.
    ///
    /// Call this periodically with the result of `platform.clipboard_read()`.
    /// Returns `true` if new content was added to history.
    pub fn poll_with_content(&mut self, content: &str) -> bool {
        // Skip empty content
        if content.trim().is_empty() {
            return false;
        }

        // Only add if content changed
        if content != self.last_content {
            self.last_content = content.to_string();
            self.add(content.to_string());
            return true;
        }

        false
    }

    /// Add content to history.
    pub fn add(&mut self, content: String) {
        // Skip empty content
        if content.trim().is_empty() {
            return;
        }

        // Remove if already exists (move to front)
        self.items.retain(|item| item.content != content);

        // Add to front
        self.items.push_front(ClipboardEntry {
            content,
            timestamp: Instant::now(),
        });

        // Limit size
        while self.items.len() > self.max_items {
            self.items.pop_back();
        }
    }

    /// Search items by content.
    pub fn search(&self, query: &str) -> Vec<&ClipboardEntry> {
        let query_lower = query.to_lowercase();
        self.items
            .iter()
            .filter(|item| item.content.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Get all items (most recent first).
    pub fn all(&self) -> Vec<&ClipboardEntry> {
        self.items.iter().collect()
    }

    /// Get item by index.
    #[allow(dead_code)]
    pub fn get(&self, index: usize) -> Option<&ClipboardEntry> {
        self.items.get(index)
    }

    /// Number of items in history.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if history is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Clear all history.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.items.clear();
        self.last_content.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_history() {
        let mut history = ClipboardHistory::new(5);

        history.add("first".to_string());
        history.add("second".to_string());
        history.add("third".to_string());

        assert_eq!(history.len(), 3);
        assert_eq!(history.get(0).unwrap().content, "third"); // Most recent first
        assert_eq!(history.get(1).unwrap().content, "second");
        assert_eq!(history.get(2).unwrap().content, "first");
    }

    #[test]
    fn test_deduplication() {
        let mut history = ClipboardHistory::new(5);

        history.add("first".to_string());
        history.add("second".to_string());
        history.add("first".to_string()); // Duplicate

        assert_eq!(history.len(), 2);
        assert_eq!(history.get(0).unwrap().content, "first"); // Moved to front
    }

    #[test]
    fn test_search() {
        let mut history = ClipboardHistory::new(10);

        history.add("hello world".to_string());
        history.add("goodbye world".to_string());
        history.add("hello there".to_string());

        let results = history.search("hello");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_preview() {
        let entry = ClipboardEntry {
            content: "This is a very long line of text that should be truncated".to_string(),
            timestamp: Instant::now(),
        };

        let preview = entry.preview(20);
        assert!(preview.len() <= 23); // 20 + "..."
    }

    #[test]
    fn test_poll_with_content() {
        let mut history = ClipboardHistory::new(5);

        // First poll adds content
        assert!(history.poll_with_content("first"));
        assert_eq!(history.len(), 1);

        // Same content doesn't add
        assert!(!history.poll_with_content("first"));
        assert_eq!(history.len(), 1);

        // New content adds
        assert!(history.poll_with_content("second"));
        assert_eq!(history.len(), 2);

        // Empty content doesn't add
        assert!(!history.poll_with_content(""));
        assert!(!history.poll_with_content("   "));
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_max_items() {
        let mut history = ClipboardHistory::new(3);

        history.add("one".to_string());
        history.add("two".to_string());
        history.add("three".to_string());
        history.add("four".to_string()); // Should push out "one"

        assert_eq!(history.len(), 3);
        assert_eq!(history.get(0).unwrap().content, "four");
        assert_eq!(history.get(2).unwrap().content, "two");
    }
}
