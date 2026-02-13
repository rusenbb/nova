use nova_core::{NovaError, NovaResult};

use crate::traits::{AutostartManager, ClipboardAccess, Notifications, SystemOpen};

/// Cross-platform clipboard using arboard
pub struct ArboardClipboard {
    clipboard: std::sync::Mutex<arboard::Clipboard>,
}

impl ArboardClipboard {
    pub fn new() -> NovaResult<Self> {
        let clipboard = arboard::Clipboard::new()
            .map_err(|e| NovaError::Clipboard(e.to_string()))?;
        Ok(Self {
            clipboard: std::sync::Mutex::new(clipboard),
        })
    }
}

impl ClipboardAccess for ArboardClipboard {
    fn get_text(&self) -> NovaResult<String> {
        self.clipboard
            .lock()
            .map_err(|e| NovaError::Clipboard(e.to_string()))?
            .get_text()
            .map_err(|e| NovaError::Clipboard(e.to_string()))
    }

    fn set_text(&self, content: &str) -> NovaResult<()> {
        self.clipboard
            .lock()
            .map_err(|e| NovaError::Clipboard(e.to_string()))?
            .set_text(content)
            .map_err(|e| NovaError::Clipboard(e.to_string()))
    }
}

/// Cross-platform notifications using notify-rust
pub struct NotifyRustNotifications;

impl Notifications for NotifyRustNotifications {
    fn show(&self, title: &str, body: &str) -> NovaResult<()> {
        notify_rust::Notification::new()
            .summary(title)
            .body(body)
            .show()
            .map_err(|e| NovaError::Platform(e.to_string()))?;
        Ok(())
    }
}

/// Cross-platform file/URL opening using the `open` crate
pub struct OpenCrateOpener;

impl SystemOpen for OpenCrateOpener {
    fn open_url(&self, url: &str) -> NovaResult<()> {
        open::that(url).map_err(|e| NovaError::Platform(e.to_string()))
    }

    fn open_file(&self, path: &str) -> NovaResult<()> {
        open::that(path).map_err(|e| NovaError::Platform(e.to_string()))
    }
}

/// Cross-platform autostart using auto-launch
pub struct AutoLaunchAutostart {
    auto_launch: auto_launch::AutoLaunch,
}

impl AutoLaunchAutostart {
    pub fn new() -> NovaResult<Self> {
        let exe_path = std::env::current_exe()
            .map_err(|e| NovaError::Platform(e.to_string()))?
            .to_string_lossy()
            .to_string();

        let auto_launch = auto_launch::AutoLaunchBuilder::new()
            .set_app_name("Nova")
            .set_app_path(&exe_path)
            .build()
            .map_err(|e| NovaError::Platform(e.to_string()))?;

        Ok(Self { auto_launch })
    }
}

impl AutostartManager for AutoLaunchAutostart {
    fn set_enabled(&self, enabled: bool) -> NovaResult<()> {
        if enabled {
            self.auto_launch
                .enable()
                .map_err(|e| NovaError::Platform(e.to_string()))
        } else {
            self.auto_launch
                .disable()
                .map_err(|e| NovaError::Platform(e.to_string()))
        }
    }

    fn is_enabled(&self) -> bool {
        self.auto_launch.is_enabled().unwrap_or(false)
    }
}
