//! Component validation.
//!
//! Provides validation for component trees to ensure required fields
//! are present and values are valid.

use thiserror::Error;

use super::detail::DetailComponent;
use super::form::{FormCheckbox, FormComponent, FormDatePicker, FormDropdown, FormField, FormTextField};
use super::list::{ListChild, ListComponent, ListItem, ListSection};
use super::Component;

/// Error type for component validation failures.
#[derive(Debug, Error)]
pub enum ComponentError {
    /// A required field is missing
    #[error("Missing required field '{field}' in {component}")]
    MissingRequired { component: String, field: String },

    /// A field has an invalid value
    #[error("Invalid value for '{field}' in {component}: {reason}")]
    InvalidValue {
        component: String,
        field: String,
        reason: String,
    },

    /// The component structure is invalid
    #[error("Invalid component structure: {0}")]
    InvalidStructure(String),

    /// Error deserializing component from JSON
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

/// Trait for validating components.
pub trait Validate {
    /// Validate this component and return an error if invalid.
    fn validate(&self) -> Result<(), ComponentError>;
}

impl Validate for Component {
    fn validate(&self) -> Result<(), ComponentError> {
        match self {
            Component::List(list) => list.validate(),
            Component::Detail(detail) => detail.validate(),
            Component::Form(form) => form.validate(),
        }
    }
}

impl Validate for ListComponent {
    fn validate(&self) -> Result<(), ComponentError> {
        for child in &self.children {
            child.validate()?;
        }
        Ok(())
    }
}

impl Validate for ListChild {
    fn validate(&self) -> Result<(), ComponentError> {
        match self {
            ListChild::Item(item) => item.validate(),
            ListChild::Section(section) => section.validate(),
        }
    }
}

impl Validate for ListItem {
    fn validate(&self) -> Result<(), ComponentError> {
        if self.id.is_empty() {
            return Err(ComponentError::MissingRequired {
                component: "List.Item".to_string(),
                field: "id".to_string(),
            });
        }
        if self.title.is_empty() {
            return Err(ComponentError::MissingRequired {
                component: "List.Item".to_string(),
                field: "title".to_string(),
            });
        }
        Ok(())
    }
}

impl Validate for ListSection {
    fn validate(&self) -> Result<(), ComponentError> {
        for item in &self.children {
            item.validate()?;
        }
        Ok(())
    }
}

impl Validate for DetailComponent {
    fn validate(&self) -> Result<(), ComponentError> {
        // Detail is valid as long as it has markdown or is loading
        // No strict requirements
        Ok(())
    }
}

impl Validate for FormComponent {
    fn validate(&self) -> Result<(), ComponentError> {
        for field in &self.children {
            field.validate()?;
        }
        Ok(())
    }
}

impl Validate for FormField {
    fn validate(&self) -> Result<(), ComponentError> {
        match self {
            FormField::TextField(tf) => tf.validate(),
            FormField::Dropdown(dd) => dd.validate(),
            FormField::Checkbox(cb) => cb.validate(),
            FormField::DatePicker(dp) => dp.validate(),
        }
    }
}

impl Validate for FormTextField {
    fn validate(&self) -> Result<(), ComponentError> {
        if self.id.is_empty() {
            return Err(ComponentError::MissingRequired {
                component: "Form.TextField".to_string(),
                field: "id".to_string(),
            });
        }
        if self.title.is_empty() {
            return Err(ComponentError::MissingRequired {
                component: "Form.TextField".to_string(),
                field: "title".to_string(),
            });
        }
        Ok(())
    }
}

impl Validate for FormDropdown {
    fn validate(&self) -> Result<(), ComponentError> {
        if self.id.is_empty() {
            return Err(ComponentError::MissingRequired {
                component: "Form.Dropdown".to_string(),
                field: "id".to_string(),
            });
        }
        if self.title.is_empty() {
            return Err(ComponentError::MissingRequired {
                component: "Form.Dropdown".to_string(),
                field: "title".to_string(),
            });
        }
        Ok(())
    }
}

impl Validate for FormCheckbox {
    fn validate(&self) -> Result<(), ComponentError> {
        if self.id.is_empty() {
            return Err(ComponentError::MissingRequired {
                component: "Form.Checkbox".to_string(),
                field: "id".to_string(),
            });
        }
        if self.title.is_empty() {
            return Err(ComponentError::MissingRequired {
                component: "Form.Checkbox".to_string(),
                field: "title".to_string(),
            });
        }
        Ok(())
    }
}

impl Validate for FormDatePicker {
    fn validate(&self) -> Result<(), ComponentError> {
        if self.id.is_empty() {
            return Err(ComponentError::MissingRequired {
                component: "Form.DatePicker".to_string(),
                field: "id".to_string(),
            });
        }
        if self.title.is_empty() {
            return Err(ComponentError::MissingRequired {
                component: "Form.DatePicker".to_string(),
                field: "title".to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_list() {
        let list = ListComponent {
            children: vec![ListChild::Item(ListItem {
                id: "1".to_string(),
                title: "Item".to_string(),
                subtitle: None,
                icon: None,
                accessories: vec![],
                keywords: vec![],
                actions: None,
            })],
            ..Default::default()
        };

        assert!(list.validate().is_ok());
    }

    #[test]
    fn test_invalid_list_item_missing_id() {
        let item = ListItem {
            id: "".to_string(),
            title: "Item".to_string(),
            subtitle: None,
            icon: None,
            accessories: vec![],
            keywords: vec![],
            actions: None,
        };

        let err = item.validate().unwrap_err();
        assert!(err.to_string().contains("id"));
    }

    #[test]
    fn test_invalid_list_item_missing_title() {
        let item = ListItem {
            id: "1".to_string(),
            title: "".to_string(),
            subtitle: None,
            icon: None,
            accessories: vec![],
            keywords: vec![],
            actions: None,
        };

        let err = item.validate().unwrap_err();
        assert!(err.to_string().contains("title"));
    }

    #[test]
    fn test_valid_form() {
        let form = FormComponent {
            children: vec![FormField::TextField(FormTextField {
                id: "name".to_string(),
                title: "Name".to_string(),
                placeholder: None,
                default_value: None,
                field_type: super::super::form::TextFieldType::Text,
                validation: None,
            })],
            ..Default::default()
        };

        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_invalid_form_field() {
        let field = FormTextField {
            id: "".to_string(),
            title: "Name".to_string(),
            placeholder: None,
            default_value: None,
            field_type: super::super::form::TextFieldType::Text,
            validation: None,
        };

        let err = field.validate().unwrap_err();
        assert!(err.to_string().contains("id"));
    }

    #[test]
    fn test_component_enum_validation() {
        let component = Component::List(ListComponent {
            children: vec![ListChild::Item(ListItem {
                id: "".to_string(), // Invalid!
                title: "Item".to_string(),
                subtitle: None,
                icon: None,
                accessories: vec![],
                keywords: vec![],
                actions: None,
            })],
            ..Default::default()
        });

        assert!(component.validate().is_err());
    }
}
