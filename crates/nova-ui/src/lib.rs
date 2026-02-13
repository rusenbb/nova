mod app;
mod execute;
mod subscriptions;
pub mod style;
pub mod widgets;
pub mod settings;

use nova_core::{Config, PlatformAppEntry};
use nova_platform::Platform;

pub fn run(config: Config, platform: Platform, apps: Vec<PlatformAppEntry>) -> iced::Result {
    let window_width = config.appearance.window_width as f32;

    iced::application("Nova", app::Nova::update, app::Nova::view)
        .subscription(app::Nova::subscription)
        .theme(app::Nova::theme)
        .window_size(iced::Size::new(window_width, 400.0))
        .decorations(false)
        .transparent(true)
        .level(iced::window::Level::AlwaysOnTop)
        .resizable(false)
        .position(iced::window::Position::Centered)
        .run_with(move || app::Nova::new(config, platform, apps))
}

/// Try to send a toggle command to an already-running Nova instance.
/// Returns Ok(true) if the message was sent successfully.
pub fn try_send_toggle() -> Result<bool, Box<dyn std::error::Error>> {
    use std::io::Write;
    use std::os::unix::net::UnixStream;

    let socket_path = ipc_socket_path();

    match UnixStream::connect(&socket_path) {
        Ok(mut stream) => {
            stream.write_all(b"toggle")?;
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}

/// Get the IPC socket path
pub fn ipc_socket_path() -> std::path::PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| "/tmp".to_string());
    std::path::PathBuf::from(runtime_dir).join("nova.sock")
}
