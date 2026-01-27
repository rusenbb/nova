//! Extension manifest parsing.
//!
//! Each extension has a `nova.toml` manifest file that defines:
//! - Extension metadata (name, title, version, etc.)
//! - Permissions (network, clipboard, storage, etc.)
//! - Commands (searchable actions)
//! - Preferences (user-configurable settings)

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::error::{ExtensionError, ExtensionResult};

/// Complete extension manifest parsed from `nova.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub extension: ExtensionMeta,

    #[serde(default)]
    pub permissions: PermissionsConfig,

    #[serde(default)]
    pub background: Option<BackgroundConfig>,

    #[serde(default)]
    pub commands: Vec<CommandConfig>,

    #[serde(default)]
    pub preferences: Vec<PreferenceConfig>,
}

impl ExtensionManifest {
    /// Load manifest from a directory containing `nova.toml`.
    pub fn load(extension_dir: &Path) -> ExtensionResult<Self> {
        let manifest_path = extension_dir.join("nova.toml");

        if !manifest_path.exists() {
            return Err(ExtensionError::ManifestNotFound(
                extension_dir.to_path_buf(),
            ));
        }

        let content = std::fs::read_to_string(&manifest_path)?;

        toml::from_str(&content).map_err(|e| ExtensionError::ManifestInvalid {
            path: manifest_path,
            message: e.to_string(),
        })
    }

    /// Validate the manifest for required fields and constraints.
    pub fn validate(&self) -> ExtensionResult<()> {
        if self.extension.name.is_empty() {
            return Err(ExtensionError::ManifestInvalid {
                path: "nova.toml".into(),
                message: "extension.name is required".to_string(),
            });
        }

        if self.extension.title.is_empty() {
            return Err(ExtensionError::ManifestInvalid {
                path: "nova.toml".into(),
                message: "extension.title is required".to_string(),
            });
        }

        if self.extension.version.is_empty() {
            return Err(ExtensionError::ManifestInvalid {
                path: "nova.toml".into(),
                message: "extension.version is required".to_string(),
            });
        }

        // Validate commands
        for cmd in &self.commands {
            if cmd.name.is_empty() {
                return Err(ExtensionError::ManifestInvalid {
                    path: "nova.toml".into(),
                    message: "command.name is required".to_string(),
                });
            }
            if cmd.title.is_empty() {
                return Err(ExtensionError::ManifestInvalid {
                    path: "nova.toml".into(),
                    message: format!("command '{}' requires a title", cmd.name),
                });
            }
        }

        // Validate background config
        if let Some(ref bg) = self.background {
            if bg.interval < 60 {
                return Err(ExtensionError::ManifestInvalid {
                    path: "nova.toml".into(),
                    message: "background.interval must be at least 60 seconds".to_string(),
                });
            }
        }

        Ok(())
    }
}

/// Extension metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionMeta {
    /// Unique identifier (lowercase, alphanumeric, hyphens).
    pub name: String,

    /// Human-readable display name.
    pub title: String,

    /// Short description.
    #[serde(default)]
    pub description: String,

    /// Semantic version (e.g., "1.0.0").
    pub version: String,

    /// Author name.
    #[serde(default)]
    pub author: Option<String>,

    /// Source repository URL.
    #[serde(default)]
    pub repo: Option<String>,

    /// Homepage URL.
    #[serde(default)]
    pub homepage: Option<String>,

    /// SPDX license identifier.
    #[serde(default)]
    pub license: Option<String>,

    /// Icon filename relative to extension root.
    #[serde(default)]
    pub icon: Option<String>,

    /// Additional search keywords.
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Minimum Nova version required.
    #[serde(default)]
    pub nova_version: Option<String>,
}

/// Permission configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionsConfig {
    /// Allowed network domains (supports wildcards like "*.github.com").
    #[serde(default)]
    pub network: Vec<String>,

    /// Whether clipboard access is allowed.
    #[serde(default)]
    pub clipboard: bool,

    /// Whether persistent storage is allowed.
    #[serde(default)]
    pub storage: bool,

    /// Whether system notifications are allowed.
    #[serde(default)]
    pub notifications: bool,

    /// Whether background execution is allowed.
    #[serde(default)]
    pub background: bool,
}

impl PermissionsConfig {
    /// Check if a domain is allowed for network access.
    pub fn is_domain_allowed(&self, domain: &str) -> bool {
        self.network.iter().any(|pattern| {
            if pattern.starts_with("*.") {
                // Wildcard subdomain match
                let suffix = &pattern[1..]; // ".example.com"
                domain.ends_with(suffix) || domain == &pattern[2..]
            } else {
                domain == pattern
            }
        })
    }
}

/// Background execution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundConfig {
    /// Interval in seconds between background ticks (minimum: 60).
    #[serde(default = "default_background_interval")]
    pub interval: u64,

    /// Whether to run immediately when extension loads.
    #[serde(default)]
    pub run_on_load: bool,
}

fn default_background_interval() -> u64 {
    300 // 5 minutes
}

