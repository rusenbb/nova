use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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
    // Description text customization
    pub description_size: u32,
    pub description_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BehaviorConfig {
    pub autostart: bool,
    pub max_results: u32,
}

#[allow(clippy::derivable_impls)]
impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            appearance: AppearanceConfig::default(),
            behavior: BehaviorConfig::default(),
            aliases: Vec::new(),
            quicklinks: Vec::new(),
            scripts: ScriptsConfig::default(),
        }
    }
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
            description_color: None, // Uses theme's subtext color by default
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
                // Fallback: ~ is not expanded by PathBuf, so use dirs::home_dir
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
        // Clamp opacity to valid range (0.5 - 1.0)
        self.appearance.opacity = self.appearance.opacity.clamp(0.5, 1.0);

        // Clamp max_results to reasonable range (1 - 20)
        self.behavior.max_results = self.behavior.max_results.clamp(1, 20);

        // Clamp window_width to reasonable range (400 - 1200)
        self.appearance.window_width = self.appearance.window_width.clamp(400, 1200);

        // Clamp description_size to reasonable range (10 - 24)
        self.appearance.description_size = self.appearance.description_size.clamp(10, 24);
    }

    /// Save config to file
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();

        // Create config directory if it doesn't exist
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

/// Set up autostart by creating/removing desktop file
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    let autostart_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("autostart");

    fs::create_dir_all(&autostart_dir)
        .map_err(|e| format!("Failed to create autostart directory: {}", e))?;

    let desktop_file = autostart_dir.join("nova.desktop");

    if enabled {
        let exe_path =
            std::env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))?;

        let content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=Nova\n\
             Comment=Keyboard-driven productivity launcher\n\
             Exec={}\n\
             StartupNotify=false\n\
             X-GNOME-Autostart-enabled=true\n",
            exe_path.display()
        );

        fs::write(&desktop_file, content)
            .map_err(|e| format!("Failed to write autostart file: {}", e))?;

        println!("[Nova] Autostart enabled");
    } else {
        if desktop_file.exists() {
            fs::remove_file(&desktop_file)
                .map_err(|e| format!("Failed to remove autostart file: {}", e))?;
        }
        println!("[Nova] Autostart disabled");
    }

    Ok(())
}

/// Parse a hex color string like "#cba6f7" to (r, g, b)
pub fn parse_hex_color(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(203);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(166);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(247);
        (r, g, b)
    } else {
        (203, 166, 247) // Default to catppuccin mauve
    }
}

/// Get theme colors based on theme name
pub fn get_theme_colors(theme: &str) -> (&'static str, &'static str, &'static str) {
    // Returns (background_rgb, text_color, subtext_color)
    match theme {
        "catppuccin-mocha" => ("30, 30, 46", "#cdd6f4", "#6c7086"),
        "catppuccin-macchiato" => ("36, 39, 58", "#cad3f5", "#6e738d"),
        "catppuccin-frappe" => ("48, 52, 70", "#c6d0f5", "#737994"),
        "catppuccin-latte" => ("239, 241, 245", "#4c4f69", "#6c6f85"),
        "nord" => ("46, 52, 64", "#eceff4", "#4c566a"),
        "dracula" => ("40, 42, 54", "#f8f8f2", "#6272a4"),
        "gruvbox-dark" => ("40, 40, 40", "#ebdbb2", "#928374"),
        "tokyo-night" => ("26, 27, 38", "#c0caf5", "#565f89"),
        "one-dark" => ("40, 44, 52", "#abb2bf", "#5c6370"),
        _ => ("30, 30, 46", "#cdd6f4", "#6c7086"), // Default to catppuccin-mocha
    }
}

/// Generate CSS with appearance settings from config
pub fn generate_css(config: &AppearanceConfig) -> String {
    let (bg_rgb, text_color, subtext_color) = get_theme_colors(&config.theme);
    let (accent_r, accent_g, accent_b) = parse_hex_color(&config.accent_color);
    let opacity = config.opacity;
    let desc_size = config.description_size;
    let desc_color = config.description_color.as_deref().unwrap_or(subtext_color);

    format!(
        r#"
    window {{
        background-color: transparent;
    }}

    .nova-container {{
        background-color: rgba({bg_rgb}, {opacity});
        border-radius: 12px;
        padding: 12px;
        border: 1px solid rgba(255, 255, 255, 0.1);
    }}

    .nova-entry {{
        background-color: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 8px;
        padding: 12px 16px;
        font-size: 18px;
        color: {text_color};
        min-width: 550px;
    }}

    .nova-entry:focus {{
        border-color: rgba({accent_r}, {accent_g}, {accent_b}, 0.5);
        outline: none;
    }}

    .nova-results {{
        background-color: transparent;
        margin-top: 8px;
    }}

    .nova-results row {{
        padding: 8px 12px;
        border-radius: 6px;
        background-color: transparent;
    }}

    .nova-results row:selected,
    list.nova-results row:selected,
    row:selected {{
        background-color: rgba({accent_r}, {accent_g}, {accent_b}, 0.35);
        border-left: 3px solid rgba({accent_r}, {accent_g}, {accent_b}, 0.9);
    }}

    .nova-result-name {{
        font-size: 15px;
        font-weight: 500;
        color: {text_color};
    }}

    .nova-result-desc {{
        font-size: {desc_size}px;
        color: {desc_color};
        margin-top: 2px;
    }}

    .nova-entry-container {{
        background-color: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 8px;
        padding: 8px 12px;
    }}

    .nova-entry-container:focus-within {{
        border-color: rgba({accent_r}, {accent_g}, {accent_b}, 0.5);
    }}

    .nova-command-pill {{
        background-color: rgba({accent_r}, {accent_g}, {accent_b}, 0.25);
        border-radius: 4px;
        padding: 4px 10px;
        margin-right: 8px;
        font-size: 13px;
        font-weight: 500;
        color: {accent_color};
    }}

    .nova-entry-in-container {{
        background-color: transparent;
        border: none;
        padding: 4px 4px;
        font-size: 18px;
        color: {text_color};
        min-width: 450px;
    }}

    .nova-entry-in-container:focus {{
        outline: none;
        border: none;
    }}
"#,
        bg_rgb = bg_rgb,
        opacity = opacity,
        text_color = text_color,
        desc_color = desc_color,
        desc_size = desc_size,
        accent_r = accent_r,
        accent_g = accent_g,
        accent_b = accent_b,
        accent_color = config.accent_color,
    )
}
