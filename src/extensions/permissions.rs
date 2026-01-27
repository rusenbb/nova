//! Permission management for Nova extensions.
//!
//! This module provides:
//! - `PermissionSet` - The set of permissions requested/granted to an extension
//! - `PermissionStore` - Persistent storage for granted permissions
//! - Permission checking utilities

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::manifest::PermissionsConfig;

/// Permission-related errors.
#[derive(Debug, Error)]
pub enum PermissionError {
    #[error("Permission denied: {permission}")]
    Denied { permission: String },

    #[error("Network access to '{domain}' not allowed")]
    NetworkDomainDenied { domain: String },

    #[error("Filesystem access to '{path}' not allowed")]
    FilesystemPathDenied { path: String },

    #[error("Permission not yet granted by user")]
    NotYetGranted { permission: String },

    #[error("Failed to load permissions: {0}")]
    LoadFailed(String),

    #[error("Failed to save permissions: {0}")]
    SaveFailed(String),
}

/// Result type for permission operations.
pub type PermissionResult<T> = Result<T, PermissionError>;

/// The set of permissions an extension may request or have been granted.
///
/// This struct maps directly to the permissions declared in `nova.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissionSet {
    /// Whether clipboard read/write access is allowed.
    #[serde(default)]
    pub clipboard: bool,

    /// Network configuration - allowed domains (supports wildcards like "*.github.com").
    #[serde(default)]
    pub network: NetworkPermission,

    /// Filesystem access configuration.
    #[serde(default)]
    pub filesystem: FilesystemPermission,

    /// Whether system operations (notifications, open URL, etc.) are allowed.
    #[serde(default)]
    pub system: bool,

    /// Whether persistent storage is allowed.
    #[serde(default)]
    pub storage: bool,

    /// Whether background execution is allowed.
    #[serde(default)]
    pub background: bool,
}

/// Network permission configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkPermission {
    /// Whether network access is enabled at all.
    #[serde(default)]
    pub enabled: bool,

    /// Allowed domains (supports wildcards like "*.github.com").
    #[serde(default)]
    pub allowed_domains: Vec<String>,
}

/// Filesystem permission configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FilesystemPermission {
    /// Whether filesystem access is enabled at all.
    #[serde(default)]
    pub enabled: bool,

    /// Allowed paths (can include home directory expansion).
    #[serde(default)]
    pub allowed_paths: Vec<String>,

    /// Whether read access is allowed.
    #[serde(default)]
    pub read: bool,

    /// Whether write access is allowed.
    #[serde(default)]
    pub write: bool,
}

impl PermissionSet {
    /// Create a new empty permission set (all denied).
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a permission set from manifest permissions config.
    pub fn from_manifest(config: &PermissionsConfig) -> Self {
        Self {
            clipboard: config.clipboard,
            network: NetworkPermission {
                enabled: !config.network.is_empty(),
                allowed_domains: config.network.clone(),
            },
            filesystem: FilesystemPermission::default(),
            system: config.notifications, // Map notifications to system
            storage: config.storage,
            background: config.background,
        }
    }

    /// Check if clipboard access is allowed.
    pub fn check_clipboard(&self) -> PermissionResult<()> {
        if self.clipboard {
            Ok(())
        } else {
            Err(PermissionError::Denied {
                permission: "clipboard".to_string(),
            })
        }
    }

    /// Check if network access to a specific domain is allowed.
    pub fn check_network(&self, domain: &str) -> PermissionResult<()> {
        if !self.network.enabled {
            return Err(PermissionError::Denied {
                permission: "network".to_string(),
            });
        }

        if self.is_domain_allowed(domain) {
            Ok(())
        } else {
            Err(PermissionError::NetworkDomainDenied {
                domain: domain.to_string(),
            })
        }
    }

    /// Check if a domain is in the allowed list.
    fn is_domain_allowed(&self, domain: &str) -> bool {
        // If "*" is in the list, all domains are allowed
        if self.network.allowed_domains.iter().any(|d| d == "*") {
            return true;
        }

        self.network.allowed_domains.iter().any(|pattern| {
            if pattern.starts_with("*.") {
                // Wildcard subdomain match
                let suffix = &pattern[1..]; // ".example.com"
                domain.ends_with(suffix) || domain == &pattern[2..]
            } else {
                domain == pattern
            }
        })
    }

