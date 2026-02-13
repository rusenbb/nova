use std::path::PathBuf;

use crate::services::{LoadedCommand, ScriptOutputMode};

/// The action to perform when a result is executed
#[derive(Debug, Clone)]
pub enum ExecutionAction {
    /// Launch an application by its exec command
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

    /// Open a file or directory
    OpenFile { path: String },

    /// No action needed (e.g., quicklink waiting for query input)
    NeedsInput,
}

/// System commands that can be executed (pure data, no OS-specific logic)
#[derive(Debug, Clone, Copy)]
pub enum SystemCommand {
    Lock,
    Sleep,
    Logout,
    Restart,
    Shutdown,
}
