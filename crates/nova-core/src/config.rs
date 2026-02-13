use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct Config {
    pub general: GeneralConfig,
    pub appearance: AppearanceConfig,
    pub behavior: BehaviorConfig,
    #[serde(default)]
    pub aliases: Vec<AliasConfig>,
    #[serde(default)]
    pub quicklinks: Vec<QuicklinkConfig>,
    #[serde(default)]
    pub scripts: ScriptsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasConfig {
    pub keyword: String,
    pub name: String,
    pub target: String,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuicklinkConfig {
    pub keyword: String,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub icon: Option<String>,
}

impl QuicklinkConfig {
    pub fn has_query_placeholder(&self) -> bool {
        self.url.contains("{query}")
    }

    pub fn resolve_url(&self, query: &str) -> String {
        if self.has_query_placeholder() {
            self.url.replace("{query}", &urlencoding::encode(query))
        } else {
            self.url.clone()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScriptsConfig {
    pub directory: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub hotkey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    pub theme: String,
    pub accent_color: String,
    pub opacity: f64,
    pub window_width: i32,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    pub description_size: u32,
    pub description_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BehaviorConfig {
    pub autostart: bool,
    pub max_results: u32,
}


impl Default for ScriptsConfig {
    fn default() -> Self {
        Self {
            directory: "~/.config/nova/scripts".to_string(),
            enabled: true,
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            hotkey: "<Alt>space".to_string(),
        }
    }
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: "catppuccin-mocha".to_string(),
            accent_color: "#cba6f7".to_string(),
            opacity: 0.92,
            window_width: 600,
            window_x: None,
            window_y: None,
            description_size: 13,
            description_color: None,
        }
    }
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            autostart: false,
            max_results: 8,
        }
    }
}

impl Config {
    /// Get the config file path
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .map(|h| h.join(".config"))
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
            })
            .join("nova")
            .join("config.toml")
    }

    /// Load config from file, or return defaults if not found
    pub fn load() -> Self {
        let path = Self::config_path();

        let mut config = if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!("[Nova] Failed to parse config: {}", e);
                        Self::default()
                    }
                },
                Err(e) => {
                    eprintln!("[Nova] Failed to read config: {}", e);
                    Self::default()
                }
            }
        } else {
            Self::default()
        };

        config.validate();
        config
    }

    /// Validate and clamp config values to acceptable ranges
    fn validate(&mut self) {
        self.appearance.opacity = self.appearance.opacity.clamp(0.5, 1.0);
        self.behavior.max_results = self.behavior.max_results.clamp(1, 20);
        self.appearance.window_width = self.appearance.window_width.clamp(400, 1200);
        self.appearance.description_size = self.appearance.description_size.clamp(10, 24);
    }

    /// Save config to file
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&path, content).map_err(|e| format!("Failed to write config: {}", e))?;

        Ok(())
    }
}