/// Command configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandConfig {
    /// Unique command identifier within the extension.
    pub name: String,

    /// Human-readable command title.
    pub title: String,

    /// Command description.
    #[serde(default)]
    pub description: String,

    /// UI mode: "list", "detail", or "form".
    #[serde(default = "default_command_mode")]
    pub mode: CommandMode,

    /// Additional trigger keywords.
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Arguments that can be passed via deep link.
    #[serde(default)]
    pub arguments: Vec<ArgumentConfig>,
}

fn default_command_mode() -> CommandMode {
    CommandMode::List
}

/// Command UI mode.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CommandMode {
    #[default]
    List,
    Detail,
    Form,
}

/// Command argument configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentConfig {
    pub name: String,

    #[serde(rename = "type", default = "default_argument_type")]
    pub arg_type: String,

    #[serde(default)]
    pub required: bool,
}

fn default_argument_type() -> String {
    "string".to_string()
}

/// User preference configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceConfig {
    /// Preference key.
    pub name: String,

    /// Human-readable title.
    pub title: String,

    /// Optional description.
    #[serde(default)]
    pub description: String,

    /// Preference type.
    #[serde(rename = "type")]
    pub pref_type: PreferenceType,

    /// Whether this preference is required.
    #[serde(default)]
    pub required: bool,

    /// Default value (as string, will be parsed based on type).
    #[serde(default)]
    pub default: Option<String>,

    /// Options for dropdown type.
    #[serde(default)]
    pub options: Vec<PreferenceOption>,
}

/// Preference type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PreferenceType {
    Text,
    Password,
    Checkbox,
    Dropdown,
}

/// Dropdown option.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceOption {
    pub value: String,
    pub title: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_manifest() {
        let toml = r#"
[extension]
name = "test"
title = "Test Extension"
version = "1.0.0"
"#;

        let manifest: ExtensionManifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.extension.name, "test");
        assert_eq!(manifest.extension.title, "Test Extension");
        assert_eq!(manifest.extension.version, "1.0.0");
        assert!(manifest.commands.is_empty());
        assert!(manifest.preferences.is_empty());
    }

    #[test]
    fn test_parse_full_manifest() {
        let toml = r#"
[extension]
name = "github"
title = "GitHub"
description = "Manage PRs and issues"
version = "1.0.0"
author = "nova-extensions"
icon = "icon.png"
keywords = ["git", "code"]

[permissions]
network = ["api.github.com", "*.github.com"]
clipboard = true
storage = true
notifications = true
background = true

[background]
interval = 300
run_on_load = true

[[commands]]
name = "search-repos"
title = "Search Repositories"
description = "Search your GitHub repos"
mode = "list"
keywords = ["gh", "repo"]

[[commands]]
name = "create-issue"
title = "Create Issue"
mode = "form"

[[preferences]]
name = "token"
title = "Personal Access Token"
type = "password"
required = true

[[preferences]]
name = "showPrivate"
title = "Show Private Repos"
type = "checkbox"
default = "true"
"#;

        let manifest: ExtensionManifest = toml::from_str(toml).unwrap();

        // Extension meta
        assert_eq!(manifest.extension.name, "github");
        assert_eq!(
            manifest.extension.author,
            Some("nova-extensions".to_string())
        );

        // Permissions
        assert!(manifest.permissions.clipboard);
        assert_eq!(manifest.permissions.network.len(), 2);

        // Background
        assert!(manifest.background.is_some());
        let bg = manifest.background.as_ref().unwrap();
        assert_eq!(bg.interval, 300);
        assert!(bg.run_on_load);

        // Commands
        assert_eq!(manifest.commands.len(), 2);
        assert_eq!(manifest.commands[0].mode, CommandMode::List);
        assert_eq!(manifest.commands[1].mode, CommandMode::Form);

        // Preferences
        assert_eq!(manifest.preferences.len(), 2);
        assert_eq!(manifest.preferences[0].pref_type, PreferenceType::Password);
        assert!(manifest.preferences[0].required);
    }

    #[test]
    fn test_domain_matching() {
        let perms = PermissionsConfig {
            network: vec!["api.github.com".to_string(), "*.example.com".to_string()],
            ..Default::default()
        };

        assert!(perms.is_domain_allowed("api.github.com"));
        assert!(!perms.is_domain_allowed("github.com"));
        assert!(perms.is_domain_allowed("example.com"));
        assert!(perms.is_domain_allowed("sub.example.com"));
        assert!(perms.is_domain_allowed("deep.sub.example.com"));
        assert!(!perms.is_domain_allowed("notexample.com"));
    }

    #[test]
    fn test_validate_manifest() {
        let mut manifest = ExtensionManifest {
            extension: ExtensionMeta {
                name: "test".to_string(),
                title: "Test".to_string(),
                description: String::new(),
                version: "1.0.0".to_string(),
                author: None,
                repo: None,
                homepage: None,
                license: None,
                icon: None,
                keywords: vec![],
                nova_version: None,
            },
            permissions: PermissionsConfig::default(),
            background: None,
            commands: vec![],
            preferences: vec![],
        };

        // Valid manifest
        assert!(manifest.validate().is_ok());

        // Missing name
        manifest.extension.name = String::new();
        assert!(manifest.validate().is_err());
        manifest.extension.name = "test".to_string();

        // Invalid background interval
        manifest.background = Some(BackgroundConfig {
            interval: 30, // Too short
            run_on_load: false,
        });
        assert!(manifest.validate().is_err());
    }
}
