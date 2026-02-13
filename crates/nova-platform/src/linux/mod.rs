mod app_discovery;
mod system_commands;

use crate::shared::{ArboardClipboard, AutoLaunchAutostart, NotifyRustNotifications, OpenCrateOpener};
use crate::traits::Platform;

pub use app_discovery::LinuxAppDiscovery;
pub use system_commands::LinuxSystemCommands;

pub fn create_platform() -> Platform {
    Platform {
        apps: Box::new(LinuxAppDiscovery::new()),
        system: Box::new(LinuxSystemCommands),
        clipboard: Box::new(ArboardClipboard::new().expect("Failed to init clipboard")),
        notifications: Box::new(NotifyRustNotifications),
        opener: Box::new(OpenCrateOpener),
        autostart: Box::new(AutoLaunchAutostart::new().expect("Failed to init autostart")),
    }
}
