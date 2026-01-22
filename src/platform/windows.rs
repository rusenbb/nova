//! Windows platform implementation (stub).
//!
//! TODO: Implement Platform trait for Windows using:
//! - Start Menu and Registry for app discovery
//! - Win32 clipboard API (via arboard crate)
//! - ShellExecute for URLs and files
//! - Toast notifications
//! - ExitWindowsEx/shutdown.exe for system commands

use super::{AppEntry, CommandResult, NotifyResult, Platform, SystemCommand};
use std::path::PathBuf;

/// Windows platform implementation.
pub struct WindowsPlatform;

impl WindowsPlatform {
    /// Create a new Windows platform instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for WindowsPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl Platform for WindowsPlatform {
    fn discover_apps(&self) -> Vec<AppEntry> {
        // TODO: Scan Start Menu folders
        // TODO: Query Registry App Paths
        // TODO: Handle UWP apps
        eprintln!("[Nova] Windows app discovery not yet implemented");
        Vec::new()
    }

    fn clipboard_read(&self) -> Option<String> {
        // TODO: Use arboard crate
        eprintln!("[Nova] Windows clipboard read not yet implemented");
        None
    }

    fn clipboard_write(&self, _content: &str) -> Result<(), String> {
        // TODO: Use arboard crate
        Err("Windows clipboard write not yet implemented".to_string())
    }

    fn open_url(&self, url: &str) -> Result<(), String> {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
        Ok(())
    }

    fn open_file(&self, path: &str) -> Result<(), String> {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", path])
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
        Ok(())
    }

    fn show_notification(&self, _title: &str, _body: &str) -> NotifyResult {
        // TODO: Use windows-rs toast notifications
        Err("Windows notifications not yet implemented".to_string())
    }

    fn system_command(&self, command: SystemCommand) -> CommandResult {
        let (cmd, args): (&str, Vec<&str>) = match command {
            SystemCommand::Lock => ("rundll32.exe", vec!["user32.dll,LockWorkStation"]),
            SystemCommand::Sleep => (
                "rundll32.exe",
                vec!["powrprof.dll,SetSuspendState", "0", "1", "0"],
            ),
            SystemCommand::Logout => ("shutdown", vec!["/l"]),
            SystemCommand::Restart => ("shutdown", vec!["/r", "/t", "0"]),
            SystemCommand::Shutdown => ("shutdown", vec!["/s", "/t", "0"]),
        };

        std::process::Command::new(cmd)
            .args(&args)
            .spawn()
            .map_err(|e| format!("Failed to execute {}: {}", cmd, e))?;

        Ok(())
    }

    fn config_dir(&self) -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| {
                std::env::var("APPDATA")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Default\\AppData\\Roaming"))
            })
            .join("nova")
    }

    fn data_dir(&self) -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| {
                std::env::var("LOCALAPPDATA")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Default\\AppData\\Local"))
            })
            .join("nova")
    }

    fn runtime_dir(&self) -> PathBuf {
        std::env::var("TEMP")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\Windows\\Temp"))
    }

    fn launch_app(&self, app: &AppEntry) -> Result<(), String> {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &app.exec])
            .spawn()
            .map_err(|e| format!("Failed to launch {}: {}", app.name, e))?;
        Ok(())
    }

    fn run_shell_command(&self, command: &str) -> Result<(), String> {
        std::process::Command::new("cmd")
            .args(["/C", command])
            .spawn()
            .map_err(|e| format!("Failed to run command: {}", e))?;
        Ok(())
    }
}
