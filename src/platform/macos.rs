//! macOS platform implementation.
//!
//! This module implements the Platform trait for macOS using:
//! - /Applications and ~/Applications scanning for app discovery
//! - pbcopy/pbpaste for clipboard operations
//! - `open` command for URLs and files
//! - osascript for notifications and system commands
//! - pmset for sleep, loginwindow for logout
//! - System Events for window management (requires Accessibility permission)

use super::{
    AppEntry, CommandResult, NotifyResult, Platform, ScreenInfo, SystemCommand, WindowFrame,
    WindowInfo, WindowPosition,
};
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
            return Some(AppEntry::new(
                bundle_name.to_lowercase().replace(' ', "-"),
                bundle_name.clone(),
                app_path.to_string_lossy().to_string(),
                self.find_app_icon(app_path),
                None,
                vec![bundle_name.to_lowercase()],
            ));
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

        Some(AppEntry::new(
            bundle_id,
            name,
            app_path.to_string_lossy().to_string(),
            self.find_app_icon(app_path),
            plist
                .get("CFBundleGetInfoString")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            keywords,
        ))
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

    // ==================== Window Management ====================

    fn window_management_supported(&self) -> bool {
        true
    }

    fn get_focused_window(&self) -> Result<WindowInfo, String> {
        // AppleScript to get the frontmost window info
        let script = r#"
            tell application "System Events"
                set frontApp to first application process whose frontmost is true
                set appName to name of frontApp
                set bundleId to bundle identifier of frontApp
                set appPid to unix id of frontApp

                try
                    set frontWindow to window 1 of frontApp
                    set windowTitle to name of frontWindow
                    set windowPos to position of frontWindow
                    set windowSize to size of frontWindow
                    set isMinimized to (value of attribute "AXMinimized" of frontWindow) as boolean
                    set isFullscreen to false
                    try
                        set isFullscreen to (value of attribute "AXFullScreen" of frontWindow) as boolean
                    end try

                    return appName & "||" & bundleId & "||" & appPid & "||" & windowTitle & "||" & (item 1 of windowPos) & "||" & (item 2 of windowPos) & "||" & (item 1 of windowSize) & "||" & (item 2 of windowSize) & "||" & isMinimized & "||" & isFullscreen
                on error
                    return "error||No window available"
                end try
            end tell
        "#;

        let output = Command::new("osascript")
            .args(["-e", script])
            .output()
            .map_err(|e| format!("Failed to run osascript: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Check for accessibility permission error
            if stderr.contains("not allowed assistive access") || stderr.contains("osascript is not allowed") {
                return Err("Accessibility permission required. Please enable Nova in System Preferences > Security & Privacy > Privacy > Accessibility".to_string());
            }
            return Err(format!("osascript failed: {}", stderr));
        }

        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if let Some(error_msg) = result.strip_prefix("error||") {
            return Err(error_msg.to_string());
        }

        // Parse the result: appName||bundleId||pid||title||x||y||width||height||minimized||fullscreen
        let parts: Vec<&str> = result.split("||").collect();
        if parts.len() < 10 {
            return Err(format!("Unexpected output format: {}", result));
        }

        Ok(WindowInfo {
            id: parts[2].parse::<u32>().unwrap_or(0) as u64, // Using PID as window ID for AppleScript
            title: parts[3].to_string(),
            app_name: parts[0].to_string(),
            app_id: parts[1].to_string(),
            pid: parts[2].parse::<u32>().unwrap_or(0),
            frame: WindowFrame {
                x: parts[4].parse::<f64>().unwrap_or(0.0) as i32,
                y: parts[5].parse::<f64>().unwrap_or(0.0) as i32,
                width: parts[6].parse::<f64>().unwrap_or(0.0) as u32,
                height: parts[7].parse::<f64>().unwrap_or(0.0) as u32,
            },
            is_minimized: parts[8] == "true",
            is_fullscreen: parts[9] == "true",
        })
    }

    fn list_windows(&self) -> Result<Vec<WindowInfo>, String> {
        // AppleScript to list all windows (limited to reduce performance impact)
        let script = r#"
            set windowList to ""
            tell application "System Events"
                repeat with proc in (every process whose visible is true)
                    set appName to name of proc
                    set bundleId to bundle identifier of proc
                    set appPid to unix id of proc

                    try
                        repeat with win in (every window of proc)
                            try
                                set windowTitle to name of win
                                set windowPos to position of win
                                set windowSize to size of win
                                set isMinimized to (value of attribute "AXMinimized" of win) as boolean
                                set isFullscreen to false
                                try
                                    set isFullscreen to (value of attribute "AXFullScreen" of win) as boolean
                                end try

                                set windowList to windowList & appName & "||" & bundleId & "||" & appPid & "||" & windowTitle & "||" & (item 1 of windowPos) & "||" & (item 2 of windowPos) & "||" & (item 1 of windowSize) & "||" & (item 2 of windowSize) & "||" & isMinimized & "||" & isFullscreen & "
"
                            end try
                        end repeat
                    end try
                end repeat
            end tell
            return windowList
        "#;

        let output = Command::new("osascript")
            .args(["-e", script])
            .output()
            .map_err(|e| format!("Failed to run osascript: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("not allowed assistive access") {
                return Err("Accessibility permission required".to_string());
            }
            return Err(format!("osascript failed: {}", stderr));
        }

        let result = String::from_utf8_lossy(&output.stdout);
        let mut windows = Vec::new();

        for line in result.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split("||").collect();
            if parts.len() >= 10 {
                windows.push(WindowInfo {
                    id: parts[2].parse::<u32>().unwrap_or(0) as u64,
                    title: parts[3].to_string(),
                    app_name: parts[0].to_string(),
                    app_id: parts[1].to_string(),
                    pid: parts[2].parse::<u32>().unwrap_or(0),
                    frame: WindowFrame {
                        x: parts[4].parse::<f64>().unwrap_or(0.0) as i32,
                        y: parts[5].parse::<f64>().unwrap_or(0.0) as i32,
                        width: parts[6].parse::<f64>().unwrap_or(0.0) as u32,
                        height: parts[7].parse::<f64>().unwrap_or(0.0) as u32,
                    },
                    is_minimized: parts[8] == "true",
                    is_fullscreen: parts[9] == "true",
                });
            }
        }

        Ok(windows)
    }

    fn list_screens(&self) -> Result<Vec<ScreenInfo>, String> {
        // Use system_profiler to get display info (faster than AppleScript)
        let output = Command::new("system_profiler")
            .args(["SPDisplaysDataType", "-json"])
            .output()
            .map_err(|e| format!("Failed to run system_profiler: {}", e))?;

        if !output.status.success() {
            // Fallback to AppleScript
            return self.list_screens_applescript();
        }

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("Failed to parse display info: {}", e))?;

        let mut screens = Vec::new();
        let mut screen_id = 0u32;

        // Parse the JSON structure
        if let Some(displays) = json.get("SPDisplaysDataType") {
            if let Some(gpu_list) = displays.as_array() {
                for gpu in gpu_list {
                    if let Some(display_list) = gpu.get("spdisplays_ndrvs").and_then(|d| d.as_array()) {
                        for display in display_list {
                            let name = display
                                .get("_name")
                                .and_then(|n| n.as_str())
                                .unwrap_or("Unknown Display")
                                .to_string();

                            // Get resolution string like "1920 x 1080"
                            let resolution = display
                                .get("_spdisplays_resolution")
                                .and_then(|r| r.as_str())
                                .unwrap_or("1920 x 1080");

                            // Parse resolution
                            let (width, height) = parse_resolution(resolution);

                            let is_main = display
                                .get("spdisplays_main")
                                .and_then(|m| m.as_str())
                                .map(|m| m == "spdisplays_yes")
                                .unwrap_or(screen_id == 0);

                            // Calculate position (main screen at 0,0, others offset)
                            let x = if is_main { 0 } else { screens.iter().map(|s: &ScreenInfo| s.frame.width as i32).sum() };

                            // Menu bar is typically 25 pixels on macOS
                            let menu_bar_height = if is_main { 25 } else { 0 };

                            screens.push(ScreenInfo {
                                id: screen_id,
                                name,
                                frame: WindowFrame {
                                    x,
                                    y: 0,
                                    width,
                                    height,
                                },
                                visible_frame: WindowFrame {
                                    x,
                                    y: menu_bar_height,
                                    width,
                                    height: height.saturating_sub(menu_bar_height as u32),
                                },
                                is_primary: is_main,
                            });

                            screen_id += 1;
                        }
                    }
                }
            }
        }

        if screens.is_empty() {
            // Fallback: return a default screen
            screens.push(ScreenInfo {
                id: 0,
                name: "Main Display".to_string(),
                frame: WindowFrame {
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1080,
                },
                visible_frame: WindowFrame {
                    x: 0,
                    y: 25,
                    width: 1920,
                    height: 1055,
                },
                is_primary: true,
            });
        }

        Ok(screens)
    }

    fn set_window_frame(&self, window_id: u64, frame: WindowFrame) -> Result<(), String> {
        // window_id is the PID in our implementation
        let pid = window_id as i32;

        // AppleScript to move and resize window by PID
        let script = format!(
            r#"
            tell application "System Events"
                set targetProc to first process whose unix id is {}
                set frontWindow to window 1 of targetProc
                set position of frontWindow to {{{}, {}}}
                set size of frontWindow to {{{}, {}}}
            end tell
            "#,
            pid, frame.x, frame.y, frame.width, frame.height
        );

        let output = Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| format!("Failed to run osascript: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("not allowed assistive access") {
                return Err("Accessibility permission required".to_string());
            }
            return Err(format!("Failed to set window frame: {}", stderr));
        }

        Ok(())
    }

    fn set_window_position(&self, window_id: u64, position: WindowPosition) -> Result<(), String> {
        // Get current window info first
        let window = self.get_focused_window()?;

        // Get screens and find the one containing the window
        let screens = self.list_screens()?;

        let window_center_x = window.frame.x + (window.frame.width as i32 / 2);
        let window_center_y = window.frame.y + (window.frame.height as i32 / 2);

        let screen = screens
            .iter()
            .find(|s| {
                window_center_x >= s.visible_frame.x
                    && window_center_x < s.visible_frame.x + s.visible_frame.width as i32
                    && window_center_y >= s.visible_frame.y
                    && window_center_y < s.visible_frame.y + s.visible_frame.height as i32
            })
            .or_else(|| screens.first())
            .ok_or("No screens available")?;

        let frame = position.calculate_frame(&screen.visible_frame);
        self.set_window_frame(window_id, frame)
    }
}