    /// Check if filesystem access to a specific path is allowed.
    pub fn check_filesystem(&self, path: &str, write: bool) -> PermissionResult<()> {
        if !self.filesystem.enabled {
            return Err(PermissionError::Denied {
                permission: "filesystem".to_string(),
            });
        }

        if write && !self.filesystem.write {
            return Err(PermissionError::Denied {
                permission: "filesystem.write".to_string(),
            });
        }

        if !write && !self.filesystem.read {
            return Err(PermissionError::Denied {
                permission: "filesystem.read".to_string(),
            });
        }

        if self.is_path_allowed(path) {
            Ok(())
        } else {
            Err(PermissionError::FilesystemPathDenied {
                path: path.to_string(),
            })
        }
    }

    /// Check if a path is in the allowed list.
    fn is_path_allowed(&self, path: &str) -> bool {
        // Expand home directory
        let expanded_path = if path.starts_with('~') {
            if let Some(home) = dirs::home_dir() {
                path.replacen('~', &home.to_string_lossy(), 1)
            } else {
                path.to_string()
            }
        } else {
            path.to_string()
        };

        self.filesystem.allowed_paths.iter().any(|allowed| {
            let expanded_allowed = if allowed.starts_with('~') {
                if let Some(home) = dirs::home_dir() {
                    allowed.replacen('~', &home.to_string_lossy(), 1)
                } else {
                    allowed.clone()
                }
            } else {
                allowed.clone()
            };

            // Check if path starts with allowed path
            expanded_path.starts_with(&expanded_allowed)
        })
    }

    /// Check if system operations are allowed.
    pub fn check_system(&self) -> PermissionResult<()> {
        if self.system {
            Ok(())
        } else {
            Err(PermissionError::Denied {
                permission: "system".to_string(),
            })
        }
    }

    /// Check if storage is allowed.
    pub fn check_storage(&self) -> PermissionResult<()> {
        if self.storage {
            Ok(())
        } else {
            Err(PermissionError::Denied {
                permission: "storage".to_string(),
            })
        }
    }

    /// Check if background execution is allowed.
    pub fn check_background(&self) -> PermissionResult<()> {
        if self.background {
            Ok(())
        } else {
            Err(PermissionError::Denied {
                permission: "background".to_string(),
            })
        }
    }

    /// Get a list of all permissions that are enabled.
    pub fn enabled_permissions(&self) -> Vec<&'static str> {
        let mut perms = Vec::new();
        if self.clipboard {
            perms.push("clipboard");
        }
        if self.network.enabled {
            perms.push("network");
        }
        if self.filesystem.enabled {
            perms.push("filesystem");
        }
        if self.system {
            perms.push("system");
        }
        if self.storage {
            perms.push("storage");
        }
        if self.background {
            perms.push("background");
        }
        perms
    }
}

/// Granted permissions for a single extension.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtensionGrants {
    /// The permissions that have been granted.
    pub permissions: PermissionSet,

    /// When the permissions were last updated (Unix timestamp).
    #[serde(default)]
    pub updated_at: u64,

    /// Extension version when permissions were granted.
    #[serde(default)]
    pub extension_version: Option<String>,
}

/// Persistent storage for granted permissions.
///
/// Stores permissions in `~/.nova/permissions.json`.
pub struct PermissionStore {
    /// Path to the permissions file.
    path: PathBuf,

    /// In-memory cache of granted permissions by extension ID.
    grants: HashMap<String, ExtensionGrants>,

    /// Whether the cache has unsaved changes.
    dirty: bool,
}

impl PermissionStore {
    /// Create a new permission store, loading from disk if the file exists.
    pub fn new() -> Self {
        let path = Self::default_path();
        let grants = Self::load_from_path(&path).unwrap_or_default();

        Self {
            path,
            grants,
            dirty: false,
        }
    }

    /// Create a permission store with a custom path.
    pub fn with_path(path: PathBuf) -> Self {
        let grants = Self::load_from_path(&path).unwrap_or_default();

        Self {
            path,
            grants,
            dirty: false,
        }
    }

    /// Get the default path for the permissions file.
    fn default_path() -> PathBuf {
        dirs::data_dir()
            .map(|d| d.join("nova").join("permissions.json"))
            .unwrap_or_else(|| PathBuf::from("~/.nova/permissions.json"))
    }

    /// Load grants from a file.
    fn load_from_path(path: &PathBuf) -> Option<HashMap<String, ExtensionGrants>> {
        if !path.exists() {
            return None;
        }

        let contents = fs::read_to_string(path).ok()?;
        serde_json::from_str(&contents).ok()
    }

    /// Check if an extension has any granted permissions.
    pub fn has_grants(&self, extension_id: &str) -> bool {
        self.grants.contains_key(extension_id)
    }

    /// Get the granted permissions for an extension.
    pub fn get_grants(&self, extension_id: &str) -> Option<&ExtensionGrants> {
        self.grants.get(extension_id)
    }

