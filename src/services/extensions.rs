//! Extension system for Nova
//!
//! Extensions are bundles of commands that extend Nova's functionality.
//! Each extension is a directory containing:
//! - extension.toml: Manifest file defining the extension and its commands
//! - Script files: Executable scripts that implement commands
//! - Optional icon files

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Extension manifest (extension.toml)
#[derive(Debug, Clone, Deserialize)]
pub struct ExtensionManifest {
    pub extension: ExtensionMeta,
    #[serde(default)]
    pub commands: Vec<ExtensionCommand>,
}

/// Extension metadata
#[derive(Debug, Clone, Deserialize)]
pub struct ExtensionMeta {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub icon: Option<String>,
}

/// A command defined by an extension
#[derive(Debug, Clone, Deserialize)]
pub struct ExtensionCommand {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub keyword: String,
    pub script: String,
    #[serde(default)]
    pub has_argument: bool,
    #[serde(default = "default_output_mode")]
    pub output: OutputMode,
    #[serde(default)]
    pub icon: Option<String>,
}

fn default_output_mode() -> OutputMode {
    OutputMode::Silent
}

/// How to handle script output
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    /// Show results as a list (JSON array)
    List,
    /// Show as desktop notification
    Notification,
    /// Copy to clipboard
    Clipboard,
    /// No output handling
    #[default]
    Silent,
}

/// A loaded extension with resolved paths
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields reserved for future features (listing, icons)
pub struct LoadedExtension {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub path: PathBuf,
    pub icon_path: Option<PathBuf>,
    pub commands: Vec<LoadedCommand>,
}

/// A command ready to execute
#[derive(Debug, Clone)]
#[allow(dead_code)] // icon_path reserved for future UI features
pub struct LoadedCommand {
    pub id: String,
    pub extension_id: String,
    pub name: String,
    pub description: String,
    pub keyword: String,
    pub script_path: PathBuf,
    pub has_argument: bool,
    pub output: OutputMode,
    pub icon_path: Option<PathBuf>,
}

/// Result item from extension script (JSON output)
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // action reserved for future result actions
pub struct ResultItem {
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub action: Option<ResultAction>,
}

/// Action to perform when selecting a result
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
#[allow(dead_code)] // Reserved for future result action handling
pub enum ResultAction {
    Open {
        url: String,
    },
    Copy {
        text: String,
    },
    Run {
        command: String,
    },
    #[serde(other)]
    None,
}

/// Extension script output (JSON format)
#[derive(Debug, Clone, Deserialize)]
pub struct ScriptOutput {
    #[serde(default)]
    pub items: Vec<ResultItem>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Manages all loaded extensions
#[allow(dead_code)] // extensions field reserved for future listing
pub struct ExtensionManager {
    extensions: Vec<LoadedExtension>,
    commands_by_keyword: HashMap<String, LoadedCommand>,
}

impl ExtensionManager {
    /// Load extensions from the extensions directory
    pub fn load(extensions_dir: &Path) -> Self {
        let mut extensions = Vec::new();
        let mut commands_by_keyword = HashMap::new();

        if !extensions_dir.exists() {
            // Create extensions directory if it doesn't exist
            let _ = fs::create_dir_all(extensions_dir);
            println!(
                "[Nova] Created extensions directory: {}",
                extensions_dir.display()
            );
            return Self {
                extensions,
                commands_by_keyword,
            };
        }

        // Scan for extension directories
        if let Ok(entries) = fs::read_dir(extensions_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(ext) = Self::load_extension(&path) {
                        // Register commands by keyword
                        for cmd in &ext.commands {
                            commands_by_keyword.insert(cmd.keyword.to_lowercase(), cmd.clone());
                        }
                        extensions.push(ext);
                    }
                }
            }
        }

        println!(
            "[Nova] Loaded {} extensions with {} commands",
            extensions.len(),
            commands_by_keyword.len()
        );