impl MacOSPlatform {
    /// Fallback method to get screen info via AppleScript
    fn list_screens_applescript(&self) -> Result<Vec<ScreenInfo>, String> {
        let script = r#"
            tell application "Finder"
                set screenBounds to bounds of window of desktop
                return (item 3 of screenBounds) & "||" & (item 4 of screenBounds)
            end tell
        "#;

        let output = Command::new("osascript")
            .args(["-e", script])
            .output()
            .map_err(|e| format!("Failed to run osascript: {}", e))?;

        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let parts: Vec<&str> = result.split("||").collect();

        let width = parts.first().and_then(|s| s.parse::<u32>().ok()).unwrap_or(1920);
        let height = parts.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(1080);

        Ok(vec![ScreenInfo {
            id: 0,
            name: "Main Display".to_string(),
            frame: WindowFrame {
                x: 0,
                y: 0,
                width,
                height,
            },
            visible_frame: WindowFrame {
                x: 0,
                y: 25, // Menu bar
                width,
                height: height.saturating_sub(25),
            },
            is_primary: true,
        }])
    }
}

/// Parse a resolution string like "1920 x 1080" or "2560 x 1440 (QHD/WQHD)"
fn parse_resolution(s: &str) -> (u32, u32) {
    let clean = s.split('(').next().unwrap_or(s).trim();
    let parts: Vec<&str> = clean.split('x').map(|p| p.trim()).collect();

    let width = parts.first()
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1920);

    let height = parts.get(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1080);

    (width, height)
}
