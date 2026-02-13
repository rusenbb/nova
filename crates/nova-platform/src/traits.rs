use nova_core::{NovaResult, PlatformAppEntry, SystemCommand};

/// Discover and launch applications
pub trait AppDiscovery: Send + Sync {
    fn discover_apps(&self) -> Vec<PlatformAppEntry>;
    fn launch_app(&self, app: &PlatformAppEntry) -> NovaResult<()>;
}

/// Execute system-level commands (lock, sleep, shutdown, etc.)
pub trait SystemCommands: Send + Sync {
    fn execute(&self, command: SystemCommand) -> NovaResult<()>;
}

/// Read/write system clipboard
pub trait ClipboardAccess: Send + Sync {
    fn get_text(&self) -> NovaResult<String>;
    fn set_text(&self, content: &str) -> NovaResult<()>;
}

/// Show desktop notifications
pub trait Notifications: Send + Sync {
    fn show(&self, title: &str, body: &str) -> NovaResult<()>;
}

/// Open URLs and files with the system default handler
pub trait SystemOpen: Send + Sync {
    fn open_url(&self, url: &str) -> NovaResult<()>;
    fn open_file(&self, path: &str) -> NovaResult<()>;
}

/// Manage application autostart
pub trait AutostartManager: Send + Sync {
    fn set_enabled(&self, enabled: bool) -> NovaResult<()>;
    fn is_enabled(&self) -> bool;
}

/// Aggregate struct holding all platform-specific implementations
pub struct Platform {
    pub apps: Box<dyn AppDiscovery>,
    pub system: Box<dyn SystemCommands>,
    pub clipboard: Box<dyn ClipboardAccess>,
    pub notifications: Box<dyn Notifications>,
    pub opener: Box<dyn SystemOpen>,
    pub autostart: Box<dyn AutostartManager>,
}

impl Platform {
    /// Create a Platform instance with OS-appropriate implementations
    pub fn current() -> Self {
        #[cfg(target_os = "linux")]
        {
            crate::linux::create_platform()
        }
        #[cfg(target_os = "macos")]
        {
            crate::macos::create_platform()
        }
        #[cfg(target_os = "windows")]
        {
            crate::windows::create_platform()
        }
    }
}