        Self {
            extensions,
            commands_by_keyword,
        }
    }

    /// Load a single extension from a directory
    fn load_extension(path: &Path) -> Option<LoadedExtension> {
        let manifest_path = path.join("extension.toml");
        if !manifest_path.exists() {
            return None;
        }

        let content = fs::read_to_string(&manifest_path).ok()?;
        let manifest: ExtensionManifest = toml::from_str(&content).ok()?;

        // Generate extension ID from directory name
        let id = path.file_name()?.to_str()?.to_string();

        // Resolve icon path
        let icon_path = manifest.extension.icon.as_ref().map(|i| path.join(i));

        // Load commands with resolved paths
        let commands: Vec<LoadedCommand> = manifest
            .commands
            .into_iter()
            .map(|cmd| {
                let script_path = path.join(&cmd.script);
                let cmd_icon = cmd.icon.as_ref().map(|i| path.join(i));

                LoadedCommand {
                    id: cmd.id,
                    extension_id: id.clone(),
                    name: cmd.name,
                    description: cmd.description,
                    keyword: cmd.keyword,
                    script_path,
                    has_argument: cmd.has_argument,
                    output: cmd.output,
                    icon_path: cmd_icon.or_else(|| icon_path.clone()),
                }
            })
            .collect();

        Some(LoadedExtension {
            id,
            name: manifest.extension.name,
            description: manifest.extension.description,
            author: manifest.extension.author,
            version: manifest.extension.version,
            path: path.to_path_buf(),
            icon_path,
            commands,
        })
    }

    /// Get all loaded extensions
    #[allow(dead_code)] // Reserved for future extension listing UI
    pub fn extensions(&self) -> &[LoadedExtension] {
        &self.extensions
    }

    /// Get a command by keyword
    #[allow(dead_code)] // Reserved for exact keyword lookup
    pub fn get_command(&self, keyword: &str) -> Option<&LoadedCommand> {
        self.commands_by_keyword.get(&keyword.to_lowercase())
    }

    /// Search commands by partial keyword or name match
    pub fn search_commands(&self, query: &str) -> Vec<&LoadedCommand> {
        let query_lower = query.to_lowercase();
        self.commands_by_keyword
            .values()
            .filter(|cmd| {
                cmd.keyword.to_lowercase().starts_with(&query_lower)
                    || cmd.name.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Execute an extension command
    pub fn execute_command(
        &self,
        cmd: &LoadedCommand,
        argument: Option<&str>,
    ) -> Result<ScriptOutput, String> {
        if !cmd.script_path.exists() {
            return Err(format!("Script not found: {}", cmd.script_path.display()));
        }

        let mut command = Command::new(&cmd.script_path);

        // Pass argument if provided
        if let Some(arg) = argument {
            command.arg(arg);
        }

        // Set environment variables for context
        command.env("NOVA_EXTENSION_ID", &cmd.extension_id);
        command.env("NOVA_COMMAND_ID", &cmd.id);
        if let Some(arg) = argument {
            command.env("NOVA_QUERY", arg);
        }
        if let Some(config_dir) = dirs::config_dir() {
            command.env("NOVA_CONFIG_DIR", config_dir.join("nova"));
        }

        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let output = command
            .output()
            .map_err(|e| format!("Failed to execute script: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Err(format!("Script failed: {}", stderr));
        }

        // Try to parse as JSON, fall back to plain text
        if cmd.output == OutputMode::List {
            // Expect JSON output for list mode
            serde_json::from_str(&stdout)
                .map_err(|e| format!("Invalid JSON output: {} - {}", e, stdout))
        } else {
            // For other modes, treat output as plain text
            Ok(ScriptOutput {
                items: vec![ResultItem {
                    title: stdout.trim().to_string(),
                    subtitle: None,
                    action: None,
                }],
                error: None,
            })
        }
    }
}

/// Get the default extensions directory
pub fn get_extensions_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nova")
        .join("extensions")
}