    /// Get the granted permission set for an extension (or empty if none).
    pub fn get_permissions(&self, extension_id: &str) -> PermissionSet {
        self.grants
            .get(extension_id)
            .map(|g| g.permissions.clone())
            .unwrap_or_default()
    }

    /// Grant permissions to an extension.
    pub fn grant(
        &mut self,
        extension_id: &str,
        permissions: PermissionSet,
        version: Option<String>,
    ) {
        let grants = ExtensionGrants {
            permissions,
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            extension_version: version,
        };

        self.grants.insert(extension_id.to_string(), grants);
        self.dirty = true;
    }

    /// Revoke all permissions for an extension.
    pub fn revoke(&mut self, extension_id: &str) {
        if self.grants.remove(extension_id).is_some() {
            self.dirty = true;
        }
    }

    /// Revoke a specific permission for an extension.
    pub fn revoke_permission(&mut self, extension_id: &str, permission: &str) {
        if let Some(grants) = self.grants.get_mut(extension_id) {
            match permission {
                "clipboard" => grants.permissions.clipboard = false,
                "network" => grants.permissions.network.enabled = false,
                "filesystem" => grants.permissions.filesystem.enabled = false,
                "system" => grants.permissions.system = false,
                "storage" => grants.permissions.storage = false,
                "background" => grants.permissions.background = false,
                _ => return,
            }
            grants.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            self.dirty = true;
        }
    }

    /// Get all extensions with granted permissions.
    pub fn all_extensions(&self) -> Vec<&String> {
        self.grants.keys().collect()
    }

    /// Get all grants (extension_id -> grants).
    pub fn all_grants(&self) -> &HashMap<String, ExtensionGrants> {
        &self.grants
    }

    /// Check which permissions from a requested set need user consent.
    ///
    /// Returns the list of permissions that have not yet been granted.
    pub fn needs_consent(&self, extension_id: &str, requested: &PermissionSet) -> Vec<String> {
        let granted = self.get_permissions(extension_id);
        let mut needs = Vec::new();

        if requested.clipboard && !granted.clipboard {
            needs.push("clipboard".to_string());
        }
        if requested.network.enabled && !granted.network.enabled {
            needs.push("network".to_string());
        }
        if requested.filesystem.enabled && !granted.filesystem.enabled {
            needs.push("filesystem".to_string());
        }
        if requested.system && !granted.system {
            needs.push("system".to_string());
        }
        if requested.storage && !granted.storage {
            needs.push("storage".to_string());
        }
        if requested.background && !granted.background {
            needs.push("background".to_string());
        }

        needs
    }

    /// Save grants to disk.
    pub fn save(&mut self) -> PermissionResult<()> {
        if !self.dirty {
            return Ok(());
        }

        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                PermissionError::SaveFailed(format!("Failed to create directory: {}", e))
            })?;
        }

        // Serialize and write
        let contents = serde_json::to_string_pretty(&self.grants)
            .map_err(|e| PermissionError::SaveFailed(format!("Serialization failed: {}", e)))?;

        fs::write(&self.path, contents)
            .map_err(|e| PermissionError::SaveFailed(format!("Write failed: {}", e)))?;

        self.dirty = false;
        Ok(())
    }

    /// Reload grants from disk.
    pub fn reload(&mut self) -> PermissionResult<()> {
        self.grants = Self::load_from_path(&self.path).unwrap_or_default();
        self.dirty = false;
        Ok(())
    }
}

impl Default for PermissionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PermissionStore {
    fn drop(&mut self) {
        // Best-effort save on drop
        let _ = self.save();
    }
}

/// Human-readable description of a permission.
pub fn permission_description(permission: &str) -> &'static str {
    match permission {
        "clipboard" => "Read and write to the system clipboard",
        "network" => "Make network requests to allowed domains",
        "filesystem" => "Access files on your computer",
        "system" => "Show notifications and open URLs",
        "storage" => "Store data persistently",
        "background" => "Run in the background",
        _ => "Unknown permission",
    }
}

