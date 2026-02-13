use nova_core::{NovaError, NovaResult, PlatformAppEntry, SystemCommand};

use crate::shared::{ArboardClipboard, AutoLaunchAutostart, NotifyRustNotifications, OpenCrateOpener};
use crate::traits::{AppDiscovery, Platform, SystemCommands};

struct MacOsAppDiscovery;

impl AppDiscovery for MacOsAppDiscovery {
    fn discover_apps(&self) -> Vec<PlatformAppEntry> {
        Vec::new() // TODO: Walk /Applications for .app bundles
    }

    fn launch_app(&self, _app: &PlatformAppEntry) -> NovaResult<()> {
        Err(NovaError::Platform("macOS app launching not yet implemented".to_string()))
    }
}

struct MacOsSystemCommands;

impl SystemCommands for MacOsSystemCommands {
    fn execute(&self, _command: SystemCommand) -> NovaResult<()> {
        Err(NovaError::Platform("macOS system commands not yet implemented".to_string()))
    }
}

pub fn create_platform() -> Platform {
    Platform {
        apps: Box::new(MacOsAppDiscovery),
        system: Box::new(MacOsSystemCommands),
        clipboard: Box::new(ArboardClipboard::new().expect("Failed to init clipboard")),
        notifications: Box::new(NotifyRustNotifications),
        opener: Box::new(OpenCrateOpener),
        autostart: Box::new(AutoLaunchAutostart::new().expect("Failed to init autostart")),
    }
}
