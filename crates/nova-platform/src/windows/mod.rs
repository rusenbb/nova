use nova_core::{NovaError, NovaResult, PlatformAppEntry, SystemCommand};

use crate::shared::{ArboardClipboard, AutoLaunchAutostart, NotifyRustNotifications, OpenCrateOpener};
use crate::traits::{AppDiscovery, Platform, SystemCommands};

struct WindowsAppDiscovery;

impl AppDiscovery for WindowsAppDiscovery {
    fn discover_apps(&self) -> Vec<PlatformAppEntry> {
        Vec::new() // TODO: Enumerate Start Menu .lnk files
    }

    fn launch_app(&self, _app: &PlatformAppEntry) -> NovaResult<()> {
        Err(NovaError::Platform("Windows app launching not yet implemented".to_string()))
    }
}

struct WindowsSystemCommands;

impl SystemCommands for WindowsSystemCommands {
    fn execute(&self, _command: SystemCommand) -> NovaResult<()> {
        Err(NovaError::Platform("Windows system commands not yet implemented".to_string()))
    }
}

pub fn create_platform() -> Platform {
    Platform {
        apps: Box::new(WindowsAppDiscovery),
        system: Box::new(WindowsSystemCommands),
        clipboard: Box::new(ArboardClipboard::new().expect("Failed to init clipboard")),
        notifications: Box::new(NotifyRustNotifications),
        opener: Box::new(OpenCrateOpener),
        autostart: Box::new(AutoLaunchAutostart::new().expect("Failed to init autostart")),
    }
}
