//! Detail component definitions.
//!
//! Detail displays markdown content with an optional metadata sidebar.
//! Useful for showing detailed information about a selected item.

use serde::{Deserialize, Serialize};

use super::action::ActionPanel;
use super::common::Icon;

/// Detail component - displays markdown content with metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DetailComponent {
    /// Markdown content to render
    #[serde(default)]
    pub markdown: Option<String>,

    /// Whether the detail is loading
    #[serde(default)]
    pub is_loading: bool,

    /// Actions available for this view
    #[serde(default)]
    pub actions: Option<ActionPanel>,

    /// Metadata sidebar
    #[serde(default)]
    pub metadata: Option<DetailMetadata>,
}

/// Metadata sidebar for Detail component.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DetailMetadata {
    /// Metadata items
    #[serde(default)]
    pub children: Vec<MetadataItem>,
}

/// A single metadata item (key-value pair).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataItem {
    /// Label for this metadata
    pub title: String,

    /// Text value
    #[serde(default)]
    pub text: Option<String>,

    /// Icon to display
    #[serde(default)]
    pub icon: Option<Icon>,

    /// Link to open
    #[serde(default)]
    pub link: Option<MetadataLink>,
}

/// A clickable link in metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataLink {
    /// Display text
    pub text: String,
    /// URL to open
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detail_component() {
        let detail = DetailComponent {
            markdown: Some("# Hello\n\nThis is **markdown**.".to_string()),
            is_loading: false,
            actions: None,
            metadata: Some(DetailMetadata {
                children: vec![
                    MetadataItem {
                        title: "Author".to_string(),
                        text: Some("John Doe".to_string()),
                        icon: None,
                        link: None,
                    },
                    MetadataItem {
                        title: "Repository".to_string(),
                        text: None,
                        icon: None,
                        link: Some(MetadataLink {
                            text: "View on GitHub".to_string(),
                            url: "https://github.com/user/repo".to_string(),
                        }),
                    },
                ],
            }),
        };

        let json = serde_json::to_string_pretty(&detail).unwrap();
        assert!(json.contains("\"markdown\""));
        assert!(json.contains("\"Author\""));
        assert!(json.contains("github.com"));
    }

    #[test]
    fn test_detail_deserialize() {
        let json = r##"{
            "markdown": "# Title",
            "isLoading": false,
            "metadata": {
                "children": [
                    {"title": "Status", "text": "Active"}
                ]
            }
        }"##;

        let detail: DetailComponent = serde_json::from_str(json).unwrap();
        assert_eq!(detail.markdown, Some("# Title".to_string()));
        assert!(detail.metadata.is_some());
        assert_eq!(detail.metadata.unwrap().children.len(), 1);
    }

    #[test]
    fn test_metadata_with_icon() {
        let json = r#"{
            "title": "Language",
            "text": "Rust",
            "icon": {"type": "system", "name": "gearshape"}
        }"#;

        let item: MetadataItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.title, "Language");
        assert!(item.icon.is_some());
    }
}
