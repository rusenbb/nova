//! Theme system for Nova UI.
//!
//! This module provides a unified theme system that can be consumed by all frontends
//! (macOS Swift, Linux GTK). The theme is defined in `assets/theme.toml` and exposed
//! via FFI as JSON for easy consumption across language boundaries.

use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::OnceLock;

/// Global theme instance, loaded once on first access.
static THEME: OnceLock<Theme> = OnceLock::new();

/// Color definitions for the theme.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeColors {
    pub background: String,
    pub background_secondary: String,
    pub background_elevated: String,
    pub foreground: String,
    pub foreground_secondary: String,
    pub foreground_tertiary: String,
    pub accent: String,
    pub accent_hover: String,
    pub border: String,
    pub error: String,
    pub success: String,
    pub warning: String,
    pub selection: String,
    pub selection_background: String,
}

/// Spacing values in pixels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSpacing {
    pub xs: u32,
    pub sm: u32,
    pub md: u32,
    pub lg: u32,
    pub xl: u32,
    pub xxl: u32,
}

/// Typography definitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeTypography {
    pub font_family: String,
    pub font_size_xs: u32,
    pub font_size_sm: u32,
    pub font_size_md: u32,
    pub font_size_lg: u32,
    pub font_size_xl: u32,
    pub font_size_xxl: u32,
    pub font_weight_normal: u32,
    pub font_weight_medium: u32,
    pub font_weight_semibold: u32,
    pub font_weight_bold: u32,
}

/// Corner radii in pixels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeRadii {
    pub xs: u32,
    pub sm: u32,
    pub md: u32,
    pub lg: u32,
    pub xl: u32,
}

/// Shadow definitions (CSS format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeShadows {
    pub panel: String,
    pub dropdown: String,
    pub subtle: String,
}

/// Component-specific styling values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeComponents {
    // Panel
    pub panel_width: u32,
    pub panel_height: u32,
    pub panel_corner_radius: u32,

    // Search field
    pub search_field_height: u32,
    pub search_field_font_size: u32,
    pub search_field_padding_horizontal: u32,
    pub search_field_padding_vertical: u32,

    // List items
    pub list_item_height: u32,
    pub list_item_padding_horizontal: u32,
    pub list_item_padding_vertical: u32,
    pub list_item_icon_size: u32,
    pub list_item_corner_radius: u32,
    pub list_item_spacing: u32,

    // Extension views
    pub extension_item_height: u32,
    pub extension_icon_size: u32,

    // Icons
    pub icon_size_xs: u32,
    pub icon_size_sm: u32,
    pub icon_size_md: u32,
    pub icon_size_lg: u32,
    pub icon_size_xl: u32,
    pub icon_size_xxl: u32,

    // Dividers
    pub divider_thickness: u32,
    pub divider_margin: u32,
}

/// Animation timing values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeAnimation {
    pub duration_fast: u32,
    pub duration_normal: u32,
    pub duration_slow: u32,
    pub easing_default: String,
    pub easing_spring: String,
}

