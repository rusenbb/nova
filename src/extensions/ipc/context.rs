//! Nova context for extension execution.
//!
//! NovaContext holds all the state needed by an extension during execution,
//! including platform access, storage, permissions, and preferences.

use std::collections::HashMap;
use std::sync::Arc;

use crate::extensions::components::Component;
use crate::extensions::manifest::PermissionsConfig;
use crate::extensions::storage::ExtensionStorage;
use crate::platform::Platform;

/// Context provided to extensions during execution.
///
/// This is stored in the Deno OpState and accessed by ops.
pub struct NovaContext {
    /// Extension identifier.
    pub extension_id: String,

    /// Platform trait for system operations.
    pub platform: Arc<dyn Platform>,

    /// Extension-specific storage.
    pub storage: ExtensionStorage,

    /// Permissions granted to this extension.
    pub permissions: PermissionsConfig,

    /// User-configured preferences for this extension.
    pub preferences: HashMap<String, serde_json::Value>,

    /// Navigation stack for push/pop views.
    pub navigation_stack: Vec<Component>,

    /// Currently rendered component tree (strongly typed).
    pub rendered_component: Option<Component>,

    /// Whether the extension requested window close.
    pub should_close: bool,
}

impl NovaContext {
    /// Create a new context for an extension.
    pub fn new(
        extension_id: String,
        platform: Arc<dyn Platform>,
        storage: ExtensionStorage,
        permissions: PermissionsConfig,
        preferences: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            extension_id,
            platform,
            storage,
            permissions,
            preferences,
            navigation_stack: Vec::new(),
            rendered_component: None,
            should_close: false,
        }
    }

    /// Check if a permission is granted.
    pub fn check_permission(&self, permission: &str) -> Result<(), anyhow::Error> {
        let allowed = match permission {
            "clipboard" => self.permissions.clipboard,
            "notifications" => self.permissions.notifications,
            "storage" => self.permissions.storage,
            "background" => self.permissions.background,
            _ => false,
        };

        if allowed {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Permission '{}' not granted for extension '{}'",
                permission,
                self.extension_id
            ))
        }
    }

    /// Set the rendered component.
    pub fn set_rendered_component(&mut self, component: Component) {
        self.rendered_component = Some(component);
    }

    /// Get the current rendered component.
    pub fn get_rendered_component(&self) -> Option<&Component> {
        self.rendered_component.as_ref()
    }

    /// Take the rendered component (consuming it).
    pub fn take_rendered_component(&mut self) -> Option<Component> {
        self.rendered_component.take()
    }
}
