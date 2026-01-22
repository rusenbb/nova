//! macOS platform implementation.
//!
//! This module implements the Platform trait for macOS using:
//! - /Applications and ~/Applications scanning for app discovery
//! - pbcopy/pbpaste for clipboard operations
//! - `open` command for URLs and files
//! - osascript for notifications and system commands
//! - pmset for sleep, loginwindow for logout

use super::{AppEntry, CommandResult, NotifyResult, Platform, SystemCommand};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Check if we're running on macOS.
#[inline]
pub fn is_macos() -> bool {
    true
}

/// macOS platform implementation.
pub struct MacOSPlatform;

impl MacOSPlatform {
    /// Create a new macOS platform instance.
    pub fn new() -> Self {
        Self
    }

    /// Parse an .app bundle's Info.plist to extract application metadata.
    fn parse_app_bundle(&self, app_path: &Path) -> Option<AppEntry> {
        let info_plist = app_path.join("Contents/Info.plist");
        if !info_plist.exists() {
            return None;
        }

        let bundle_name = app_path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())?;

        let plist_json = Command::new("plutil")
            .args(["-convert", "json", "-o", "-", info_plist.to_str()?])
            .output()
            .ok()?;

        if !plist_json.status.success() {
            return Some(AppEntry {
                id: bundle_name.to_lowercase().replace(' ', "-"),
                name: bundle_name.clone(),
                exec: app_path.to_string_lossy().to_string(),
                icon: self.find_app_icon(app_path),
                description: None,
                keywords: vec![bundle_name.to_lowercase()],
            });
        }

        let json_str = String::from_utf8_lossy(&plist_json.stdout);
        let plist: serde_json::Value = serde_json::from_str(&json_str).ok()?;