/// Complete theme definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub colors: ThemeColors,
    pub spacing: ThemeSpacing,
    pub typography: ThemeTypography,
    pub radii: ThemeRadii,
    pub shadows: ThemeShadows,
    pub components: ThemeComponents,
    pub animation: ThemeAnimation,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            colors: ThemeColors {
                background: "#1a1a1a".to_string(),
                background_secondary: "#252525".to_string(),
                background_elevated: "#2d2d2d".to_string(),
                foreground: "#ffffff".to_string(),
                foreground_secondary: "#a0a0a0".to_string(),
                foreground_tertiary: "#666666".to_string(),
                accent: "#007AFF".to_string(),
                accent_hover: "#0056CC".to_string(),
                border: "#3d3d3d".to_string(),
                error: "#FF453A".to_string(),
                success: "#32D74B".to_string(),
                warning: "#FFD60A".to_string(),
                selection: "#007AFF".to_string(),
                selection_background: "#1e3a5f".to_string(),
            },
            spacing: ThemeSpacing {
                xs: 4,
                sm: 8,
                md: 12,
                lg: 16,
                xl: 24,
                xxl: 32,
            },
            typography: ThemeTypography {
                font_family: "system-ui".to_string(),
                font_size_xs: 10,
                font_size_sm: 11,
                font_size_md: 13,
                font_size_lg: 16,
                font_size_xl: 20,
                font_size_xxl: 24,
                font_weight_normal: 400,
                font_weight_medium: 500,
                font_weight_semibold: 600,
                font_weight_bold: 700,
            },
            radii: ThemeRadii {
                xs: 2,
                sm: 4,
                md: 8,
                lg: 12,
                xl: 16,
            },
            shadows: ThemeShadows {
                panel: "0 8px 32px rgba(0,0,0,0.5)".to_string(),
                dropdown: "0 4px 16px rgba(0,0,0,0.4)".to_string(),
                subtle: "0 2px 8px rgba(0,0,0,0.2)".to_string(),
            },
            components: ThemeComponents {
                panel_width: 620,
                panel_height: 400,
                panel_corner_radius: 12,
                search_field_height: 48,
                search_field_font_size: 24,
                search_field_padding_horizontal: 16,
                search_field_padding_vertical: 12,
                list_item_height: 52,
                list_item_padding_horizontal: 10,
                list_item_padding_vertical: 8,
                list_item_icon_size: 36,
                list_item_corner_radius: 8,
                list_item_spacing: 2,
                extension_item_height: 44,
                extension_icon_size: 28,
                icon_size_xs: 12,
                icon_size_sm: 16,
                icon_size_md: 20,
                icon_size_lg: 24,
                icon_size_xl: 32,
                icon_size_xxl: 36,
                divider_thickness: 1,
                divider_margin: 8,
            },
            animation: ThemeAnimation {
                duration_fast: 100,
                duration_normal: 200,
                duration_slow: 300,
                easing_default: "ease-out".to_string(),
                easing_spring: "ease-in-out".to_string(),
            },
        }
    }
}

impl Theme {
    /// Load the theme from the embedded TOML file.
    pub fn load() -> Self {
        // Try to load from embedded theme.toml
        let theme_toml = include_str!("../assets/theme.toml");

        match toml::from_str::<Theme>(theme_toml) {
            Ok(theme) => theme,
            Err(e) => {
                eprintln!("[Nova] Warning: Failed to parse theme.toml: {}", e);
                eprintln!("[Nova] Using default theme");
                Self::default()
            }
        }
    }

    /// Get the global theme instance.
    pub fn global() -> &'static Theme {
        THEME.get_or_init(Self::load)
    }

    /// Convert theme to JSON string for FFI.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

// ============================================================================
// FFI Functions
// ============================================================================

/// Get the theme as a JSON string.
///
/// # Safety
/// The returned string must be freed using `nova_string_free()`.
#[no_mangle]
pub extern "C" fn nova_core_get_theme() -> *mut c_char {
    let theme = Theme::global();
    let json = theme.to_json();

    match CString::new(json) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Get a specific theme color by key.
///
/// # Arguments
/// * `key` - The color key (e.g., "background", "foreground", "accent")
///
/// # Returns
/// The color value as a hex string (e.g., "#1a1a1a"), or null if not found.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// The key must be a valid UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn nova_core_get_theme_color(key: *const c_char) -> *mut c_char {
    if key.is_null() {
        return std::ptr::null_mut();
    }

    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let theme = Theme::global();
    let color = match key_str {
        "background" => Some(&theme.colors.background),
        "backgroundSecondary" | "background_secondary" => Some(&theme.colors.background_secondary),
        "backgroundElevated" | "background_elevated" => Some(&theme.colors.background_elevated),
        "foreground" => Some(&theme.colors.foreground),
        "foregroundSecondary" | "foreground_secondary" => Some(&theme.colors.foreground_secondary),
        "foregroundTertiary" | "foreground_tertiary" => Some(&theme.colors.foreground_tertiary),
        "accent" => Some(&theme.colors.accent),
        "accentHover" | "accent_hover" => Some(&theme.colors.accent_hover),
        "border" => Some(&theme.colors.border),
        "error" => Some(&theme.colors.error),
        "success" => Some(&theme.colors.success),
        "warning" => Some(&theme.colors.warning),
        "selection" => Some(&theme.colors.selection),
        "selectionBackground" | "selection_background" => Some(&theme.colors.selection_background),
        _ => None,
    };

    match color {
        Some(c) => CString::new(c.as_str())
            .map(|s| s.into_raw())
            .unwrap_or(std::ptr::null_mut()),
        None => std::ptr::null_mut(),
    }
}

