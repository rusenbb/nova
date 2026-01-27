//! Linux platform implementation.
//!
//! Implements the Platform trait for Linux systems using:
//! - XDG desktop files for application discovery
//! - xclip for clipboard operations
//! - xdg-open for opening URLs and files
//! - notify-send for desktop notifications
//! - systemctl/loginctl for system commands

use super::{AppEntry, CommandResult, NotifyResult, Platform, SystemCommand};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use freedesktop_desktop_entry::DesktopEntry;
use walkdir::WalkDir;

/// Linux platform implementation.
pub struct LinuxPlatform {
    /// Cached list of applications (populated on first call to discover_apps)
    apps_cache: Option<Vec<AppEntry>>,
}

impl LinuxPlatform {
    /// Create a new Linux platform instance.
    pub fn new() -> Self {
        Self { apps_cache: None }
    }

    /// Scan a directory for .desktop files and add them to the entries list.
    fn scan_directory(dir: &PathBuf, entries: &mut Vec<AppEntry>) {
        for entry in WalkDir::new(dir)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "desktop") {
                if let Some(app_entry) = Self::parse_desktop_file(path.to_path_buf()) {
                    // Skip duplicates by ID
                    if !entries.iter().any(|e| e.id == app_entry.id) {
                        entries.push(app_entry);
                    }
                }
            }
        }
    }

    /// Parse a .desktop file into an AppEntry.
    fn parse_desktop_file(path: PathBuf) -> Option<AppEntry> {
        let content = std::fs::read_to_string(&path).ok()?;
        let entry = DesktopEntry::from_str(&path, &content, Some(&["en"])).ok()?;

        // Skip entries that shouldn't be shown
        if entry.no_display() || entry.hidden() {
            return None;
        }

        // Use empty locale list to get default (untranslated) values
        let locales: &[&str] = &[];

        // Skip entries without a name or exec
        let name = entry.name(locales)?.to_string();
        let exec = entry.exec()?.to_string();

        // Use filename as ID
        let id = path.file_stem()?.to_string_lossy().to_string();

        let icon = entry.icon().map(|s| s.to_string());
        let description = entry.comment(locales).map(|s| s.to_string());

        // Collect keywords
        let mut keywords: Vec<String> = entry
            .keywords(locales)
            .map(|kw| kw.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default();

        // Add name words as keywords for better matching
        keywords.extend(name.split_whitespace().map(|s| s.to_lowercase()));

        Some(AppEntry {
            id,
            name,
            exec,
            icon,
            description,
            keywords,
        })
    }

    /// Strip field codes from exec command (%f, %u, %F, %U, etc.)
    fn strip_field_codes(exec: &str) -> String {
        exec.replace("%f", "")
            .replace("%F", "")
            .replace("%u", "")
            .replace("%U", "")
            .replace("%i", "")
            .replace("%c", "")
            .replace("%k", "")
    }
}

impl Default for LinuxPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl Platform for LinuxPlatform {
    fn discover_apps(&self) -> Vec<AppEntry> {
        let mut entries = Vec::new();

        // Standard XDG application directories
        let mut dirs_to_scan: Vec<PathBuf> = vec![
            PathBuf::from("/usr/share/applications"),
            PathBuf::from("/usr/local/share/applications"),
        ];

        // User local applications
        if let Some(data_home) = dirs::data_local_dir() {
            dirs_to_scan.push(data_home.join("applications"));
        }

        // Flatpak applications
        if let Some(home) = dirs::home_dir() {
            dirs_to_scan.push(home.join(".local/share/flatpak/exports/share/applications"));
        }

        // Snap applications
        dirs_to_scan.push(PathBuf::from("/var/lib/snapd/desktop/applications"));

        for dir in dirs_to_scan {
            if dir.exists() {
                Self::scan_directory(&dir, &mut entries);
            }
        }

        // Sort by name for consistent ordering
        entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        entries
    }

