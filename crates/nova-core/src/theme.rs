/// A theme palette with RGB color tuples
#[derive(Debug, Clone)]
pub struct ThemePalette {
    /// Background color as (r, g, b)
    pub background: (u8, u8, u8),
    /// Primary text color as (r, g, b)
    pub text: (u8, u8, u8),
    /// Secondary/muted text color as (r, g, b)
    pub subtext: (u8, u8, u8),
    /// Default accent color as (r, g, b)
    pub accent: (u8, u8, u8),
    /// Whether this is a light theme
    pub is_light: bool,
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

/// Get the theme palette for a theme name
pub fn get_theme_palette(theme: &str) -> ThemePalette {
    match theme {
        "catppuccin-mocha" => ThemePalette {
            background: (30, 30, 46),
            text: (205, 214, 244),
            subtext: (108, 112, 134),
            accent: (203, 166, 247),
            is_light: false,
        },
        "catppuccin-macchiato" => ThemePalette {
            background: (36, 39, 58),
            text: (202, 211, 245),
            subtext: (110, 115, 141),
            accent: (198, 160, 246),
            is_light: false,
        },
        "catppuccin-frappe" => ThemePalette {
            background: (48, 52, 70),
            text: (198, 208, 245),
            subtext: (115, 121, 148),
            accent: (202, 158, 230),
            is_light: false,
        },
        "catppuccin-latte" => ThemePalette {
            background: (239, 241, 245),
            text: (76, 79, 105),
            subtext: (108, 111, 133),
            accent: (136, 57, 239),
            is_light: true,
        },
        "nord" => ThemePalette {
            background: (46, 52, 64),
            text: (236, 239, 244),
            subtext: (76, 86, 106),
            accent: (136, 192, 208),
            is_light: false,
        },
        "dracula" => ThemePalette {
            background: (40, 42, 54),
            text: (248, 248, 242),
            subtext: (98, 114, 164),
            accent: (189, 147, 249),
            is_light: false,
        },
        "gruvbox-dark" => ThemePalette {
            background: (40, 40, 40),
            text: (235, 219, 178),
            subtext: (146, 131, 116),
            accent: (250, 189, 47),
            is_light: false,
        },
        "tokyo-night" => ThemePalette {
            background: (26, 27, 38),
            text: (192, 202, 245),
            subtext: (86, 95, 137),
            accent: (122, 162, 247),
            is_light: false,
        },
        "one-dark" => ThemePalette {
            background: (40, 44, 52),
            text: (171, 178, 191),
            subtext: (92, 99, 112),
            accent: (198, 120, 221),
            is_light: false,
        },
        _ => get_theme_palette("catppuccin-mocha"),
    }
}

/// Get the list of available theme names
pub fn available_themes() -> &'static [&'static str] {
    &[
        "catppuccin-mocha",
        "catppuccin-macchiato",
        "catppuccin-frappe",
        "catppuccin-latte",
        "nord",
        "dracula",
        "gruvbox-dark",
        "tokyo-night",
        "one-dark",
    ]
}

/// Get theme colors as CSS-compatible RGB strings (for backward compatibility)
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
        _ => ("30, 30, 46", "#cdd6f4", "#6c7086"),
    }
}
