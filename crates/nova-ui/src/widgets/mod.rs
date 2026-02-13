use iced::widget::{column, container, mouse_area, text};
use iced::{Element, Theme};

use nova_core::{SearchResult, ThemePalette};

use crate::app::Message;

/// Render a single result row
pub fn result_row<'a>(
    result: &SearchResult,
    is_selected: bool,
    palette: &ThemePalette,
    index: usize,
) -> Element<'a, Message> {
    let name_text = text(result.name().to_string())
        .size(15)
        .color(iced::Color::from_rgb8(
            palette.text.0,
            palette.text.1,
            palette.text.2,
        ));

    let content = if let Some(desc) = result.description() {
        let desc_text = text(desc.to_string())
            .size(12)
            .color(iced::Color::from_rgb8(
                palette.subtext.0,
                palette.subtext.1,
                palette.subtext.2,
            ));

        column![name_text, desc_text].spacing(2).into()
    } else {
        Element::from(name_text)
    };

    let accent = palette.accent;

    let row_container = container(content)
        .width(iced::Length::Fill)
        .padding([6, 12])
        .style(move |_theme: &Theme| {
            if is_selected {
                container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba8(
                        accent.0, accent.1, accent.2, 0.35,
                    ))),
                    border: iced::Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            } else {
                container::Style::default()
            }
        });

    mouse_area(row_container)
        .on_press(Message::SelectIndex(index))
        .into()
}
