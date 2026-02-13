use iced::Theme;
use nova_core::{Config, get_theme_palette, parse_hex_color};

/// Create an Iced theme from Nova's configuration
pub fn nova_theme(config: &Config) -> Theme {
    let palette = get_theme_palette(&config.appearance.theme);
    let accent = parse_hex_color(&config.appearance.accent_color);

    let custom_palette = iced::theme::Palette {
        background: iced::Color::from_rgb8(
            palette.background.0,
            palette.background.1,
            palette.background.2,
        ),
        text: iced::Color::from_rgb8(palette.text.0, palette.text.1, palette.text.2),
        primary: iced::Color::from_rgb8(accent.0, accent.1, accent.2),
        success: iced::Color::from_rgb8(166, 227, 161), // green
        danger: iced::Color::from_rgb8(243, 139, 168),  // red
    };

    Theme::custom("Nova".to_string(), custom_palette)
}
