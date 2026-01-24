//! Form component definitions.
//!
//! Forms allow extensions to collect user input with various field types
//! including text fields, dropdowns, and checkboxes.

use serde::{Deserialize, Serialize};

use super::common::Icon;

/// Form component - collects user input.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FormComponent {
    /// Whether the form is loading/submitting
    #[serde(default)]
    pub is_loading: bool,

    /// Callback ID for form submission
    #[serde(default)]
    pub on_submit: Option<String>,

    /// Callback ID for value changes
    #[serde(default)]
    pub on_change: Option<String>,

    /// Form fields
    #[serde(default)]
    pub children: Vec<FormField>,
}

/// A form field (text, dropdown, or checkbox).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FormField {
    /// Text input field
    #[serde(rename = "Form.TextField")]
    TextField(FormTextField),
    /// Dropdown/select field
    #[serde(rename = "Form.Dropdown")]
    Dropdown(FormDropdown),
    /// Checkbox field
    #[serde(rename = "Form.Checkbox")]
    Checkbox(FormCheckbox),
    /// Date picker field
    #[serde(rename = "Form.DatePicker")]
    DatePicker(FormDatePicker),
}

/// Text input field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormTextField {
    /// Unique identifier
    pub id: String,

    /// Field label
    pub title: String,

    /// Placeholder text
    #[serde(default)]
    pub placeholder: Option<String>,

    /// Default value
    #[serde(default)]
    pub default_value: Option<String>,

    /// Input type
    #[serde(default)]
    pub field_type: TextFieldType,

    /// Validation rules
    #[serde(default)]
    pub validation: Option<FieldValidation>,
}

/// Text field input type.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TextFieldType {
    /// Normal text input
    #[default]
    Text,
    /// Password input (masked)
    Password,
    /// Number input
    Number,
}

/// Validation rules for form fields.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FieldValidation {
    /// Whether the field is required
    #[serde(default)]
    pub required: bool,

    /// Regex pattern to match
    #[serde(default)]
    pub pattern: Option<String>,

    /// Minimum length
    #[serde(default)]
    pub min_length: Option<usize>,

    /// Maximum length
    #[serde(default)]
    pub max_length: Option<usize>,
}

/// Dropdown/select field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormDropdown {
    /// Unique identifier
    pub id: String,

    /// Field label
    pub title: String,

    /// Default selected value
    #[serde(default)]
    pub default_value: Option<String>,

    /// Available options
    #[serde(default)]
    pub options: Vec<DropdownOption>,
}

/// An option in a dropdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropdownOption {
    /// Value to submit
    pub value: String,
    /// Display title
    pub title: String,
    /// Optional icon
    #[serde(default)]
    pub icon: Option<Icon>,
}

/// Checkbox field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormCheckbox {
    /// Unique identifier
    pub id: String,

    /// Field label
    pub title: String,

    /// Additional label text
    #[serde(default)]
    pub label: Option<String>,

    /// Default checked state
    #[serde(default)]
    pub default_value: bool,
}

/// Date picker field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormDatePicker {
    /// Unique identifier
    pub id: String,

    /// Field label
    pub title: String,

    /// Default date (ISO 8601)
    #[serde(default)]
    pub default_value: Option<String>,

    /// Whether to include time selection
    #[serde(default)]
    pub include_time: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_component() {
        let form = FormComponent {
            is_loading: false,
            on_submit: Some("cb_submit".to_string()),
            on_change: None,
            children: vec![
                FormField::TextField(FormTextField {
                    id: "name".to_string(),
                    title: "Name".to_string(),
                    placeholder: Some("Enter your name".to_string()),
                    default_value: None,
                    field_type: TextFieldType::Text,
                    validation: Some(FieldValidation {
                        required: true,
                        ..Default::default()
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
                            value: "typescript".to_string(),
                            title: "TypeScript".to_string(),
                            icon: None,
                        },
                    ],
                }),
            ],
        };

        let json = serde_json::to_string_pretty(&form).unwrap();
        assert!(json.contains("\"onSubmit\""));
        assert!(json.contains("Form.TextField"));
        assert!(json.contains("Form.Dropdown"));
    }

    #[test]
    fn test_form_field_deserialize() {
        let json = r#"{
            "type": "Form.TextField",
            "id": "email",
            "title": "Email",
            "fieldType": "text",
            "validation": {
                "required": true,
                "pattern": "^[^@]+@[^@]+$"
            }
        }"#;

        let field: FormField = serde_json::from_str(json).unwrap();
        match field {
            FormField::TextField(tf) => {
                assert_eq!(tf.id, "email");
                assert!(tf.validation.is_some());
                assert!(tf.validation.unwrap().required);
            }
            _ => panic!("Expected TextField"),
        }
    }

    #[test]
    fn test_checkbox_deserialize() {
        let json = r#"{
            "type": "Form.Checkbox",
            "id": "agree",
            "title": "Terms",
            "label": "I agree to the terms",
            "defaultValue": false
        }"#;

        let field: FormField = serde_json::from_str(json).unwrap();
        match field {
            FormField::Checkbox(cb) => {
                assert_eq!(cb.id, "agree");
                assert_eq!(cb.label, Some("I agree to the terms".to_string()));
            }
            _ => panic!("Expected Checkbox"),
        }
    }
}
