//! List component definitions.
//!
//! List is the primary component for displaying searchable, scrollable
//! lists of items. Items can be grouped into sections.

use serde::{Deserialize, Serialize};

use super::action::ActionPanel;
use super::common::{Accessory, Icon};

/// List component - displays a searchable list of items.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListComponent {
    /// Whether the list is loading data
    #[serde(default)]
    pub is_loading: bool,

    /// Placeholder text for the search bar
    #[serde(default)]
    pub search_bar_placeholder: Option<String>,

    /// Filtering behavior
    #[serde(default)]
    pub filtering: ListFiltering,

    /// Callback ID for search text changes
    #[serde(default)]
    pub on_search_change: Option<String>,

    /// Callback ID for selection changes
    #[serde(default)]
    pub on_selection_change: Option<String>,

    /// Child items and sections
    #[serde(default)]
    pub children: Vec<ListChild>,
}

/// Filtering behavior for the list.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ListFiltering {
    /// Default: Nova handles filtering based on title/keywords
    #[default]
    Default,
    /// No filtering - extension handles it
    None,
    /// Custom filtering via onSearchChange callback
    Custom,
}

/// A child element of a List (either an Item or a Section).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ListChild {
    /// A single list item
    #[serde(rename = "List.Item")]
    Item(ListItem),
    /// A section containing items
    #[serde(rename = "List.Section")]
    Section(ListSection),
}

/// A single item in a list.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListItem {
    /// Unique identifier (required)
    pub id: String,

    /// Primary text (required)
    pub title: String,

    /// Secondary text
    #[serde(default)]
    pub subtitle: Option<String>,

    /// Icon displayed on the left
    #[serde(default)]
    pub icon: Option<Icon>,

    /// Accessories displayed on the right
    #[serde(default)]
    pub accessories: Vec<Accessory>,

    /// Additional search keywords
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Actions available for this item
    #[serde(default)]
    pub actions: Option<ActionPanel>,
}

/// A section that groups list items.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListSection {
    /// Section title
    #[serde(default)]
    pub title: Option<String>,

    /// Section subtitle
    #[serde(default)]
    pub subtitle: Option<String>,

    /// Items in this section
    #[serde(default)]
    pub children: Vec<ListItem>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_component_serialize() {
        let list = ListComponent {
            is_loading: false,
            search_bar_placeholder: Some("Search repositories...".to_string()),
            filtering: ListFiltering::Default,
            on_search_change: None,
            on_selection_change: None,
            children: vec![ListChild::Item(ListItem {
                id: "repo-1".to_string(),
                title: "my-repo".to_string(),
                subtitle: Some("A cool project".to_string()),
                icon: Some(Icon::System {
                    name: "folder.fill".to_string(),
                }),
                accessories: vec![],
                keywords: vec!["rust".to_string()],
                actions: None,
            })],
        };

        let json = serde_json::to_string_pretty(&list).unwrap();
        assert!(json.contains("\"searchBarPlaceholder\""));
        assert!(json.contains("\"my-repo\""));
    }

    #[test]
    fn test_list_item_deserialize() {
        let json = r#"{
            "id": "item-1",
            "title": "Test Item",
            "subtitle": "Description",
            "keywords": ["test", "item"]
        }"#;

        let item: ListItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.id, "item-1");
        assert_eq!(item.title, "Test Item");
        assert_eq!(item.keywords.len(), 2);
    }

    #[test]
    fn test_list_child_item() {
        let json = r#"{
            "type": "List.Item",
            "id": "test",
            "title": "Test"
        }"#;

        let child: ListChild = serde_json::from_str(json).unwrap();
        match child {
            ListChild::Item(item) => assert_eq!(item.id, "test"),
            _ => panic!("Expected Item"),
        }
    }

    #[test]
    fn test_list_child_section() {
        let json = r#"{
            "type": "List.Section",
            "title": "Recent",
            "children": [
                {"id": "1", "title": "Item 1"},
                {"id": "2", "title": "Item 2"}
            ]
        }"#;

        let child: ListChild = serde_json::from_str(json).unwrap();
        match child {
            ListChild::Section(section) => {
                assert_eq!(section.title, Some("Recent".to_string()));
                assert_eq!(section.children.len(), 2);
            }
            _ => panic!("Expected Section"),
        }
    }

    #[test]
    fn test_list_with_accessories() {
        let json = r##"{
            "id": "repo",
            "title": "my-repo",
            "accessories": [
                {"type": "icon", "icon": {"type": "system", "name": "star"}, "text": "42"},
                {"type": "tag", "value": "TypeScript", "color": "#3178c6"}
            ]
        }"##;

        let item: ListItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.accessories.len(), 2);
    }
}