    fn clipboard_read(&self) -> Option<String> {
        let output = Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
            .ok()?;

        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout).to_string();
            // Skip empty content
            if content.trim().is_empty() {
                return None;
            }
            Some(content)
        } else {
            None
        }
    }

    fn clipboard_write(&self, content: &str) -> Result<(), String> {
        let mut child = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start xclip: {}", e))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(content.as_bytes())
                .map_err(|e| format!("Failed to write to clipboard: {}", e))?;
        }

        child
            .wait()
            .map_err(|e| format!("Failed to wait for xclip: {}", e))?;

        Ok(())
    }

    fn open_url(&self, url: &str) -> Result<(), String> {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
        Ok(())
    }

    fn open_file(&self, path: &str) -> Result<(), String> {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
        Ok(())
    }

    fn show_notification(&self, title: &str, body: &str) -> NotifyResult {
        Command::new("notify-send")
            .args([title, body])
            .spawn()
            .map_err(|e| format!("Failed to show notification: {}", e))?;
        Ok(())
    }

    fn system_command(&self, command: SystemCommand) -> CommandResult {
        let (cmd, args): (&str, Vec<&str>) = match command {
            SystemCommand::Lock => ("loginctl", vec!["lock-session"]),
            SystemCommand::Sleep => ("systemctl", vec!["suspend"]),
            SystemCommand::Logout => ("gnome-session-quit", vec!["--logout", "--no-prompt"]),
            SystemCommand::Restart => ("systemctl", vec!["reboot"]),
            SystemCommand::Shutdown => ("systemctl", vec!["poweroff"]),
        };

        let status = Command::new(cmd).args(&args).status();

        match status {
            Ok(s) if s.success() => Ok(()),
            Ok(_) => {
                // Try fallback for logout
                if command == SystemCommand::Logout {
                    let user = std::env::var("USER").unwrap_or_default();
                    Command::new("loginctl")
                        .args(["terminate-user", &user])
                        .status()
                        .map_err(|e| format!("Logout fallback failed: {}", e))?;
                    Ok(())
                } else {
                    Err(format!("Command {} failed", cmd))
                }
            }
            Err(e) => {
                // Try fallback for logout
                if command == SystemCommand::Logout {
                    let user = std::env::var("USER").unwrap_or_default();
                    Command::new("loginctl")
                        .args(["terminate-user", &user])
                        .status()
                        .map_err(|e| format!("Logout fallback failed: {}", e))?;
                    Ok(())
                } else {
                    Err(format!("Failed to execute {}: {}", cmd, e))
                }
            }
        }
    }

    fn config_dir(&self) -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .map(|h| h.join(".config"))
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
            })
            .join("nova")
    }

    fn data_dir(&self) -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .map(|h| h.join(".local/share"))
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
            })
            .join("nova")
    }

    fn runtime_dir(&self) -> PathBuf {
        std::env::var("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
    }

    fn launch_app(&self, app: &AppEntry) -> Result<(), String> {
        let exec = Self::strip_field_codes(&app.exec);
        let parts: Vec<&str> = exec.split_whitespace().collect();

        if parts.is_empty() {
            return Err("Empty exec command".to_string());
        }

        let program = parts[0];
        let args = &parts[1..];

        Command::new(program)
            .args(args)
            .spawn()
            .map_err(|e| format!("Failed to launch {}: {}", app.name, e))?;

        Ok(())
    }

    fn run_shell_command(&self, command: &str) -> Result<(), String> {
        Command::new("sh")
            .args(["-c", command])
            .spawn()
            .map_err(|e| format!("Failed to run command: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_field_codes() {
        assert_eq!(LinuxPlatform::strip_field_codes("firefox %u"), "firefox ");
        assert_eq!(LinuxPlatform::strip_field_codes("code %F"), "code ");
        assert_eq!(LinuxPlatform::strip_field_codes("gimp %f %i"), "gimp  ");
    }

    #[test]
    fn test_config_dir() {
        let platform = LinuxPlatform::new();
        let config_dir = platform.config_dir();
        assert!(config_dir.ends_with("nova"));
    }

    #[test]
    fn test_data_dir() {
        let platform = LinuxPlatform::new();
        let data_dir = platform.data_dir();
        assert!(data_dir.ends_with("nova"));
    }
}
