//! Nova launcher - iced UI entry point.
//!
//! This is the cross-platform entry point using the iced UI framework.
//! Build with: cargo build --no-default-features --features iced-ui

use iced::{window, Size};
use nova::platform;
use nova::ui::NovaApp;

fn main() -> iced::Result {
    println!("[Nova] Starting with iced UI on {}", platform::name());

    // Window settings for a launcher-style window
    let window_settings = window::Settings {
        size: Size::new(600.0, 400.0),
        position: window::Position::Centered,
        decorations: false, // Borderless
        transparent: true,
        level: window::Level::AlwaysOnTop,
        resizable: false,
        exit_on_close_request: false,
        ..Default::default()
    };

    iced::application("Nova", NovaApp::update, NovaApp::view)
        .subscription(NovaApp::subscription)
        .window(window_settings)
        .run_with(NovaApp::new)
}
