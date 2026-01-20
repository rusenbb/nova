//! Result execution module - determines what action to take for each SearchResult

use std::path::PathBuf;

use crate::services::{LoadedCommand, ScriptOutputMode};

/// The action to perform when a result is executed
#[derive(Debug, Clone)]
pub enum ExecutionAction {
    /// Launch an application by its .desktop exec command
    LaunchApp { exec: String, name: String },

    /// Open Nova settings
    OpenSettings,

    /// Quit the application
    Quit,

    /// Execute a system command (lock, sleep, logout, restart, shutdown)
    SystemCommand { command: SystemCommand },

    /// Run a shell command
    RunShellCommand { command: String },

    /// Open a URL in the default browser
    OpenUrl { url: String },

    /// Execute a script
    RunScript {
        path: PathBuf,
        argument: Option<String>,
        output_mode: ScriptOutputMode,
    },

    /// Execute an extension command
    RunExtensionCommand {
        command: LoadedCommand,
        argument: Option<String>,
    },

    /// Copy text to clipboard with notification
    CopyToClipboard {
        content: String,
        notification: String,
    },

    /// Open a file or directory with xdg-open
    OpenFile { path: String },

    /// No action needed (e.g., quicklink waiting for query input)
    NeedsInput,
}

/// System commands that can be executed
#[derive(Debug, Clone, Copy)]
pub enum SystemCommand {
    Lock,
    Sleep,
    Logout,
    Restart,
    Shutdown,
}

impl SystemCommand {
    /// Get the command and arguments to execute
    pub fn command_args(&self) -> (&'static str, Vec<&'static str>) {
        match self {
            SystemCommand::Lock => ("loginctl", vec!["lock-session"]),
            SystemCommand::Sleep => ("systemctl", vec!["suspend"]),
            SystemCommand::Logout => ("gnome-session-quit", vec!["--logout", "--no-prompt"]),
            SystemCommand::Restart => ("systemctl", vec!["reboot"]),
            SystemCommand::Shutdown => ("systemctl", vec!["poweroff"]),
        }
    }

    /// Fallback command for logout if primary fails
    pub fn logout_fallback() -> (&'static str, Vec<String>) {
        let user = std::env::var("USER").unwrap_or_default();
        ("loginctl", vec!["terminate-user".to_string(), user])
    }
}
