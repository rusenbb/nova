//! Custom styles for Nova UI widgets.

use super::theme::NovaTheme;
use iced::widget::{container, scrollable, text_input};
use iced::{Background, Border, Color};

/// Style for the main container.
pub fn main_container(theme: &NovaTheme, opacity: f32) -> container::Style {
    container::Style {
        background: Some(Background::Color(NovaTheme::with_alpha(
            theme.background,
            opacity,
        ))),
        border: Border {
            color: theme.border,
            width: 1.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

/// Style for the search input.
pub fn search_input(theme: &NovaTheme, focused: bool) -> text_input::Style {
    let border_color = if focused {
        NovaTheme::with_alpha(theme.accent, 0.5)
    } else {
        theme.border
    };

    text_input::Style {
        background: Background::Color(NovaTheme::with_alpha(Color::WHITE, 0.05)),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 8.0.into(),
        },
        icon: theme.subtext,
        placeholder: theme.subtext,
        value: theme.text,
        selection: NovaTheme::with_alpha(theme.accent, 0.3),
    }
}

/// Style for the results list container.
pub fn results_container(_theme: &NovaTheme) -> container::Style {
    container::Style {
        background: None,
        ..Default::default()
    }
}

/// Style for a result row.
pub fn result_row(theme: &NovaTheme, selected: bool) -> container::Style {
    if selected {
        container::Style {
            background: Some(Background::Color(NovaTheme::with_alpha(theme.accent, 0.35))),
            border: Border {
                color: NovaTheme::with_alpha(theme.accent, 0.9),
                width: 0.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    } else {
        container::Style {
            background: None,
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// Style for scrollable results.
pub fn results_scrollable(theme: &NovaTheme) -> scrollable::Style {
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: NovaTheme::with_alpha(theme.subtext, 0.3),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
            },
        },
        horizontal_rail: scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: NovaTheme::with_alpha(theme.subtext, 0.3),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
            },
        },
        gap: None,
    }
}

/// Style for the command mode pill.
pub fn command_pill(theme: &NovaTheme) -> container::Style {
    container::Style {
        background: Some(Background::Color(NovaTheme::with_alpha(theme.accent, 0.25))),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
