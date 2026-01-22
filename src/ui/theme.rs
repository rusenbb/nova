//! Theme definitions for Nova UI.
//!
//! Supports multiple color schemes: Catppuccin, Nord, Dracula, etc.

use iced::Color;

/// A Nova color theme.
#[derive(Debug, Clone)]
pub struct NovaTheme {
    pub name: &'static str,
    pub background: Color,
    pub surface: Color,
    pub text: Color,
    pub subtext: Color,
    pub accent: Color,
    pub border: Color,
    pub selection: Color,
}

impl NovaTheme {
    /// Parse a hex color string like "#cba6f7" to iced Color.
    pub fn from_hex(hex: &str) -> Color {
        let hex = hex.trim_start_matches('#');
        if hex.len() >= 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0;
            Color::from_rgb(r, g, b)
        } else {
            Color::from_rgb(0.5, 0.5, 0.5)
        }
    }

    /// Create a color with alpha transparency.
    pub fn with_alpha(color: Color, alpha: f32) -> Color {
        Color::from_rgba(color.r, color.g, color.b, alpha)
    }

    /// Get theme by name.
    pub fn by_name(name: &str) -> Self {
        match name {
            "catppuccin-mocha" => Self::catppuccin_mocha(),
            "catppuccin-macchiato" => Self::catppuccin_macchiato(),
            "catppuccin-frappe" => Self::catppuccin_frappe(),
            "catppuccin-latte" => Self::catppuccin_latte(),
            "nord" => Self::nord(),
            "dracula" => Self::dracula(),
            "gruvbox-dark" => Self::gruvbox_dark(),
            "tokyo-night" => Self::tokyo_night(),
            "one-dark" => Self::one_dark(),
            _ => Self::catppuccin_mocha(), // Default
        }
    }

    /// Catppuccin Mocha theme (default).
    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "catppuccin-mocha",
            background: Self::from_hex("#1e1e2e"),
            surface: Self::from_hex("#313244"),
            text: Self::from_hex("#cdd6f4"),
            subtext: Self::from_hex("#6c7086"),
            accent: Self::from_hex("#cba6f7"),
            border: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
            selection: Self::from_hex("#45475a"),
        }
    }

    /// Catppuccin Macchiato theme.
    pub fn catppuccin_macchiato() -> Self {
        Self {
            name: "catppuccin-macchiato",
            background: Self::from_hex("#24273a"),
            surface: Self::from_hex("#363a4f"),
            text: Self::from_hex("#cad3f5"),
            subtext: Self::from_hex("#6e738d"),
            accent: Self::from_hex("#c6a0f6"),
            border: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
            selection: Self::from_hex("#494d64"),
        }
    }

    /// Catppuccin Frappe theme.
    pub fn catppuccin_frappe() -> Self {
        Self {
            name: "catppuccin-frappe",
            background: Self::from_hex("#303446"),
            surface: Self::from_hex("#414559"),
            text: Self::from_hex("#c6d0f5"),
            subtext: Self::from_hex("#737994"),
            accent: Self::from_hex("#ca9ee6"),
            border: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
            selection: Self::from_hex("#51576d"),
        }
    }

    /// Catppuccin Latte theme (light).
    pub fn catppuccin_latte() -> Self {
        Self {
            name: "catppuccin-latte",
            background: Self::from_hex("#eff1f5"),
            surface: Self::from_hex("#e6e9ef"),
            text: Self::from_hex("#4c4f69"),
            subtext: Self::from_hex("#6c6f85"),
            accent: Self::from_hex("#8839ef"),
            border: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
            selection: Self::from_hex("#ccd0da"),
        }
    }

    /// Nord theme.
    pub fn nord() -> Self {
        Self {
            name: "nord",
            background: Self::from_hex("#2e3440"),
            surface: Self::from_hex("#3b4252"),
            text: Self::from_hex("#eceff4"),
            subtext: Self::from_hex("#4c566a"),
            accent: Self::from_hex("#88c0d0"),
            border: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
            selection: Self::from_hex("#434c5e"),
        }
    }

    /// Dracula theme.
    pub fn dracula() -> Self {
        Self {
            name: "dracula",
            background: Self::from_hex("#282a36"),
            surface: Self::from_hex("#44475a"),
            text: Self::from_hex("#f8f8f2"),
            subtext: Self::from_hex("#6272a4"),
            accent: Self::from_hex("#bd93f9"),
            border: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
            selection: Self::from_hex("#44475a"),
        }
    }

    /// Gruvbox Dark theme.
    pub fn gruvbox_dark() -> Self {
        Self {
            name: "gruvbox-dark",
            background: Self::from_hex("#282828"),
            surface: Self::from_hex("#3c3836"),
            text: Self::from_hex("#ebdbb2"),
            subtext: Self::from_hex("#928374"),
            accent: Self::from_hex("#d3869b"),
            border: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
            selection: Self::from_hex("#504945"),
        }
    }

    /// Tokyo Night theme.
    pub fn tokyo_night() -> Self {
        Self {
            name: "tokyo-night",
            background: Self::from_hex("#1a1b26"),
            surface: Self::from_hex("#24283b"),
            text: Self::from_hex("#c0caf5"),
            subtext: Self::from_hex("#565f89"),
            accent: Self::from_hex("#bb9af7"),
            border: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
            selection: Self::from_hex("#33467c"),
        }
    }

    /// One Dark theme.
    pub fn one_dark() -> Self {
        Self {
            name: "one-dark",
            background: Self::from_hex("#282c34"),
            surface: Self::from_hex("#3e4451"),
            text: Self::from_hex("#abb2bf"),
            subtext: Self::from_hex("#5c6370"),
            accent: Self::from_hex("#c678dd"),
            border: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
            selection: Self::from_hex("#3e4451"),
        }
    }
}

impl Default for NovaTheme {
    fn default() -> Self {
        Self::catppuccin_mocha()
    }
}