/// Get a theme spacing value by key.
///
/// # Arguments
/// * `key` - The spacing key (e.g., "xs", "sm", "md", "lg", "xl", "xxl")
///
/// # Returns
/// The spacing value in pixels, or 0 if not found.
///
/// # Safety
/// The key must be a valid UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn nova_core_get_theme_spacing(key: *const c_char) -> u32 {
    if key.is_null() {
        return 0;
    }

    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let theme = Theme::global();
    match key_str {
        "xs" => theme.spacing.xs,
        "sm" => theme.spacing.sm,
        "md" => theme.spacing.md,
        "lg" => theme.spacing.lg,
        "xl" => theme.spacing.xl,
        "xxl" => theme.spacing.xxl,
        _ => 0,
    }
}

/// Get a theme component value by key.
///
/// # Arguments
/// * `key` - The component key (e.g., "listItemHeight", "panelWidth")
///
/// # Returns
/// The component value, or 0 if not found.
///
/// # Safety
/// The key must be a valid UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn nova_core_get_theme_component(key: *const c_char) -> u32 {
    if key.is_null() {
        return 0;
    }

    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let theme = Theme::global();
    let components = &theme.components;

    match key_str {
        "panelWidth" | "panel_width" => components.panel_width,
        "panelHeight" | "panel_height" => components.panel_height,
        "panelCornerRadius" | "panel_corner_radius" => components.panel_corner_radius,
        "searchFieldHeight" | "search_field_height" => components.search_field_height,
        "searchFieldFontSize" | "search_field_font_size" => components.search_field_font_size,
        "searchFieldPaddingHorizontal" | "search_field_padding_horizontal" => {
            components.search_field_padding_horizontal
        }
        "searchFieldPaddingVertical" | "search_field_padding_vertical" => {
            components.search_field_padding_vertical
        }
        "listItemHeight" | "list_item_height" => components.list_item_height,
        "listItemPaddingHorizontal" | "list_item_padding_horizontal" => {
            components.list_item_padding_horizontal
        }
        "listItemPaddingVertical" | "list_item_padding_vertical" => {
            components.list_item_padding_vertical
        }
        "listItemIconSize" | "list_item_icon_size" => components.list_item_icon_size,
        "listItemCornerRadius" | "list_item_corner_radius" => components.list_item_corner_radius,
        "listItemSpacing" | "list_item_spacing" => components.list_item_spacing,
        "extensionItemHeight" | "extension_item_height" => components.extension_item_height,
        "extensionIconSize" | "extension_icon_size" => components.extension_icon_size,
        "iconSizeXs" | "icon_size_xs" => components.icon_size_xs,
        "iconSizeSm" | "icon_size_sm" => components.icon_size_sm,
        "iconSizeMd" | "icon_size_md" => components.icon_size_md,
        "iconSizeLg" | "icon_size_lg" => components.icon_size_lg,
        "iconSizeXl" | "icon_size_xl" => components.icon_size_xl,
        "iconSizeXxl" | "icon_size_xxl" => components.icon_size_xxl,
        "dividerThickness" | "divider_thickness" => components.divider_thickness,
        "dividerMargin" | "divider_margin" => components.divider_margin,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_load() {
        let theme = Theme::load();
        assert!(!theme.colors.background.is_empty());
        assert!(!theme.colors.accent.is_empty());
    }

    #[test]
    fn test_theme_to_json() {
        let theme = Theme::default();
        let json = theme.to_json();
        assert!(json.contains("background"));
        assert!(json.contains("foreground"));
    }

    #[test]
    fn test_default_theme_values() {
        let theme = Theme::default();
        assert_eq!(theme.colors.background, "#1a1a1a");
        assert_eq!(theme.components.list_item_height, 52);
        assert_eq!(theme.spacing.md, 12);
    }
}
