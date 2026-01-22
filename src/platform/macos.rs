//! macOS platform implementation.
//!
//! This module implements the Platform trait for macOS using:
//! - /Applications and ~/Applications scanning for app discovery
//! - arboard crate for clipboard operations
//! - `open` command for URLs and files
//! - osascript for notifications and system commands
//! - pmset for sleep, loginwindow for logout

use super::{AppEntry, CommandResult, NotifyResult, Platform, SystemCommand};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(feature = "iced-ui")]
use arboard::Clipboard;

/// macOS platform implementation.
pub struct MacOSPlatform {
    #[cfg(feature = "iced-ui")]
    clipboard: std::sync::Mutex<Option<Clipboard>>,
}

impl MacOSPlatform {
    /// Create a new macOS platform instance.
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "iced-ui")]
            clipboard: std::sync::Mutex::new(Clipboard::new().ok()),
        }
    }

    /// Parse an .app bundle's Info.plist to extract application metadata.
    fn parse_app_bundle(&self, app_path: &Path) -> Option<AppEntry> {
        let info_plist = app_path.join("Contents/Info.plist");
        if !info_plist.exists() {
            return None;
        }

        // Get the app name from the bundle name or CFBundleName
        let bundle_name = app_path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())?;

        // Try to read plist using plutil (converts to JSON for easier parsing)
        let plist_json = Command::new("plutil")
            .args(["-convert", "json", "-o", "-", info_plist.to_str()?])
            .output()
            .ok()?;

        if !plist_json.status.success() {
            // Fallback to just using bundle name
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

        // Extract metadata from plist
        let name = plist
            .get("CFBundleDisplayName")
            .or_else(|| plist.get("CFBundleName"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| bundle_name.clone());

        let bundle_id = plist
            .get("CFBundleIdentifier")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| bundle_name.to_lowercase().replace(' ', "."));

        // Build keywords from various plist fields
        let mut keywords: Vec<String> = vec![
            name.to_lowercase(),
            bundle_name.to_lowercase(),
        ];

        // Add bundle ID parts as keywords
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

    /// Find the icon file for an app bundle.
    fn find_app_icon(&self, app_path: &Path) -> Option<String> {
        // Try to get icon from plist
        let info_plist = app_path.join("Contents/Info.plist");
        if info_plist.exists() {
            if let Ok(output) = Command::new("plutil")
                .args(["-convert", "json", "-o", "-", info_plist.to_str()?])
                .output()
            {
                if let Ok(plist) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                    if let Some(icon_name) = plist.get("CFBundleIconFile").and_then(|v| v.as_str()) {
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

        // Fallback: look for common icon names
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

    /// Scan a directory for .app bundles.
    fn scan_applications_dir(&self, dir: &Path, apps: &mut Vec<AppEntry>, seen: &mut HashSet<String>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            // Check if it's an .app bundle
            if path.is_dir() && path.extension().is_some_and(|ext| ext == "app") {
                if let Some(app) = self.parse_app_bundle(&path) {
                    // Avoid duplicates
                    if !seen.contains(&app.id) {
                        seen.insert(app.id.clone());
                        apps.push(app);
                    }
                }
            }
            // Also recurse into subdirectories (for things like /Applications/Utilities)
            else if path.is_dir() {
                // Only recurse one level to avoid deep scanning
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

    /// Use Spotlight (mdfind) to discover additional applications.
    #[allow(dead_code)]
    fn discover_via_spotlight(&self, apps: &mut Vec<AppEntry>, seen: &mut HashSet<String>) {
        // Use mdfind to find applications via Spotlight index
        let output = Command::new("mdfind")
            .args(["kMDItemContentType == 'com.apple.application-bundle'"])
            .output();

        let Ok(output) = output else {
            return;
        };

        if !output.status.success() {
            return;
        }

        let paths = String::from_utf8_lossy(&output.stdout);
        for line in paths.lines() {
            let path = Path::new(line.trim());
            if path.is_dir() && path.extension().is_some_and(|ext| ext == "app") {
                if let Some(app) = self.parse_app_bundle(path) {
                    if !seen.contains(&app.id) {
                        seen.insert(app.id.clone());
                        apps.push(app);
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

        // Scan standard application directories
        self.scan_applications_dir(Path::new("/Applications"), &mut apps, &mut seen);
        self.scan_applications_dir(Path::new("/System/Applications"), &mut apps, &mut seen);

        // Scan user's Applications folder
        if let Some(home) = dirs::home_dir() {
            self.scan_applications_dir(&home.join("Applications"), &mut apps, &mut seen);
        }

        // Optionally use Spotlight for additional apps (can be slow)
        // Uncomment if you want more thorough discovery:
        // self.discover_via_spotlight(&mut apps, &mut seen);

        // Sort by name
        apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        println!("[Nova macOS] Discovered {} applications", apps.len());
        apps
    }

    fn clipboard_read(&self) -> Option<String> {
        #[cfg(feature = "iced-ui")]
        {
            let mut guard = self.clipboard.lock().ok()?;
            let clipboard = guard.as_mut()?;
            clipboard.get_text().ok()
        }
        #[cfg(not(feature = "iced-ui"))]
        {
            // Fallback using pbpaste
            let output = Command::new("pbpaste").output().ok()?;
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                None
            }
        }
    }

    fn clipboard_write(&self, content: &str) -> Result<(), String> {
        #[cfg(feature = "iced-ui")]
        {
            let mut guard = self.clipboard.lock().map_err(|e| e.to_string())?;
            let clipboard = guard.as_mut().ok_or("Clipboard not initialized")?;
            clipboard.set_text(content).map_err(|e| e.to_string())
        }
        #[cfg(not(feature = "iced-ui"))]
        {
            // Fallback using pbcopy
            use std::io::Write;
            let mut child = Command::new("pbcopy")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| format!("Failed to spawn pbcopy: {}", e))?;

            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(content.as_bytes())
                    .map_err(|e| format!("Failed to write to pbcopy: {}", e))?;
            }

            child.wait().map_err(|e| format!("pbcopy failed: {}", e))?;
            Ok(())
        }
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
        // Use osascript to display a notification
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
                // Lock screen using Keychain command
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
                // Use pmset to sleep
                Command::new("pmset")
                    .arg("sleepnow")
                    .status()
                    .map_err(|e| format!("Failed to sleep: {}", e))?;
                Ok(())
            }

            SystemCommand::Logout => {
                // Use osascript to log out
                Command::new("osascript")
                    .args([
                        "-e",
                        r#"tell application "System Events" to log out"#,
                    ])
                    .status()
                    .map_err(|e| format!("Failed to logout: {}", e))?;
                Ok(())
            }

            SystemCommand::Restart => {
                // Use osascript to restart
                Command::new("osascript")
                    .args([
                        "-e",
                        r#"tell application "System Events" to restart"#,
                    ])
                    .status()
                    .map_err(|e| format!("Failed to restart: {}", e))?;
                Ok(())
            }

            SystemCommand::Shutdown => {
                // Use osascript to shut down
                Command::new("osascript")
                    .args([
                        "-e",
                        r#"tell application "System Events" to shut down"#,
                    ])
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
        // On macOS, config and data are typically in the same location
        self.config_dir()
    }

    fn runtime_dir(&self) -> PathBuf {
        std::env::var("TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
    }

    fn launch_app(&self, app: &AppEntry) -> Result<(), String> {
        // Check if exec is a .app bundle path or an executable
        let path = Path::new(&app.exec);

        if path.extension().is_some_and(|ext| ext == "app") {
            // It's an app bundle, use `open`
            Command::new("open")
                .arg(&app.exec)
                .spawn()
                .map_err(|e| format!("Failed to launch {}: {}", app.name, e))?;
        } else {
            // Try to launch it directly or via open -a
            Command::new("open")
                .args(["-a", &app.exec])
                .spawn()
                .or_else(|_| {
                    // Fallback: try to run it directly
                    Command::new(&app.exec).spawn()
                })
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
