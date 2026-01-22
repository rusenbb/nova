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

// ==================== Settings UI Styles ====================

use iced::widget::button;

/// Style for the settings window container.
pub fn settings_container(theme: &NovaTheme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme.background)),
        border: Border {
            color: theme.border,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

/// Style for the settings sidebar.
pub fn sidebar_container(theme: &NovaTheme) -> container::Style {
    container::Style {
        background: Some(Background::Color(NovaTheme::with_alpha(theme.surface, 0.5))),
        border: Border {
            color: theme.border,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Style for sidebar tab buttons.
pub fn sidebar_button(
    theme: &NovaTheme,
    is_active: bool,
    _status: button::Status,
) -> button::Style {
    if is_active {
        button::Style {
            background: Some(Background::Color(NovaTheme::with_alpha(theme.accent, 0.2))),
            text_color: theme.accent,
            border: Border {
                color: theme.accent,
                width: 0.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    } else {
        button::Style {
            background: None,
            text_color: theme.subtext,
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// Style for the footer container.
pub fn footer_container(theme: &NovaTheme) -> container::Style {
    container::Style {
        background: Some(Background::Color(NovaTheme::with_alpha(theme.surface, 0.5))),
        border: Border {
            color: theme.border,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Style for primary action buttons.
pub fn primary_button(theme: &NovaTheme, status: button::Status) -> button::Style {
    let bg_alpha = match status {
        button::Status::Hovered => 1.0,
        button::Status::Pressed => 0.8,
        _ => 0.9,
    };

    button::Style {
        background: Some(Background::Color(NovaTheme::with_alpha(
            theme.accent,
            bg_alpha,
        ))),
        text_color: theme.background,
        border: Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Style for small/secondary buttons.
pub fn small_button(theme: &NovaTheme, status: button::Status) -> button::Style {
    let bg_alpha = match status {
        button::Status::Hovered => 0.15,
        button::Status::Pressed => 0.2,
        _ => 0.1,
    };

    button::Style {
        background: Some(Background::Color(NovaTheme::with_alpha(
            Color::WHITE,
            bg_alpha,
        ))),
        text_color: theme.text,
        border: Border {
            color: theme.border,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// Style for danger/delete buttons.
pub fn danger_button(_theme: &NovaTheme, status: button::Status) -> button::Style {
    let red = Color::from_rgb(0.9, 0.3, 0.3);
    let bg_alpha = match status {
        button::Status::Hovered => 0.25,
        button::Status::Pressed => 0.35,
        _ => 0.15,
    };

    button::Style {
        background: Some(Background::Color(NovaTheme::with_alpha(red, bg_alpha))),
        text_color: red,
        border: Border {
            color: NovaTheme::with_alpha(red, 0.5),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// Style for list items.
pub fn list_item(theme: &NovaTheme) -> container::Style {
    container::Style {
        background: Some(Background::Color(NovaTheme::with_alpha(Color::WHITE, 0.03))),
        border: Border {
            color: theme.border,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}