/// Icon name for a permission (SF Symbols).
pub fn permission_icon(permission: &str) -> &'static str {
    match permission {
        "clipboard" => "doc.on.clipboard",
        "network" => "network",
        "filesystem" => "folder",
        "system" => "bell",
        "storage" => "externaldrive",
        "background" => "clock.arrow.circlepath",
        _ => "questionmark.circle",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_permission_set_defaults() {
        let perms = PermissionSet::new();
        assert!(!perms.clipboard);
        assert!(!perms.network.enabled);
        assert!(!perms.filesystem.enabled);
        assert!(!perms.system);
        assert!(!perms.storage);
        assert!(!perms.background);
    }

    #[test]
    fn test_permission_checks() {
        let mut perms = PermissionSet::new();

        // All should fail by default
        assert!(perms.check_clipboard().is_err());
        assert!(perms.check_system().is_err());
        assert!(perms.check_storage().is_err());

        // Enable clipboard
        perms.clipboard = true;
        assert!(perms.check_clipboard().is_ok());

        // Enable system
        perms.system = true;
        assert!(perms.check_system().is_ok());
    }

    #[test]
    fn test_network_domain_matching() {
        let mut perms = PermissionSet::new();
        perms.network.enabled = true;
        perms.network.allowed_domains =
            vec!["api.github.com".to_string(), "*.example.com".to_string()];

        // Exact match
        assert!(perms.check_network("api.github.com").is_ok());

        // Wildcard subdomain
        assert!(perms.check_network("sub.example.com").is_ok());
        assert!(perms.check_network("deep.sub.example.com").is_ok());
        assert!(perms.check_network("example.com").is_ok());

        // Not allowed
        assert!(perms.check_network("github.com").is_err());
        assert!(perms.check_network("notexample.com").is_err());
    }

    #[test]
    fn test_wildcard_all_domains() {
        let mut perms = PermissionSet::new();
        perms.network.enabled = true;
        perms.network.allowed_domains = vec!["*".to_string()];

        assert!(perms.check_network("any.domain.com").is_ok());
        assert!(perms.check_network("localhost").is_ok());
    }

    #[test]
    fn test_permission_store_basic() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("permissions.json");

        let mut store = PermissionStore::with_path(path.clone());

        // Initially empty
        assert!(!store.has_grants("test-ext"));
        assert!(store.get_grants("test-ext").is_none());

        // Grant permissions
        let mut perms = PermissionSet::new();
        perms.clipboard = true;
        perms.storage = true;

        store.grant("test-ext", perms.clone(), Some("1.0.0".to_string()));

        assert!(store.has_grants("test-ext"));
        let grants = store.get_grants("test-ext").unwrap();
        assert!(grants.permissions.clipboard);
        assert!(grants.permissions.storage);
        assert_eq!(grants.extension_version, Some("1.0.0".to_string()));

        // Save and reload
        store.save().unwrap();

        let store2 = PermissionStore::with_path(path);
        assert!(store2.has_grants("test-ext"));
    }

    #[test]
    fn test_permission_store_revoke() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("permissions.json");

        let mut store = PermissionStore::with_path(path);

        let mut perms = PermissionSet::new();
        perms.clipboard = true;
        perms.storage = true;

        store.grant("test-ext", perms, None);
        assert!(store.has_grants("test-ext"));

        // Revoke specific permission
        store.revoke_permission("test-ext", "clipboard");
        let grants = store.get_grants("test-ext").unwrap();
        assert!(!grants.permissions.clipboard);
        assert!(grants.permissions.storage);

        // Revoke all
        store.revoke("test-ext");
        assert!(!store.has_grants("test-ext"));
    }

    #[test]
    fn test_needs_consent() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("permissions.json");

        let mut store = PermissionStore::with_path(path);

        // Grant some permissions
        let mut granted = PermissionSet::new();
        granted.clipboard = true;
        store.grant("test-ext", granted, None);

        // Request more
        let mut requested = PermissionSet::new();
        requested.clipboard = true;
        requested.storage = true;
        requested.network.enabled = true;

        let needs = store.needs_consent("test-ext", &requested);
        assert!(needs.contains(&"storage".to_string()));
        assert!(needs.contains(&"network".to_string()));
        assert!(!needs.contains(&"clipboard".to_string()));
    }

    #[test]
    fn test_from_manifest() {
        let config = PermissionsConfig {
            network: vec!["api.github.com".to_string()],
            clipboard: true,
            storage: true,
            notifications: true,
            background: false,
        };

        let perms = PermissionSet::from_manifest(&config);

        assert!(perms.clipboard);
        assert!(perms.network.enabled);
        assert_eq!(
            perms.network.allowed_domains,
            vec!["api.github.com".to_string()]
        );
        assert!(perms.storage);
        assert!(perms.system);
        assert!(!perms.background);
    }

    #[test]
    fn test_enabled_permissions() {
        let mut perms = PermissionSet::new();
        perms.clipboard = true;
        perms.network.enabled = true;
        perms.storage = true;

        let enabled = perms.enabled_permissions();
        assert!(enabled.contains(&"clipboard"));
        assert!(enabled.contains(&"network"));
        assert!(enabled.contains(&"storage"));
        assert!(!enabled.contains(&"system"));
        assert!(!enabled.contains(&"filesystem"));
        assert!(!enabled.contains(&"background"));
    }
}
