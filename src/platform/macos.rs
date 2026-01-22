//! macOS platform implementation (stub).
//!
//! TODO: Implement Platform trait for macOS using:
//! - /Applications scanning for app discovery
//! - NSPasteboard for clipboard (via arboard crate)
//! - `open` command for URLs and files
//! - NSUserNotificationCenter for notifications
//! - osascript/pmset for system commands

use super::{AppEntry, CommandResult, NotifyResult, Platform, SystemCommand};
use std::path::PathBuf;

/// macOS platform implementation.
pub struct MacOSPlatform;

impl MacOSPlatform {
    /// Create a new macOS platform instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for MacOSPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl Platform for MacOSPlatform {
    fn discover_apps(&self) -> Vec<AppEntry> {
        // TODO: Scan /Applications, ~/Applications
        // TODO: Use mdfind for Spotlight integration
        eprintln!("[Nova] macOS app discovery not yet implemented");
        Vec::new()
    }

    fn clipboard_read(&self) -> Option<String> {
        // TODO: Use arboard crate
        eprintln!("[Nova] macOS clipboard read not yet implemented");
        None
    }

    fn clipboard_write(&self, _content: &str) -> Result<(), String> {
        // TODO: Use arboard crate
        Err("macOS clipboard write not yet implemented".to_string())
    }

    fn open_url(&self, url: &str) -> Result<(), String> {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
        Ok(())
    }

    fn open_file(&self, path: &str) -> Result<(), String> {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
        Ok(())
    }

    fn show_notification(&self, _title: &str, _body: &str) -> NotifyResult {
        // TODO: Use notify-rust crate or osascript
        Err("macOS notifications not yet implemented".to_string())
    }

    fn system_command(&self, command: SystemCommand) -> CommandResult {
        // TODO: Implement using osascript, pmset, etc.
        let cmd_name = match command {
            SystemCommand::Lock => "lock",
            SystemCommand::Sleep => "sleep",
            SystemCommand::Logout => "logout",
            SystemCommand::Restart => "restart",
            SystemCommand::Shutdown => "shutdown",
        };
        Err(format!("macOS {} not yet implemented", cmd_name))
    }

    fn config_dir(&self) -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .map(|h| h.join("Library/Application Support"))
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
            })
            .join("nova")
    }

    fn data_dir(&self) -> PathBuf {
        // On macOS, config and data are typically in the same location
        self.config_dir()
    }

    fn runtime_dir(&self) -> PathBuf {
        std::env::var("TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
    }

    fn launch_app(&self, app: &AppEntry) -> Result<(), String> {
        // On macOS, apps are typically launched via `open -a` or direct path
        std::process::Command::new("open")
            .args(["-a", &app.exec])
            .spawn()
            .map_err(|e| format!("Failed to launch {}: {}", app.name, e))?;
        Ok(())
    }

    fn run_shell_command(&self, command: &str) -> Result<(), String> {
        std::process::Command::new("sh")
            .args(["-c", command])
            .spawn()
            .map_err(|e| format!("Failed to run command: {}", e))?;
        Ok(())
    }
}
