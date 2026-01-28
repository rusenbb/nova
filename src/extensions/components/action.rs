//! Action and ActionPanel component definitions.
//!
//! Actions are keyboard-driven commands that appear in an ActionPanel.
//! They can be attached to list items, detail views, or forms.

use serde::{Deserialize, Serialize};

use super::common::{Icon, Shortcut};

/// Container for actions associated with a component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPanel {
    /// Optional title for the action panel section
    #[serde(default)]
    pub title: Option<String>,
    /// List of actions
    #[serde(default)]
    pub children: Vec<Action>,
}

/// A single action that can be triggered by the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    /// Unique identifier for the action
    pub id: String,
    /// Display title
    pub title: String,
    /// Optional icon
    #[serde(default)]
    pub icon: Option<Icon>,
    /// Optional keyboard shortcut
    #[serde(default)]
    pub shortcut: Option<Shortcut>,
    /// Visual style
    #[serde(default)]
    pub style: ActionStyle,
    /// Callback ID to invoke when action is triggered
    #[serde(default)]
    pub on_action: Option<String>,
}

/// Visual style for an action.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ActionStyle {
    /// Normal action
    #[default]
    Default,
    /// Destructive action (shown in red)
    Destructive,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_panel_serialize() {
        let panel = ActionPanel {
            title: Some("Actions".to_string()),
            children: vec![Action {
                id: "open".to_string(),
                title: "Open".to_string(),
                icon: Some(Icon::System {
                    name: "safari".to_string(),
                }),
                shortcut: Some(Shortcut {
                    modifiers: vec![super::super::common::KeyModifier::Cmd],
                    key: "o".to_string(),
                }),
                style: ActionStyle::Default,
                on_action: Some("cb_1".to_string()),
            }],
        };

        let json = serde_json::to_string(&panel).unwrap();
        assert!(json.contains("\"title\":\"Actions\""));
        assert!(json.contains("\"id\":\"open\""));
    }

    #[test]
    fn test_action_deserialize() {
        let json = r#"{
            "id": "delete",
            "title": "Delete Item",
            "style": "destructive",
            "onAction": "cb_delete"
        }"#;

        let action: Action = serde_json::from_str(json).unwrap();
        assert_eq!(action.id, "delete");
        assert_eq!(action.title, "Delete Item");
        assert_eq!(action.style, ActionStyle::Destructive);
        assert_eq!(action.on_action, Some("cb_delete".to_string()));
    }

    #[test]
    fn test_action_style_default() {
        let json = r#"{"id": "test", "title": "Test"}"#;
        let action: Action = serde_json::from_str(json).unwrap();
        assert_eq!(action.style, ActionStyle::Default);
    }
}
