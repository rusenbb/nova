//! Result execution module - determines what action to take for each SearchResult.
//!
//! This module defines ExecutionAction, which represents the action to take
//! when a search result is selected. The actual execution is delegated to
//! the Platform trait implementation.

use std::path::PathBuf;

use crate::platform::{AppEntry, Platform, SystemCommand};
use crate::services::{ExtensionManager, LoadedCommand, OutputMode, ScriptOutputMode};

/// The action to perform when a result is executed.
#[derive(Debug, Clone)]
pub enum ExecutionAction {
    /// Launch an application
    LaunchApp { app: AppEntry },

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

/// Result of executing an action.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "result", content = "message")]
pub enum ExecutionResult {
    /// Action completed successfully, hide the window
    Success,
    /// Action completed but keep the window open (e.g., clipboard copy)
    SuccessKeepOpen,
    /// Open settings window
    OpenSettings,
    /// Quit the application
    Quit,
    /// Action failed with an error message
    Error(String),
    /// Waiting for more input from user
    NeedsInput,
}

/// Execute an action using the platform trait.
pub fn execute(
    action: &ExecutionAction,
    platform: &dyn Platform,
    extension_manager: Option<&ExtensionManager>,
) -> ExecutionResult {
    match action {
        ExecutionAction::LaunchApp { app } => match platform.launch_app(app) {
            Ok(()) => ExecutionResult::Success,
            Err(e) => ExecutionResult::Error(e),
        },

        ExecutionAction::OpenSettings => ExecutionResult::OpenSettings,

        ExecutionAction::Quit => ExecutionResult::Quit,

        ExecutionAction::SystemCommand { command } => match platform.system_command(*command) {
            Ok(()) => ExecutionResult::Success,
            Err(e) => ExecutionResult::Error(e),
        },

        ExecutionAction::RunShellCommand { command } => match platform.run_shell_command(command) {
            Ok(()) => ExecutionResult::Success,
            Err(e) => ExecutionResult::Error(e),
        },

        ExecutionAction::OpenUrl { url } => match platform.open_url(url) {
            Ok(()) => ExecutionResult::Success,
            Err(e) => ExecutionResult::Error(e),
        },

        ExecutionAction::RunScript {
            path,
            argument,
            output_mode,
        } => execute_script(path, argument.as_ref(), output_mode, platform),

        ExecutionAction::RunExtensionCommand { command, argument } => {
            if let Some(ext_manager) = extension_manager {
                execute_extension_command(ext_manager, command, argument.as_ref(), platform)
            } else {
                ExecutionResult::Error("Extension manager not available".to_string())
            }
        }

        ExecutionAction::CopyToClipboard {
            content,
            notification,
        } => {
            if let Err(e) = platform.clipboard_write(content) {
                return ExecutionResult::Error(e);
            }
            let _ = platform.show_notification("Copied", notification);
            ExecutionResult::Success
        }

        ExecutionAction::OpenFile { path } => {
            // Expand ~ to home directory
            let full_path = if path.starts_with("~/") {
                dirs::home_dir()
                    .map(|h| format!("{}{}", h.display(), &path[1..]))
                    .unwrap_or_else(|| path.clone())
            } else {
                path.clone()
            };

            match platform.open_file(&full_path) {
                Ok(()) => ExecutionResult::Success,
                Err(e) => ExecutionResult::Error(e),
            }
        }

        ExecutionAction::NeedsInput => ExecutionResult::NeedsInput,
    }
}

/// Execute a script with the given arguments and output mode.
fn execute_script(
    path: &PathBuf,
    argument: Option<&String>,
    output_mode: &ScriptOutputMode,
    platform: &dyn Platform,
) -> ExecutionResult {
    use std::process::Command;

    let mut cmd = Command::new(path);
    if let Some(arg) = argument {
        cmd.arg(arg);
    }

    match output_mode {
        ScriptOutputMode::Silent => match cmd.spawn() {
            Ok(_) => ExecutionResult::Success,
            Err(e) => ExecutionResult::Error(format!("Failed to execute script: {}", e)),
        },
        ScriptOutputMode::Notification => match cmd.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !stdout.is_empty() {
                    let _ = platform.show_notification("Nova Script", &stdout);
                }
                ExecutionResult::Success
            }
            Err(e) => ExecutionResult::Error(format!("Failed to execute script: {}", e)),
        },
        ScriptOutputMode::Clipboard => match cmd.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !stdout.is_empty() {
                    if let Err(e) = platform.clipboard_write(&stdout) {
                        return ExecutionResult::Error(e);
                    }
                    let _ = platform.show_notification("Copied to clipboard", &stdout);
                }
                ExecutionResult::Success
            }
            Err(e) => ExecutionResult::Error(format!("Failed to execute script: {}", e)),
        },
        ScriptOutputMode::Inline => {
            // For now, treat inline same as notification
            match cmd.output() {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !stdout.is_empty() {
                        let _ = platform.show_notification("Nova Script", &stdout);
                    }
                    ExecutionResult::Success
                }
                Err(e) => ExecutionResult::Error(format!("Failed to execute script: {}", e)),
            }
        }
    }
}

/// Execute an extension command and handle its output.
fn execute_extension_command(
    extension_manager: &ExtensionManager,
    command: &LoadedCommand,
    argument: Option<&String>,
    platform: &dyn Platform,
) -> ExecutionResult {
    let result = match extension_manager.execute_command(command, argument.map(|s| s.as_str())) {
        Ok(r) => r,
        Err(e) => return ExecutionResult::Error(e),
    };

    // Check for script errors
    if let Some(ref error) = result.error {
        return ExecutionResult::Error(error.clone());
    }

    match command.output {
        OutputMode::Silent => ExecutionResult::Success,
        OutputMode::Notification => {
            if let Some(item) = result.items.first() {
                let title = &item.title;
                let body = item.subtitle.as_deref().unwrap_or("");
                let _ = platform.show_notification(title, body);
            }
            ExecutionResult::Success
        }
        OutputMode::Clipboard => {
            if let Some(item) = result.items.first() {
                if let Err(e) = platform.clipboard_write(&item.title) {
                    return ExecutionResult::Error(e);
                }
                let _ = platform.show_notification("Copied to clipboard", &item.title);
            }
            ExecutionResult::Success
        }
        OutputMode::List => {
            // For list mode, show as notification for now
            if !result.items.is_empty() {
                let summary = result
                    .items
                    .iter()
                    .take(3)
                    .map(|i| i.title.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                let _ = platform.show_notification("Extension Results", &summary);
            }
            ExecutionResult::Success
        }
    }
}
