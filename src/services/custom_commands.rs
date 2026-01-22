use crate::config::{AliasConfig, Config, QuicklinkConfig};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub enum ScriptOutputMode {
    #[default]
    Silent,
    Notification,
    Clipboard,
    Inline,
}

#[derive(Debug, Clone)]
pub struct ScriptEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub path: PathBuf,
    pub keywords: Vec<String>,
    pub has_argument: bool,
    pub output_mode: ScriptOutputMode,
}

pub struct CustomCommandsIndex {
    pub aliases: Vec<AliasConfig>,
    pub quicklinks: Vec<QuicklinkConfig>,
    pub scripts: Vec<ScriptEntry>,
}

impl CustomCommandsIndex {
    pub fn new(config: &Config) -> Self {
        let scripts = if config.scripts.enabled {
            Self::load_scripts(&config.scripts.directory)
        } else {
            Vec::new()
        };

        Self {
            aliases: config.aliases.clone(),
            quicklinks: config.quicklinks.clone(),
            scripts,
        }
    }

    fn load_scripts(directory: &str) -> Vec<ScriptEntry> {
        let expanded = shellexpand::tilde(directory);
        let path = Path::new(expanded.as_ref());

        if !path.exists() {
            // Create the scripts directory if it doesn't exist
            if let Err(e) = fs::create_dir_all(path) {
                eprintln!("[Nova] Failed to create scripts directory: {}", e);
            }
            return Vec::new();
        }

        let mut scripts = Vec::new();

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let file_path = entry.path();
                if file_path.is_file() {
                    if let Some(script) = Self::parse_script_metadata(&file_path) {
                        scripts.push(script);
                    }
                }
            }
        }

        println!("[Nova] Loaded {} scripts", scripts.len());
        scripts
    }

    fn parse_script_metadata(path: &PathBuf) -> Option<ScriptEntry> {
        let content = fs::read_to_string(path).ok()?;
        let id = path.file_stem()?.to_string_lossy().to_string();

        // Parse TOML-like header from script comments
        let metadata = Self::extract_metadata_header(&content);

        // Skip files without nova metadata
        if metadata.is_empty() {
            return None;
        }

        Some(ScriptEntry {
            id: id.clone(),
            name: metadata.get("name").cloned().unwrap_or_else(|| id.clone()),
            description: metadata.get("description").cloned().unwrap_or_default(),
            icon: metadata.get("icon").cloned(),
            path: path.clone(),
            keywords: metadata
                .get("keywords")
                .map(|k| k.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
            has_argument: metadata
                .get("argument")
                .map(|v| v == "true")
                .unwrap_or(false),
            output_mode: match metadata.get("output").map(|s| s.as_str()) {
                Some("notification") => ScriptOutputMode::Notification,
                Some("clipboard") => ScriptOutputMode::Clipboard,
                Some("inline") => ScriptOutputMode::Inline,
                _ => ScriptOutputMode::Silent,
            },
        })
    }

    fn extract_metadata_header(content: &str) -> HashMap<String, String> {
        let mut metadata = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Stop at first non-comment, non-empty, non-shebang line
            if !trimmed.starts_with('#') && !trimmed.is_empty() {
                break;
            }

            // Skip shebang
            if trimmed.starts_with("#!") {
                continue;
            }

            // Look for # nova: key = "value" or # nova: key = value
            if let Some(rest) = trimmed.strip_prefix("# nova:") {
                if let Some((key, value)) = rest.split_once('=') {
                    let key = key.trim().to_string();
                    let value = value.trim().trim_matches('"').to_string();
                    metadata.insert(key, value);
                }
            }
        }

        metadata
    }

    #[allow(dead_code)] // For future hot-reload feature
    pub fn reload_scripts(&mut self, config: &Config) {
        self.scripts = if config.scripts.enabled {
            Self::load_scripts(&config.scripts.directory)
        } else {
            Vec::new()
        };
    }
}
