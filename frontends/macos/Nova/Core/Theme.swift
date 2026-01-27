//
//  Theme.swift
//  Nova
//
//  Unified theme system that loads design tokens from the Rust core.
//  This ensures consistent styling across macOS and Linux UIs.
//

import Cocoa

// MARK: - Theme Models

/// Color definitions from theme.toml
struct ThemeColors: Codable {
    let background: String
    let backgroundSecondary: String
    let backgroundElevated: String
    let foreground: String
    let foregroundSecondary: String
    let foregroundTertiary: String
    let accent: String
    let accentHover: String
    let border: String
    let error: String
    let success: String
    let warning: String
    let selection: String
    let selectionBackground: String
}

/// Spacing values in pixels
struct ThemeSpacing: Codable {
    let xs: Int
    let sm: Int
    let md: Int
    let lg: Int
    let xl: Int
    let xxl: Int
}

/// Typography definitions
struct ThemeTypography: Codable {
    let fontFamily: String
    let fontSizeXs: Int
    let fontSizeSm: Int
    let fontSizeMd: Int
    let fontSizeLg: Int
    let fontSizeXl: Int
    let fontSizeXxl: Int
    let fontWeightNormal: Int
    let fontWeightMedium: Int
    let fontWeightSemibold: Int
    let fontWeightBold: Int
}

/// Corner radii in pixels
struct ThemeRadii: Codable {
    let xs: Int
    let sm: Int
    let md: Int
    let lg: Int
    let xl: Int
}

/// Shadow definitions
struct ThemeShadows: Codable {
    let panel: String
    let dropdown: String
    let subtle: String
}

/// Component-specific styling values
struct ThemeComponents: Codable {
    let panelWidth: Int
    let panelHeight: Int
    let panelCornerRadius: Int
    let searchFieldHeight: Int
    let searchFieldFontSize: Int
    let searchFieldPaddingHorizontal: Int
    let searchFieldPaddingVertical: Int
    let listItemHeight: Int
    let listItemPaddingHorizontal: Int
    let listItemPaddingVertical: Int
    let listItemIconSize: Int
    let listItemCornerRadius: Int
    let listItemSpacing: Int
    let extensionItemHeight: Int
    let extensionIconSize: Int
    let iconSizeXs: Int
    let iconSizeSm: Int
    let iconSizeMd: Int
    let iconSizeLg: Int
    let iconSizeXl: Int
    let iconSizeXxl: Int
    let dividerThickness: Int
    let dividerMargin: Int
}

/// Animation timing values
struct ThemeAnimation: Codable {
    let durationFast: Int
    let durationNormal: Int
    let durationSlow: Int
    let easingDefault: String
    let easingSpring: String
}

/// Complete theme definition
struct ThemeData: Codable {
    let colors: ThemeColors
    let spacing: ThemeSpacing
    let typography: ThemeTypography
    let radii: ThemeRadii
    let shadows: ThemeShadows
    let components: ThemeComponents
    let animation: ThemeAnimation
}

// MARK: - Theme Singleton

/// Global theme accessor that loads values from the Rust core.
final class Theme {
    /// Shared theme instance
    static let shared = Theme()

    /// The loaded theme data
    private(set) var data: ThemeData

    private init() {
        // Load theme from FFI
        if let themePtr = nova_core_get_theme() {
            defer { nova_string_free(themePtr) }
            let jsonString = String(cString: themePtr)

            if let jsonData = jsonString.data(using: .utf8) {
                do {
                    self.data = try JSONDecoder().decode(ThemeData.self, from: jsonData)
                    return
                } catch {
                    print("[Nova] Failed to decode theme: \(error)")
                }
            }
        }

        // Fallback to hardcoded defaults
        self.data = Theme.defaultTheme()
    }

    /// Reload theme from FFI (call if theme file changes)
    func reload() {
        if let themePtr = nova_core_get_theme() {
            defer { nova_string_free(themePtr) }
            let jsonString = String(cString: themePtr)

            if let jsonData = jsonString.data(using: .utf8),
               let newData = try? JSONDecoder().decode(ThemeData.self, from: jsonData) {
                self.data = newData
            }
        }
    }

    // MARK: - Default Theme

