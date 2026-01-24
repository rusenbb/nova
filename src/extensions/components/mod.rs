//! Component system for Nova extensions.
//!
//! This module defines the component types that extensions can render.
//! Components are serializable data structures that describe UI elements.
//!
//! # Component Types
//!
//! - **List** - Searchable list of items with optional sections
//! - **Detail** - Markdown content with metadata sidebar
//! - **Form** - User input form with various field types
//!
//! # Example
//!
//! ```ignore
//! use nova::extensions::components::{Component, ListComponent, ListChild, ListItem};
//!
//! let component = Component::List(ListComponent {
//!     search_bar_placeholder: Some("Search...".to_string()),
//!     children: vec![
//!         ListChild::Item(ListItem {
//!             id: "1".to_string(),
//!             title: "Hello".to_string(),
//!             ..Default::default()
//!         })
//!     ],
//!     ..Default::default()
//! });
//! ```

mod action;
mod common;
mod detail;
mod form;
mod list;
mod validation;

pub use action::{Action, ActionPanel, ActionStyle};
pub use common::{Accessory, DateFormat, Icon, KeyModifier, Shortcut};
pub use detail::{DetailComponent, DetailMetadata, MetadataItem, MetadataLink};
pub use form::{
    DropdownOption, FieldValidation, FormCheckbox, FormComponent, FormDatePicker, FormDropdown,
    FormField, FormTextField, TextFieldType,
};
pub use list::{ListChild, ListComponent, ListFiltering, ListItem, ListSection};
pub use validation::{ComponentError, Validate};

use serde::{Deserialize, Serialize};

/// Root component type that can be rendered by an extension.
///
/// Extensions call `Nova.render(component)` to display one of these
/// component types in the Nova window.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Component {
    /// A searchable list of items
    List(ListComponent),
    /// A detail view with markdown and metadata
    Detail(DetailComponent),
    /// A form for user input
    Form(FormComponent),
}

impl Default for Component {
    fn default() -> Self {
        Component::List(ListComponent::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_list_roundtrip() {
        let component = Component::List(ListComponent {
            is_loading: false,
            search_bar_placeholder: Some("Search repositories...".to_string()),
            filtering: ListFiltering::Default,
            on_search_change: None,
            on_selection_change: None,
            children: vec![
                ListChild::Section(ListSection {
                    title: Some("Recent".to_string()),
                    subtitle: None,
                    children: vec![ListItem {
                        id: "repo-1".to_string(),
                        title: "my-repo".to_string(),
                        subtitle: Some("A cool project".to_string()),
                        icon: Some(Icon::System {
                            name: "folder.fill".to_string(),
                        }),
                        accessories: vec![
                            Accessory::Tag {
                                value: "Rust".to_string(),
                                color: Some("#dea584".to_string()),
                            },
                            Accessory::Text {
                                text: "2h ago".to_string(),
                            },
                        ],
                        keywords: vec!["rust".to_string(), "cli".to_string()],
                        actions: Some(ActionPanel {
                            title: None,
                            children: vec![Action {
                                id: "open".to_string(),
                                title: "Open in Browser".to_string(),
                                icon: Some(Icon::System {
                                    name: "safari".to_string(),
                                }),
                                shortcut: Some(Shortcut {
                                    modifiers: vec![KeyModifier::Cmd],
                                    key: "o".to_string(),
                                }),
                                style: ActionStyle::Default,
                                on_action: Some("cb_open".to_string()),
                            }],
                        }),
                    }],
                }),
            ],
        });

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&component).unwrap();
        assert!(json.contains("\"type\": \"List\""));
        assert!(json.contains("\"searchBarPlaceholder\""));

        // Deserialize back
        let parsed: Component = serde_json::from_str(&json).unwrap();
        match parsed {
            Component::List(list) => {
                assert_eq!(
                    list.search_bar_placeholder,
                    Some("Search repositories...".to_string())
                );
                assert_eq!(list.children.len(), 1);
            }
            _ => panic!("Expected List component"),
        }
    }

    #[test]
    fn test_component_detail_roundtrip() {
        let component = Component::Detail(DetailComponent {
            markdown: Some("# Hello World\n\nThis is **markdown**.".to_string()),
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
                        title: "Website".to_string(),
                        text: None,
                        icon: None,
                        link: Some(MetadataLink {
                            text: "Visit".to_string(),
                            url: "https://example.com".to_string(),
                        }),
                    },
                ],
            }),
        });

        let json = serde_json::to_string(&component).unwrap();
        assert!(json.contains("\"type\":\"Detail\""));

        let parsed: Component = serde_json::from_str(&json).unwrap();
        match parsed {
            Component::Detail(detail) => {
                assert!(detail.markdown.is_some());
                assert!(detail.metadata.is_some());
            }
            _ => panic!("Expected Detail component"),
        }
    }

    #[test]
    fn test_component_form_roundtrip() {
        let component = Component::Form(FormComponent {
            is_loading: false,
            on_submit: Some("cb_submit".to_string()),
            on_change: None,
            children: vec![
                FormField::TextField(FormTextField {
                    id: "name".to_string(),
                    title: "Name".to_string(),
                    placeholder: Some("Enter name".to_string()),
                    default_value: None,
                    field_type: TextFieldType::Text,
                    validation: Some(FieldValidation {
                        required: true,
                        min_length: Some(2),
                        max_length: Some(50),
                        pattern: None,
                    }),
                }),
                FormField::Dropdown(FormDropdown {
                    id: "language".to_string(),
                    title: "Language".to_string(),
                    default_value: Some("rust".to_string()),
                    options: vec![
                        DropdownOption {
                            value: "rust".to_string(),
                            title: "Rust".to_string(),
                            icon: None,
                        },
                        DropdownOption {
                            value: "ts".to_string(),
                            title: "TypeScript".to_string(),
                            icon: None,
                        },
                    ],
                }),
                FormField::Checkbox(FormCheckbox {
                    id: "public".to_string(),
                    title: "Visibility".to_string(),
                    label: Some("Make repository public".to_string()),
                    default_value: true,
                }),
            ],
        });

        let json = serde_json::to_string(&component).unwrap();
        assert!(json.contains("\"type\":\"Form\""));
        assert!(json.contains("Form.TextField"));
        assert!(json.contains("Form.Dropdown"));
        assert!(json.contains("Form.Checkbox"));

        let parsed: Component = serde_json::from_str(&json).unwrap();
        match parsed {
            Component::Form(form) => {
                assert_eq!(form.children.len(), 3);
            }
            _ => panic!("Expected Form component"),
        }
    }

    #[test]
    fn test_component_validation() {
        use validation::Validate;

        // Valid component
        let valid = Component::List(ListComponent {
            children: vec![ListChild::Item(ListItem {
                id: "1".to_string(),
                title: "Test".to_string(),
                ..Default::default()
            })],
            ..Default::default()
        });
        assert!(valid.validate().is_ok());

        // Invalid component (empty id)
        let invalid = Component::List(ListComponent {
            children: vec![ListChild::Item(ListItem {
                id: "".to_string(),
                title: "Test".to_string(),
                ..Default::default()
            })],
            ..Default::default()
        });
        assert!(invalid.validate().is_err());
    }
}