        let name = plist
            .get("CFBundleDisplayName")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                plist
                    .get("CFBundleName")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
            })
            .map(|s| s.to_string())
            .unwrap_or_else(|| bundle_name.clone());

        let bundle_id = plist
            .get("CFBundleIdentifier")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| bundle_name.to_lowercase().replace(' ', "."));

        let mut keywords: Vec<String> = vec![name.to_lowercase(), bundle_name.to_lowercase()];

        for part in bundle_id.split('.') {
            let lower = part.to_lowercase();
            if !keywords.contains(&lower) && lower.len() > 2 {
                keywords.push(lower);
            }
        }

        Some(AppEntry {
            id: bundle_id,
            name,
            exec: app_path.to_string_lossy().to_string(),
            icon: self.find_app_icon(app_path),
            description: plist
                .get("CFBundleGetInfoString")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            keywords,
        })
    }

    fn find_app_icon(&self, app_path: &Path) -> Option<String> {
        let info_plist = app_path.join("Contents/Info.plist");
        if info_plist.exists() {
            if let Ok(output) = Command::new("plutil")
                .args(["-convert", "json", "-o", "-", info_plist.to_str()?])
                .output()
            {
                if let Ok(plist) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                    if let Some(icon_name) = plist.get("CFBundleIconFile").and_then(|v| v.as_str())
                    {
                        let icon_name = if icon_name.ends_with(".icns") {
                            icon_name.to_string()
                        } else {
                            format!("{}.icns", icon_name)
                        };
                        let icon_path = app_path.join("Contents/Resources").join(&icon_name);
                        if icon_path.exists() {
                            return Some(icon_path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        let resources = app_path.join("Contents/Resources");
        if resources.is_dir() {
            for entry in fs::read_dir(&resources).ok()?.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "icns") {
                    return Some(path.to_string_lossy().to_string());
                }
            }
        }

        None
    }

    fn scan_applications_dir(
        &self,
        dir: &Path,
        apps: &mut Vec<AppEntry>,
        seen: &mut HashSet<String>,
    ) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() && path.extension().is_some_and(|ext| ext == "app") {
                if let Some(app) = self.parse_app_bundle(&path) {
                    if !seen.contains(&app.id) {
                        seen.insert(app.id.clone());
                        apps.push(app);
                    }
                }
            } else if path.is_dir() {
                if let Ok(subentries) = fs::read_dir(&path) {
                    for subentry in subentries.flatten() {
                        let subpath = subentry.path();
                        if subpath.is_dir() && subpath.extension().is_some_and(|ext| ext == "app") {
                            if let Some(app) = self.parse_app_bundle(&subpath) {
                                if !seen.contains(&app.id) {
                                    seen.insert(app.id.clone());
                                    apps.push(app);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Default for MacOSPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl Platform for MacOSPlatform {
    fn discover_apps(&self) -> Vec<AppEntry> {
        let mut apps = Vec::new();
        let mut seen = HashSet::new();

        self.scan_applications_dir(Path::new("/Applications"), &mut apps, &mut seen);
        self.scan_applications_dir(Path::new("/System/Applications"), &mut apps, &mut seen);

        if let Some(home) = dirs::home_dir() {
            self.scan_applications_dir(&home.join("Applications"), &mut apps, &mut seen);
        }

        apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        println!("[Nova macOS] Discovered {} applications", apps.len());
        apps
    }

    fn clipboard_read(&self) -> Option<String> {
        let output = Command::new("pbpaste").output().ok()?;
        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            None
        }
    }

    fn clipboard_write(&self, content: &str) -> Result<(), String> {
        use std::io::Write;
        let mut child = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn pbcopy: {}", e))?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(content.as_bytes())
                .map_err(|e| format!("Failed to write to pbcopy: {}", e))?;
        }

        child.wait().map_err(|e| format!("pbcopy failed: {}", e))?;
        Ok(())
    }

    fn open_url(&self, url: &str) -> Result<(), String> {
        Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
        Ok(())
    }

    fn open_file(&self, path: &str) -> Result<(), String> {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
        Ok(())
    }

    fn show_notification(&self, title: &str, body: &str) -> NotifyResult {
        let script = format!(
            r#"display notification "{}" with title "{}""#,
            body.replace('"', "\\\"").replace('\n', " "),
            title.replace('"', "\\\"")
        );

        let status = Command::new("osascript")
            .args(["-e", &script])
            .status()
            .map_err(|e| format!("Failed to run osascript: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err("osascript notification failed".to_string())
        }
    }

    fn system_command(&self, command: SystemCommand) -> CommandResult {
        match command {
            SystemCommand::Lock => {
                Command::new("osascript")
                    .args([
                        "-e",
                        r#"tell application "System Events" to keystroke "q" using {control down, command down}"#,
                    ])
                    .status()
                    .map_err(|e| format!("Failed to lock screen: {}", e))?;
                Ok(())
            }

            SystemCommand::Sleep => {
                Command::new("pmset")
                    .arg("sleepnow")
                    .status()
                    .map_err(|e| format!("Failed to sleep: {}", e))?;
                Ok(())
            }

            SystemCommand::Logout => {
                Command::new("osascript")
                    .args(["-e", r#"tell application "System Events" to log out"#])
                    .status()
                    .map_err(|e| format!("Failed to logout: {}", e))?;
                Ok(())
            }

            SystemCommand::Restart => {
                Command::new("osascript")
                    .args(["-e", r#"tell application "System Events" to restart"#])
                    .status()
                    .map_err(|e| format!("Failed to restart: {}", e))?;
                Ok(())
            }

            SystemCommand::Shutdown => {
                Command::new("osascript")
                    .args(["-e", r#"tell application "System Events" to shut down"#])
                    .status()
                    .map_err(|e| format!("Failed to shutdown: {}", e))?;
                Ok(())
            }
        }
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
        self.config_dir()
    }

    fn runtime_dir(&self) -> PathBuf {
        std::env::var("TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
    }

    fn launch_app(&self, app: &AppEntry) -> Result<(), String> {
        let path = Path::new(&app.exec);

        if path.extension().is_some_and(|ext| ext == "app") {
            Command::new("open")
                .arg(&app.exec)
                .spawn()
                .map_err(|e| format!("Failed to launch {}: {}", app.name, e))?;
        } else {
            Command::new("open")
                .args(["-a", &app.exec])
                .spawn()
                .or_else(|_| Command::new(&app.exec).spawn())
                .map_err(|e| format!("Failed to launch {}: {}", app.name, e))?;
        }

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
