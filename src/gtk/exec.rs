//! Execution helpers for GTK frontend.

use nova::services::{self, OutputMode, ScriptOutputMode};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;

pub fn open_url(url: &str) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map_err(|e| format!("Failed to open URL: {}", e))?;
    Ok(())
}

pub fn execute_script(
    path: &PathBuf,
    argument: Option<&String>,
    output_mode: &ScriptOutputMode,
) -> Result<(), String> {
    let mut cmd = Command::new(path);

    if let Some(arg) = argument {
        cmd.arg(arg);
    }

    match output_mode {
        ScriptOutputMode::Silent => {
            cmd.spawn()
                .map_err(|e| format!("Failed to execute script: {}", e))?;
        }
        ScriptOutputMode::Notification => {
            let output = cmd
                .output()
                .map_err(|e| format!("Failed to execute script: {}", e))?;
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !stdout.is_empty() {
                show_notification("Nova Script", &stdout)?;
            }
        }
        ScriptOutputMode::Clipboard => {
            let output = cmd
                .output()
                .map_err(|e| format!("Failed to execute script: {}", e))?;
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !stdout.is_empty() {
                copy_to_clipboard(&stdout)?;
                show_notification("Copied to clipboard", &stdout)?;
            }
        }
        ScriptOutputMode::Inline => {
            // For now, treat inline same as notification
            let output = cmd
                .output()
                .map_err(|e| format!("Failed to execute script: {}", e))?;
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !stdout.is_empty() {
                show_notification("Nova Script", &stdout)?;
            }
        }
    }

    Ok(())
}

/// Execute an extension command and handle its output
pub fn execute_extension_command(
    extension_manager: &Rc<services::ExtensionManager>,
    command: &services::LoadedCommand,
    argument: Option<&String>,
) -> Result<(), String> {
    let result = extension_manager.execute_command(command, argument.map(|s| s.as_str()))?;

    // Check for script errors
    if let Some(ref error) = result.error {
        return Err(error.clone());
    }

    match command.output {
        OutputMode::Silent => {
            // Nothing to do
        }
        OutputMode::Notification => {
            // Show first result item as notification
            if let Some(item) = result.items.first() {
                let title = &item.title;
                let body = item.subtitle.as_deref().unwrap_or("");
                show_notification(title, body)?;
            }
        }
        OutputMode::Clipboard => {
            // Copy first result to clipboard
            if let Some(item) = result.items.first() {
                copy_to_clipboard(&item.title)?;
                show_notification("Copied to clipboard", &item.title)?;
            }
        }
        OutputMode::List => {
            // For list mode, we would show results in the UI
            // For now, show as notification (TODO: implement inline results)
            if !result.items.is_empty() {
                let summary = result
                    .items
                    .iter()
                    .take(3)
                    .map(|i| i.title.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                show_notification("Extension Results", &summary)?;
            }
        }
    }

    Ok(())
}

pub fn show_notification(title: &str, body: &str) -> Result<(), String> {
    Command::new("notify-send")
        .args([title, body])
        .spawn()
        .map_err(|e| format!("Failed to show notification: {}", e))?;
    Ok(())
}

pub fn copy_to_clipboard(content: &str) -> Result<(), String> {
    let mut child = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to set clipboard: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write to clipboard: {}", e))?;
    }

    Ok(())
}
