use std::process::Command;
use std::sync::Arc;

use iced::Task;

use nova_core::{Config, ExecutionAction};
use nova_platform::Platform;

use crate::app::Message;

/// Execute an action and return a Task with any resulting message
pub fn run_action(
    action: ExecutionAction,
    platform: &Arc<Platform>,
    _config: &Config,
) -> Task<Message> {
    match action {
        ExecutionAction::LaunchApp { exec, name } => {
            let platform = Arc::clone(platform);
            let app = nova_core::PlatformAppEntry {
                id: String::new(),
                name: name.clone(),
                exec,
                icon: None,
                description: None,
                keywords: Vec::new(),
            };
            match platform.apps.launch_app(&app) {
                Ok(()) => hide_task(),
                Err(e) => {
                    eprintln!("[Nova] Failed to launch {}: {}", name, e);
                    Task::none()
                }
            }
        }

        ExecutionAction::OpenSettings => Task::done(Message::SettingsToggle),

        ExecutionAction::Quit => {
            std::process::exit(0);
        }

        ExecutionAction::SystemCommand { command } => {
            let platform = Arc::clone(platform);
            if let Err(e) = platform.system.execute(command) {
                eprintln!("[Nova] System command failed: {}", e);
            }
            hide_task()
        }

        ExecutionAction::RunShellCommand { command } => {
            if let Err(e) = Command::new("sh").arg("-c").arg(&command).spawn() {
                eprintln!("[Nova] Failed to run command: {}", e);
            }
            hide_task()
        }

        ExecutionAction::OpenUrl { url } => {
            let platform = Arc::clone(platform);
            if let Err(e) = platform.opener.open_url(&url) {
                eprintln!("[Nova] Failed to open URL: {}", e);
            }
            hide_task()
        }

        ExecutionAction::RunScript {
            path,
            argument,
            output_mode,
        } => {
            let platform = Arc::clone(platform);

            let mut cmd = Command::new(&path);
            if let Some(ref arg) = argument {
                cmd.arg(arg);
            }

            match cmd.output() {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout)
                        .trim()
                        .to_string();
                    handle_script_output(&platform, &stdout, &output_mode);
                }
                Err(e) => {
                    eprintln!("[Nova] Script failed: {}", e);
                }
            }
            hide_task()
        }

        ExecutionAction::RunExtensionCommand { command, argument } => {
            // Extension command execution would go through ExtensionManager
            // For now just hide
            eprintln!(
                "[Nova] Would execute extension command: {} with arg: {:?}",
                command.name, argument
            );
            hide_task()
        }

        ExecutionAction::CopyToClipboard {
            content,
            notification,
        } => {
            let platform = Arc::clone(platform);
            if let Err(e) = platform.clipboard.set_text(&content) {
                eprintln!("[Nova] Clipboard copy failed: {}", e);
            } else {
                let _ = platform.notifications.show("Copied", &notification);
            }
            hide_task()
        }

        ExecutionAction::OpenFile { path } => {
            let platform = Arc::clone(platform);
            if let Err(e) = platform.opener.open_file(&path) {
                eprintln!("[Nova] Failed to open file: {}", e);
            }
            hide_task()
        }

        ExecutionAction::NeedsInput => Task::none(),
    }
}

fn handle_script_output(
    platform: &Arc<Platform>,
    stdout: &str,
    output_mode: &nova_core::services::ScriptOutputMode,
) {
    use nova_core::services::ScriptOutputMode;
    match output_mode {
        ScriptOutputMode::Notification => {
            let _ = platform.notifications.show("Nova Script", stdout);
        }
        ScriptOutputMode::Clipboard => {
            if let Err(e) = platform.clipboard.set_text(stdout) {
                eprintln!("[Nova] Failed to copy script output: {}", e);
            } else {
                let _ = platform.notifications.show("Copied", "Script output copied");
            }
        }
        ScriptOutputMode::Silent | ScriptOutputMode::Inline => {}
    }
}

fn hide_task() -> Task<Message> {
    Task::done(Message::Hide)
}
