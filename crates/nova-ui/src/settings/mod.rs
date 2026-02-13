use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};

use nova_core::{available_themes, Config};

use crate::app::Message;

/// Render the settings view
pub fn view(config: &Config) -> Element<'_, Message> {
    let title = text("Settings")
        .size(24);

    let close_button = button(text("Back"))
        .on_press(Message::SettingsToggle);

    let header = row![title, iced::widget::horizontal_space(), close_button]
        .padding(12)
        .align_y(iced::Alignment::Center);

    // Theme selector
    let _themes: Vec<String> = available_themes().iter().map(|t| t.to_string()).collect();
    let theme_label = text("Theme:").size(14);
    let theme_row = row![theme_label].spacing(8);

    // Opacity slider
    let opacity_label = text(format!("Opacity: {:.0}%", config.appearance.opacity * 100.0))
        .size(14);

    // Max results
    let max_results_label = text(format!("Max results: {}", config.behavior.max_results))
        .size(14);

    // Hotkey
    let hotkey_label = text("Hotkey:").size(14);
    let hotkey_value = text(&config.general.hotkey).size(14);
    let hotkey_row = row![hotkey_label, hotkey_value].spacing(8);

    let content = column![
        header,
        iced::widget::horizontal_rule(1),
        column![
            text("Appearance").size(18),
            theme_row,
            opacity_label,
        ]
        .spacing(8)
        .padding(12),
        column![
            text("Behavior").size(18),
            max_results_label,
            hotkey_row,
        ]
        .spacing(8)
        .padding(12),
    ]
    .spacing(8);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(8)
        .into()
}
