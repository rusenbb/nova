//! Clipboard history module for tracking recent clipboard items

use std::collections::VecDeque;
use std::process::Command;
use std::time::Instant;

/// Entry in clipboard history
#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    pub content: String,
    pub timestamp: Instant,
}

impl ClipboardEntry {
    /// Get a preview of the content (first line, limited chars)
    pub fn preview(&self, max_chars: usize) -> String {
        let first_line = self.content.lines().next().unwrap_or(&self.content);
        if first_line.len() > max_chars {
            format!("{}...", &first_line[..max_chars])
        } else {
            first_line.to_string()
        }
    }

    /// Get relative time description
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

/// Manages clipboard history
pub struct ClipboardHistory {
    items: VecDeque<ClipboardEntry>,
    max_items: usize,
    last_content: String,
}

impl ClipboardHistory {
    /// Create a new clipboard history with max items limit
    pub fn new(max_items: usize) -> Self {
        Self {
            items: VecDeque::with_capacity(max_items),
            max_items,
            last_content: String::new(),
        }
    }

    /// Get current clipboard content
    fn get_clipboard_content() -> Option<String> {
        let output = Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
            .ok()?;

        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout).to_string();
            // Skip empty content
            if content.trim().is_empty() {
                return None;
            }
            Some(content)
        } else {
            None
        }
    }

    /// Poll clipboard and add new content to history
    /// Returns true if new content was added
    pub fn poll(&mut self) -> bool {
        if let Some(content) = Self::get_clipboard_content() {
            // Only add if content changed
            if content != self.last_content {
                self.last_content = content.clone();
                self.add(content);
                return true;
            }
        }
        false
    }

    /// Add content to history
    pub fn add(&mut self, content: String) {
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

    /// Search items by content
    pub fn search(&self, query: &str) -> Vec<&ClipboardEntry> {
        let query_lower = query.to_lowercase();
        self.items
            .iter()
            .filter(|item| item.content.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Get all items (most recent first)
    pub fn all(&self) -> Vec<&ClipboardEntry> {
        self.items.iter().collect()
    }

    /// Get item by index
    pub fn get(&self, index: usize) -> Option<&ClipboardEntry> {
        self.items.get(index)
    }

    /// Number of items in history
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
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
}
