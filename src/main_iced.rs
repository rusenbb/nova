//! Nova launcher - iced UI entry point.
//!
//! This is the cross-platform entry point using the iced UI framework.
//! Build with: cargo build --no-default-features --features iced-ui

use iced::{window, Size};
use nova::platform;
use nova::ui::{acquire_or_signal, single_instance, HotkeyConfig, InstanceResult, NovaApp};

fn main() -> iced::Result {
    println!("[Nova] Starting with iced UI on {}", platform::name());

    // Check for existing instance
    match acquire_or_signal() {
        InstanceResult::Secondary => {
            println!("[Nova] Another instance is already running. Signaled it to show.");
            return Ok(());
        }
        InstanceResult::Primary(ipc_rx) => {
            println!("[Nova] Primary instance starting...");
            // The IPC receiver will be passed to the app for handling show requests
            run_app(ipc_rx)
        }
    }
}

fn run_app(
    ipc_rx: std::sync::mpsc::Receiver<nova::ui::InstanceMessage>,
) -> iced::Result {
    // Initialize global hotkey (Alt+Space by default)
    let hotkey_config = HotkeyConfig::default();
    let hotkey_rx = nova::ui::hotkey::init_hotkey_manager(&hotkey_config);

    // Create app factory that captures the receivers
    let app_factory = move || NovaApp::new_with_channels(hotkey_rx, Some(ipc_rx));

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

    // Register cleanup handler
    ctrlc_handler();

    iced::application("Nova", NovaApp::update, NovaApp::view)
        .subscription(NovaApp::subscription)
        .window(window_settings)
        .run_with(app_factory)
}

fn ctrlc_handler() {
    let _ = ctrlc::set_handler(move || {
        println!("[Nova] Shutting down...");
        single_instance::cleanup();
        std::process::exit(0);
    });
}
