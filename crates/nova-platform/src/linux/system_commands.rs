use nova_core::{NovaError, NovaResult, SystemCommand};
use std::process::Command;

use crate::traits::SystemCommands;

pub struct LinuxSystemCommands;

impl SystemCommands for LinuxSystemCommands {
    fn execute(&self, command: SystemCommand) -> NovaResult<()> {
        let (cmd, args) = command_args(&command);

        let result = Command::new(cmd).args(&args).spawn();

        match result {
            Ok(_) => Ok(()),
            Err(_) if matches!(command, SystemCommand::Logout) => {
                // Fallback for logout
                let user = std::env::var("USER").unwrap_or_default();
                Command::new("loginctl")
                    .args(["terminate-user", &user])
                    .spawn()
                    .map_err(|e| NovaError::Platform(format!("Failed to logout: {}", e)))?;
                Ok(())
            }
            Err(e) => Err(NovaError::Platform(format!(
                "Failed to execute system command: {}",
                e
            ))),
        }
    }
}

fn command_args(command: &SystemCommand) -> (&'static str, Vec<&'static str>) {
    match command {
        SystemCommand::Lock => ("loginctl", vec!["lock-session"]),
        SystemCommand::Sleep => ("systemctl", vec!["suspend"]),
        SystemCommand::Logout => ("gnome-session-quit", vec!["--logout", "--no-prompt"]),
        SystemCommand::Restart => ("systemctl", vec!["reboot"]),
        SystemCommand::Shutdown => ("systemctl", vec!["poweroff"]),
    }
}