    private static func defaultTheme() -> ThemeData {
        ThemeData(
            colors: ThemeColors(
                background: "#1a1a1a",
                backgroundSecondary: "#252525",
                backgroundElevated: "#2d2d2d",
                foreground: "#ffffff",
                foregroundSecondary: "#a0a0a0",
                foregroundTertiary: "#666666",
                accent: "#007AFF",
                accentHover: "#0056CC",
                border: "#3d3d3d",
                error: "#FF453A",
                success: "#32D74B",
                warning: "#FFD60A",
                selection: "#007AFF",
                selectionBackground: "#1e3a5f"
            ),
            spacing: ThemeSpacing(xs: 4, sm: 8, md: 12, lg: 16, xl: 24, xxl: 32),
            typography: ThemeTypography(
                fontFamily: "system-ui",
                fontSizeXs: 10,
                fontSizeSm: 11,
                fontSizeMd: 13,
                fontSizeLg: 16,
                fontSizeXl: 20,
                fontSizeXxl: 24,
                fontWeightNormal: 400,
                fontWeightMedium: 500,
                fontWeightSemibold: 600,
                fontWeightBold: 700
            ),
            radii: ThemeRadii(xs: 2, sm: 4, md: 8, lg: 12, xl: 16),
            shadows: ThemeShadows(
                panel: "0 8px 32px rgba(0,0,0,0.5)",
                dropdown: "0 4px 16px rgba(0,0,0,0.4)",
                subtle: "0 2px 8px rgba(0,0,0,0.2)"
            ),
            components: ThemeComponents(
                panelWidth: 620,
                panelHeight: 400,
                panelCornerRadius: 12,
                searchFieldHeight: 48,
                searchFieldFontSize: 24,
                searchFieldPaddingHorizontal: 16,
                searchFieldPaddingVertical: 12,
                listItemHeight: 52,
                listItemPaddingHorizontal: 10,
                listItemPaddingVertical: 8,
                listItemIconSize: 36,
                listItemCornerRadius: 8,
                listItemSpacing: 2,
                extensionItemHeight: 44,
                extensionIconSize: 28,
                iconSizeXs: 12,
                iconSizeSm: 16,
                iconSizeMd: 20,
                iconSizeLg: 24,
                iconSizeXl: 32,
                iconSizeXxl: 36,
                dividerThickness: 1,
                dividerMargin: 8
            ),
            animation: ThemeAnimation(
                durationFast: 100,
                durationNormal: 200,
                durationSlow: 300,
                easingDefault: "ease-out",
                easingSpring: "ease-in-out"
            )
        )
    }
}

// MARK: - NSColor Extensions

extension NSColor {
    /// Create NSColor from hex string (e.g., "#1a1a1a" or "1a1a1a")
    convenience init?(hex: String) {
        var hexString = hex.trimmingCharacters(in: .whitespacesAndNewlines)
        if hexString.hasPrefix("#") {
            hexString.removeFirst()
        }

        guard hexString.count == 6,
              let rgb = UInt64(hexString, radix: 16) else {
            return nil
        }

        let red = CGFloat((rgb >> 16) & 0xFF) / 255.0
        let green = CGFloat((rgb >> 8) & 0xFF) / 255.0
        let blue = CGFloat(rgb & 0xFF) / 255.0

        self.init(red: red, green: green, blue: blue, alpha: 1.0)
    }
}

// MARK: - Theme Color Accessors

extension Theme {
    // Convenience accessors for colors as NSColor

    var backgroundColor: NSColor {
        NSColor(hex: data.colors.background) ?? .black
    }

    var backgroundSecondaryColor: NSColor {
        NSColor(hex: data.colors.backgroundSecondary) ?? .darkGray
    }

    var backgroundElevatedColor: NSColor {
        NSColor(hex: data.colors.backgroundElevated) ?? .darkGray
    }

    var foregroundColor: NSColor {
        NSColor(hex: data.colors.foreground) ?? .white
    }

    var foregroundSecondaryColor: NSColor {
        NSColor(hex: data.colors.foregroundSecondary) ?? .gray
    }

    var foregroundTertiaryColor: NSColor {
        NSColor(hex: data.colors.foregroundTertiary) ?? .darkGray
    }

    var accentColor: NSColor {
        NSColor(hex: data.colors.accent) ?? .controlAccentColor
    }

    var accentHoverColor: NSColor {
        NSColor(hex: data.colors.accentHover) ?? .controlAccentColor
    }

    var borderColor: NSColor {
        NSColor(hex: data.colors.border) ?? .separatorColor
    }

    var errorColor: NSColor {
        NSColor(hex: data.colors.error) ?? .systemRed
    }

    var successColor: NSColor {
        NSColor(hex: data.colors.success) ?? .systemGreen
    }

    var warningColor: NSColor {
        NSColor(hex: data.colors.warning) ?? .systemYellow
    }

