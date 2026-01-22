//! Platform abstraction layer for cross-platform support.
//!
//! This module defines the `Platform` trait that abstracts all OS-specific
//! operations, allowing the core engine and UI to remain platform-agnostic.

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

use std::path::PathBuf;

/// Represents an installed application discovered on the system.
#[derive(Debug, Clone)]
pub struct AppEntry {
    /// Unique identifier (e.g., filename without extension on Linux)
    pub id: String,
    /// Display name of the application
    pub name: String,
    /// Command to execute the application
    pub exec: String,
    /// Path to the application icon (platform-specific format)
    pub icon: Option<String>,
    /// Description or comment about the application
    pub description: Option<String>,
    /// Keywords for search matching
    pub keywords: Vec<String>,
}

/// System commands that can be executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemCommand {
    /// Lock the screen/session
    Lock,
    /// Put the computer to sleep/suspend
    Sleep,
    /// Log out of the current session
    Logout,
    /// Restart the computer
    Restart,
    /// Shut down the computer
    Shutdown,
}

/// Result of a notification being shown.
pub type NotifyResult = Result<(), String>;

/// Result of executing a system command.
pub type CommandResult = Result<(), String>;

/// Platform-specific operations trait.
///
/// Implementations of this trait provide OS-specific functionality for:
/// - Application discovery
/// - Clipboard operations
/// - Opening URLs and files
/// - Desktop notifications
/// - System commands (lock, sleep, shutdown, etc.)
/// - Configuration and data directory paths
pub trait Platform: Send + Sync {
    /// Discover installed applications on the system.
    ///
    /// Returns a list of applications that can be launched.
    /// The implementation should scan platform-specific locations:
    /// - Linux: XDG .desktop files
    /// - macOS: /Applications, ~/Applications
    /// - Windows: Start Menu, Registry App Paths
    fn discover_apps(&self) -> Vec<AppEntry>;

    /// Read the current clipboard content as text.
    ///
    /// Returns `None` if the clipboard is empty or contains non-text data.
    fn clipboard_read(&self) -> Option<String>;

    /// Write text to the clipboard.
    fn clipboard_write(&self, content: &str) -> Result<(), String>;

    /// Open a URL in the default browser.
    fn open_url(&self, url: &str) -> Result<(), String>;

    /// Open a file or directory with the default application.
    fn open_file(&self, path: &str) -> Result<(), String>;

    /// Show a desktop notification.
    fn show_notification(&self, title: &str, body: &str) -> NotifyResult;

    /// Execute a system command (lock, sleep, logout, restart, shutdown).
    fn system_command(&self, command: SystemCommand) -> CommandResult;

    /// Get the platform-specific configuration directory.
    ///
    /// - Linux: `~/.config/nova/`
    /// - macOS: `~/Library/Application Support/nova/`
    /// - Windows: `%APPDATA%\nova\`
    fn config_dir(&self) -> PathBuf;

    /// Get the platform-specific data directory.
    ///
    /// - Linux: `~/.local/share/nova/`
    /// - macOS: `~/Library/Application Support/nova/`
    /// - Windows: `%LOCALAPPDATA%\nova\`
    fn data_dir(&self) -> PathBuf;

    /// Get the platform-specific runtime directory (for sockets, etc.).
    ///
    /// - Linux: `$XDG_RUNTIME_DIR` or `/tmp`
    /// - macOS: `$TMPDIR` or `/tmp`
    /// - Windows: `%TEMP%`
    fn runtime_dir(&self) -> PathBuf;

    /// Launch an application by its exec command.
    ///
    /// The `exec` string may contain field codes (like `%f`, `%u`) that
    /// should be stripped or handled appropriately.
    fn launch_app(&self, app: &AppEntry) -> Result<(), String>;

    /// Run an arbitrary shell command.
    fn run_shell_command(&self, command: &str) -> Result<(), String>;
}

/// Get the platform implementation for the current OS.
pub fn current() -> Box<dyn Platform> {
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxPlatform::new())
    }

    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOSPlatform::new())
    }

    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsPlatform::new())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        compile_error!("Unsupported platform")
    }
}

/// Platform name as a string (for logging/display).
pub fn name() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        "Linux"
    }

    #[cfg(target_os = "macos")]
    {
        "macOS"
    }

    #[cfg(target_os = "windows")]
    {
        "Windows"
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "Unknown"
    }
}
