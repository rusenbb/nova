//! Common types used across components.
//!
//! This module defines shared types like Icon, Accessory, and Shortcut
//! that are used by multiple component types.

use serde::{Deserialize, Serialize};

/// Icon reference - can be a system icon, URL, asset, emoji, or text.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Icon {
    /// System icon (SF Symbols on macOS, freedesktop on Linux)
    System { name: String },
    /// Remote image URL
    Url { url: String },
    /// Asset from extension bundle
    Asset { name: String },
    /// Emoji character
    Emoji { emoji: String },
    /// Text badge (1-2 characters)
    Text {
        text: String,
        #[serde(default)]
        color: Option<String>,
    },
}

/// Accessory displayed on the right side of list items.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Accessory {
    /// Simple text
    Text { text: String },
    /// Icon with optional text
    Icon {
        icon: Icon,
        #[serde(default)]
        text: Option<String>,
    },
    /// Colored tag/badge
    Tag {
        value: String,
        #[serde(default)]
        color: Option<String>,
    },
    /// Date display
    Date {
        /// ISO 8601 date string
        date: String,
        #[serde(default)]
        format: Option<DateFormat>,
    },
}

/// Date display format.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DateFormat {
    /// Relative time (e.g., "2 hours ago")
    #[default]
    Relative,
    /// Absolute date (e.g., "Jan 15, 2024")
    Absolute,
    /// Time only (e.g., "3:45 PM")
    Time,
}

/// Keyboard shortcut definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shortcut {
    /// Modifier keys
    pub modifiers: Vec<KeyModifier>,
    /// The key to press (e.g., "o", "enter", "backspace")
    pub key: String,
}

/// Keyboard modifier keys.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum KeyModifier {
    /// Command key (macOS) / Super key (Linux)
    Cmd,
    /// Control key
    Ctrl,
    /// Alt/Option key
    Alt,
    /// Shift key
    Shift,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_system_serialize() {
        let icon = Icon::System {
            name: "star.fill".to_string(),
        };
        let json = serde_json::to_string(&icon).unwrap();
        assert!(json.contains("\"type\":\"system\""));
        assert!(json.contains("\"name\":\"star.fill\""));
    }

    #[test]
    fn test_icon_deserialize() {
        let json = r#"{"type": "emoji", "emoji": "🚀"}"#;
        let icon: Icon = serde_json::from_str(json).unwrap();
        match icon {
            Icon::Emoji { emoji } => assert_eq!(emoji, "🚀"),
            _ => panic!("Expected emoji icon"),
        }
    }

    #[test]
    fn test_accessory_tag() {
        let json = r##"{"type": "tag", "value": "JavaScript", "color": "#f0db4f"}"##;
        let acc: Accessory = serde_json::from_str(json).unwrap();
        match acc {
            Accessory::Tag { value, color } => {
                assert_eq!(value, "JavaScript");
                assert_eq!(color, Some("#f0db4f".to_string()));
            }
            _ => panic!("Expected tag accessory"),
        }
    }

    #[test]
    fn test_shortcut() {
        let json = r#"{"modifiers": ["cmd", "shift"], "key": "o"}"#;
        let shortcut: Shortcut = serde_json::from_str(json).unwrap();
        assert_eq!(shortcut.modifiers.len(), 2);
        assert_eq!(shortcut.key, "o");
    }
}