    var selectionColor: NSColor {
        NSColor(hex: data.colors.selection) ?? .controlAccentColor
    }

    var selectionBackgroundColor: NSColor {
        NSColor(hex: data.colors.selectionBackground) ?? accentColor.withAlphaComponent(0.15)
    }
}

// MARK: - Theme CGFloat Accessors

extension Theme {
    // Spacing
    var spacingXs: CGFloat { CGFloat(data.spacing.xs) }
    var spacingSm: CGFloat { CGFloat(data.spacing.sm) }
    var spacingMd: CGFloat { CGFloat(data.spacing.md) }
    var spacingLg: CGFloat { CGFloat(data.spacing.lg) }
    var spacingXl: CGFloat { CGFloat(data.spacing.xl) }
    var spacingXxl: CGFloat { CGFloat(data.spacing.xxl) }

    // Radii
    var radiusXs: CGFloat { CGFloat(data.radii.xs) }
    var radiusSm: CGFloat { CGFloat(data.radii.sm) }
    var radiusMd: CGFloat { CGFloat(data.radii.md) }
    var radiusLg: CGFloat { CGFloat(data.radii.lg) }
    var radiusXl: CGFloat { CGFloat(data.radii.xl) }

    // Components
    var panelWidth: CGFloat { CGFloat(data.components.panelWidth) }
    var panelHeight: CGFloat { CGFloat(data.components.panelHeight) }
    var panelCornerRadius: CGFloat { CGFloat(data.components.panelCornerRadius) }

    var searchFieldHeight: CGFloat { CGFloat(data.components.searchFieldHeight) }
    var searchFieldFontSize: CGFloat { CGFloat(data.components.searchFieldFontSize) }
    var searchFieldPaddingH: CGFloat { CGFloat(data.components.searchFieldPaddingHorizontal) }
    var searchFieldPaddingV: CGFloat { CGFloat(data.components.searchFieldPaddingVertical) }

    var listItemHeight: CGFloat { CGFloat(data.components.listItemHeight) }
    var listItemPaddingH: CGFloat { CGFloat(data.components.listItemPaddingHorizontal) }
    var listItemPaddingV: CGFloat { CGFloat(data.components.listItemPaddingVertical) }
    var listItemIconSize: CGFloat { CGFloat(data.components.listItemIconSize) }
    var listItemCornerRadius: CGFloat { CGFloat(data.components.listItemCornerRadius) }
    var listItemSpacing: CGFloat { CGFloat(data.components.listItemSpacing) }

    var extensionItemHeight: CGFloat { CGFloat(data.components.extensionItemHeight) }
    var extensionIconSize: CGFloat { CGFloat(data.components.extensionIconSize) }

    var iconSizeXs: CGFloat { CGFloat(data.components.iconSizeXs) }
    var iconSizeSm: CGFloat { CGFloat(data.components.iconSizeSm) }
    var iconSizeMd: CGFloat { CGFloat(data.components.iconSizeMd) }
    var iconSizeLg: CGFloat { CGFloat(data.components.iconSizeLg) }
    var iconSizeXl: CGFloat { CGFloat(data.components.iconSizeXl) }
    var iconSizeXxl: CGFloat { CGFloat(data.components.iconSizeXxl) }

    var dividerThickness: CGFloat { CGFloat(data.components.dividerThickness) }
    var dividerMargin: CGFloat { CGFloat(data.components.dividerMargin) }
}

// MARK: - Theme Font Accessors

extension Theme {
    /// Get a system font with the specified size from theme
    func font(size: ThemeFontSize, weight: NSFont.Weight = .regular) -> NSFont {
        let fontSize: CGFloat
        switch size {
        case .xs: fontSize = CGFloat(data.typography.fontSizeXs)
        case .sm: fontSize = CGFloat(data.typography.fontSizeSm)
        case .md: fontSize = CGFloat(data.typography.fontSizeMd)
        case .lg: fontSize = CGFloat(data.typography.fontSizeLg)
        case .xl: fontSize = CGFloat(data.typography.fontSizeXl)
        case .xxl: fontSize = CGFloat(data.typography.fontSizeXxl)
        }
        return .systemFont(ofSize: fontSize, weight: weight)
    }

    /// Map theme weight value to NSFont.Weight
    func fontWeight(_ themeWeight: Int) -> NSFont.Weight {
        switch themeWeight {
        case 0..<450: return .regular
        case 450..<550: return .medium
        case 550..<650: return .semibold
        default: return .bold
        }
    }
}

/// Theme font size identifiers
enum ThemeFontSize {
    case xs, sm, md, lg, xl, xxl
}
