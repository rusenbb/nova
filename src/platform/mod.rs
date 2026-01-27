//! Platform abstraction layer for cross-platform support.
//!
//! This module defines the `Platform` trait that abstracts all OS-specific
//! operations, allowing the core engine and UI to remain platform-agnostic.

// Allow dead code - trait methods and functions are used by FFI layer but not by GTK binary.
#![allow(dead_code)]

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
mod windows;

use std::path::PathBuf;

/// Represents an installed application discovered on the system.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppEntry {
    /// Unique identifier (e.g., filename without extension on Linux)
    pub id: String,
    /// Display name of the application
    pub name: String,
    /// Command to execute the application
    pub exec: String,
    /// Path to the application icon (platform-specific format)
    pub icon: Option<String>,
    /// Description or comment about the application
    pub description: Option<String>,
    /// Keywords for search matching
    pub keywords: Vec<String>,
}

/// System commands that can be executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SystemCommand {
    /// Lock the screen/session
    Lock,
    /// Put the computer to sleep/suspend
    Sleep,
    /// Log out of the current session
    Logout,
    /// Restart the computer
    Restart,
    /// Shut down the computer
    Shutdown,
}

// ============================================================================
// Window Management Types
// ============================================================================

/// Information about a window.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowInfo {
    /// Platform-specific window identifier
    pub id: u64,
    /// Window title
    pub title: String,
    /// Application name
    pub app_name: String,
    /// Application bundle ID (macOS) or class (X11) or exe name (Windows)
    pub app_id: String,
    /// Process ID
    pub pid: u32,
    /// Current window frame (position and size)
    pub frame: WindowFrame,
    /// Whether the window is fullscreen
    pub is_fullscreen: bool,
    /// Whether the window is minimized
    pub is_minimized: bool,
}

/// Window position and size.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct WindowFrame {
    /// X position (pixels from left of screen)
    pub x: i32,
    /// Y position (pixels from top of screen)
    pub y: i32,
    /// Window width in pixels
    pub width: u32,
    /// Window height in pixels
    pub height: u32,
}

/// Preset window positions for tiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WindowPosition {
    /// Left half of screen
    LeftHalf,
    /// Right half of screen
    RightHalf,
    /// Top half of screen
    TopHalf,
    /// Bottom half of screen
    BottomHalf,
    /// Top-left quarter
    TopLeftQuarter,
    /// Top-right quarter
    TopRightQuarter,
    /// Bottom-left quarter
    BottomLeftQuarter,
    /// Bottom-right quarter
    BottomRightQuarter,
    /// Left third of screen
    LeftThird,
    /// Center third of screen
    CenterThird,
    /// Right third of screen
    RightThird,
    /// Center of screen (reasonable size)
    Center,
    /// Maximize (full screen minus system UI)
    Maximize,
    /// Almost maximize (small margin)
    AlmostMaximize,
}

/// Screen/display information.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenInfo {
    /// Screen identifier
    pub id: u32,
    /// Screen name
    pub name: String,
    /// Total screen frame
    pub frame: WindowFrame,
    /// Visible frame (excluding menu bar, dock, etc.)
    pub visible_frame: WindowFrame,
    /// Is this the primary/main screen
    pub is_primary: bool,
}

impl WindowPosition {
    /// Calculate frame for this position given a screen's visible frame.
    pub fn calculate_frame(&self, screen: &WindowFrame) -> WindowFrame {
        let x = screen.x;
        let y = screen.y;
        let w = screen.width;
        let h = screen.height;
        let half_w = w / 2;
        let half_h = h / 2;
        let third_w = w / 3;
        let margin = 10i32;

        match self {
            WindowPosition::LeftHalf => WindowFrame {
                x,
                y,
                width: half_w,
                height: h,
            },
            WindowPosition::RightHalf => WindowFrame {
                x: x + half_w as i32,
                y,
                width: half_w,
                height: h,
            },
            WindowPosition::TopHalf => WindowFrame {
                x,
                y,
                width: w,
                height: half_h,
            },
            WindowPosition::BottomHalf => WindowFrame {
                x,
                y: y + half_h as i32,
                width: w,
                height: half_h,
            },
            WindowPosition::TopLeftQuarter => WindowFrame {
                x,
                y,
                width: half_w,
                height: half_h,
            },
            WindowPosition::TopRightQuarter => WindowFrame {
                x: x + half_w as i32,
                y,
                width: half_w,
                height: half_h,
            },
            WindowPosition::BottomLeftQuarter => WindowFrame {
                x,
                y: y + half_h as i32,
                width: half_w,
                height: half_h,
            },
            WindowPosition::BottomRightQuarter => WindowFrame {
                x: x + half_w as i32,
                y: y + half_h as i32,
                width: half_w,
                height: half_h,
            },
            WindowPosition::LeftThird => WindowFrame {
                x,
                y,
                width: third_w,
                height: h,
            },
            WindowPosition::CenterThird => WindowFrame {
                x: x + third_w as i32,
                y,
                width: third_w,
                height: h,
            },
            WindowPosition::RightThird => WindowFrame {
                x: x + (third_w * 2) as i32,
                y,
                width: third_w,
                height: h,
            },
            WindowPosition::Center => {
                let center_w = w * 2 / 3;
                let center_h = h * 2 / 3;
                WindowFrame {
                    x: x + ((w - center_w) / 2) as i32,
                    y: y + ((h - center_h) / 2) as i32,
                    width: center_w,
                    height: center_h,
                }
            }
            WindowPosition::Maximize => WindowFrame {
                x,
                y,
                width: w,
                height: h,
            },
            WindowPosition::AlmostMaximize => WindowFrame {
                x: x + margin,
                y: y + margin,
                width: w.saturating_sub((margin * 2) as u32),
                height: h.saturating_sub((margin * 2) as u32),
            },
        }
    }
}

impl std::str::FromStr for WindowPosition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left-half" | "left" => Ok(WindowPosition::LeftHalf),
            "right-half" | "right" => Ok(WindowPosition::RightHalf),
            "top-half" | "top" => Ok(WindowPosition::TopHalf),
            "bottom-half" | "bottom" => Ok(WindowPosition::BottomHalf),
            "top-left-quarter" | "top-left" => Ok(WindowPosition::TopLeftQuarter),
            "top-right-quarter" | "top-right" => Ok(WindowPosition::TopRightQuarter),
            "bottom-left-quarter" | "bottom-left" => Ok(WindowPosition::BottomLeftQuarter),
            "bottom-right-quarter" | "bottom-right" => Ok(WindowPosition::BottomRightQuarter),
            "left-third" => Ok(WindowPosition::LeftThird),
            "center-third" => Ok(WindowPosition::CenterThird),
            "right-third" => Ok(WindowPosition::RightThird),
            "center" => Ok(WindowPosition::Center),
            "maximize" | "max" => Ok(WindowPosition::Maximize),
            "almost-maximize" | "almost-max" => Ok(WindowPosition::AlmostMaximize),
            _ => Err(format!("Unknown window position: {}", s)),
        }
    }
}

/// Result of a notification being shown.
pub type NotifyResult = Result<(), String>;

/// Result of executing a system command.
pub type CommandResult = Result<(), String>;

/// Platform-specific operations trait.
///
/// Implementations of this trait provide OS-specific functionality for:
/// - Application discovery
/// - Clipboard operations
/// - Opening URLs and files
/// - Desktop notifications
/// - System commands (lock, sleep, shutdown, etc.)
/// - Configuration and data directory paths
pub trait Platform: Send + Sync {
    /// Discover installed applications on the system.
    ///
    /// Returns a list of applications that can be launched.
    /// The implementation should scan platform-specific locations:
    /// - Linux: XDG .desktop files
    /// - macOS: /Applications, ~/Applications
    /// - Windows: Start Menu, Registry App Paths
    fn discover_apps(&self) -> Vec<AppEntry>;

    /// Read the current clipboard content as text.
    ///
    /// Returns `None` if the clipboard is empty or contains non-text data.
    fn clipboard_read(&self) -> Option<String>;

    /// Write text to the clipboard.
    fn clipboard_write(&self, content: &str) -> Result<(), String>;

    /// Open a URL in the default browser.
    fn open_url(&self, url: &str) -> Result<(), String>;

    /// Open a file or directory with the default application.
    fn open_file(&self, path: &str) -> Result<(), String>;

    /// Show a desktop notification.
    fn show_notification(&self, title: &str, body: &str) -> NotifyResult;

    /// Execute a system command (lock, sleep, logout, restart, shutdown).
    fn system_command(&self, command: SystemCommand) -> CommandResult;

    /// Get the platform-specific configuration directory.
    ///
    /// - Linux: `~/.config/nova/`
    /// - macOS: `~/Library/Application Support/nova/`
    /// - Windows: `%APPDATA%\nova\`
    fn config_dir(&self) -> PathBuf;

    /// Get the platform-specific data directory.
    ///
    /// - Linux: `~/.local/share/nova/`
    /// - macOS: `~/Library/Application Support/nova/`
    /// - Windows: `%LOCALAPPDATA%\nova\`
    fn data_dir(&self) -> PathBuf;

    /// Get the platform-specific runtime directory (for sockets, etc.).
    ///
    /// - Linux: `$XDG_RUNTIME_DIR` or `/tmp`
    /// - macOS: `$TMPDIR` or `/tmp`
    /// - Windows: `%TEMP%`
    fn runtime_dir(&self) -> PathBuf;

    /// Launch an application by its exec command.
    ///
    /// The `exec` string may contain field codes (like `%f`, `%u`) that
    /// should be stripped or handled appropriately.
    fn launch_app(&self, app: &AppEntry) -> Result<(), String>;

    /// Run an arbitrary shell command.
    fn run_shell_command(&self, command: &str) -> Result<(), String>;

    // ==================== Window Management ====================

    /// Get information about the currently focused window.
    ///
    /// Returns `Err` if there is no focused window or if window
    /// management is not supported on this platform.
    fn get_focused_window(&self) -> Result<WindowInfo, String> {
        Err("Window management not supported on this platform".to_string())
    }

    /// List all visible windows on all screens.
    ///
    /// Returns an empty Vec if window management is not supported.
    fn list_windows(&self) -> Result<Vec<WindowInfo>, String> {
        Ok(Vec::new())
    }

    /// Get information about all screens/displays.
    fn list_screens(&self) -> Result<Vec<ScreenInfo>, String> {
        Ok(Vec::new())
    }

    /// Move and resize a window to the given frame.
    fn set_window_frame(&self, _window_id: u64, _frame: WindowFrame) -> Result<(), String> {
        Err("Window management not supported on this platform".to_string())
    }

    /// Apply a preset window position to a window.
    ///
    /// The window will be moved to the screen it's currently on.
    fn set_window_position(&self, window_id: u64, position: WindowPosition) -> Result<(), String> {
        // Default implementation: get window's screen, calculate frame, apply
        let window = self.get_focused_window()?;
        let screens = self.list_screens()?;

        // Find screen that contains the window center
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

    /// Check if window management is supported on this platform.
    fn window_management_supported(&self) -> bool {
        false
    }
}

/// Get the platform implementation for the current OS.
pub fn current() -> Box<dyn Platform> {
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxPlatform::new())
    }

    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOSPlatform::new())
    }

    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsPlatform::new())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        compile_error!("Unsupported platform")
    }
}

/// Platform name as a string (for logging/display).
pub fn name() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        "Linux"
    }

    #[cfg(target_os = "macos")]
    {
        "macOS"
    }

    #[cfg(target_os = "windows")]
    {
        "Windows"
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "Unknown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    /// Get platform instance for testing.
    fn get_platform() -> Box<dyn Platform> {
        current()
    }

    // ==================== App Discovery Tests ====================

    #[test]
    fn test_discover_apps_returns_valid_entries() {
        let platform = get_platform();
        let apps = platform.discover_apps();

        // Should discover at least some applications
        assert!(!apps.is_empty(), "Should discover at least one application");

        // Each app should have required fields
        for app in &apps {
            assert!(
                !app.id.is_empty(),
                "App ID should not be empty for app: {:?}",
                app
            );
            assert!(
                !app.name.is_empty(),
                "App name should not be empty for app with ID: {} exec: {}",
                app.id,
                app.exec
            );
            assert!(
                !app.exec.is_empty(),
                "App exec should not be empty for app: {}",
                app.name
            );
        }
    }

    #[test]
    fn test_discover_apps_no_duplicates() {
        let platform = get_platform();
        let apps = platform.discover_apps();

        let mut seen_ids = std::collections::HashSet::new();
        for app in &apps {
            assert!(
                seen_ids.insert(&app.id),
                "Duplicate app ID found: {}",
                app.id
            );
        }
    }

    #[test]
    fn test_discover_apps_has_keywords() {
        let platform = get_platform();
        let apps = platform.discover_apps();

        // At least some apps should have keywords
        let apps_with_keywords = apps.iter().filter(|a| !a.keywords.is_empty()).count();
        assert!(
            apps_with_keywords > 0,
            "At least some apps should have keywords"
        );
    }

    // ==================== Directory Tests ====================

    #[test]
    fn test_config_dir_is_valid() {
        let platform = get_platform();
        let config_dir = platform.config_dir();

        // Should end with 'nova'
        assert!(
            config_dir.ends_with("nova"),
            "Config dir should end with 'nova': {:?}",
            config_dir
        );

        // Parent directory should exist or be creatable
        if let Some(parent) = config_dir.parent() {
            // Parent should either exist or be a valid path
            assert!(
                parent.to_str().is_some(),
                "Config dir parent should be a valid path"
            );
        }
    }

    #[test]
    fn test_data_dir_is_valid() {
        let platform = get_platform();
        let data_dir = platform.data_dir();

        // Should end with 'nova'
        assert!(
            data_dir.ends_with("nova"),
            "Data dir should end with 'nova': {:?}",
            data_dir
        );
    }

    #[test]
    fn test_runtime_dir_exists() {
        let platform = get_platform();
        let runtime_dir = platform.runtime_dir();

        // Runtime dir should exist (it's typically /tmp or similar)
        assert!(
            runtime_dir.exists(),
            "Runtime dir should exist: {:?}",
            runtime_dir
        );
    }

    // ==================== Clipboard Tests ====================
    // Note: These tests may fail in CI environments without clipboard access

    #[test]
    #[ignore] // Ignore by default as it requires clipboard access
    fn test_clipboard_write_and_read() {
        let platform = get_platform();

        // Write a unique test value
        let test_value = format!("nova_test_{}", std::process::id());
        let write_result = platform.clipboard_write(&test_value);
        assert!(write_result.is_ok(), "Clipboard write should succeed");

        // Read it back
        let read_value = platform.clipboard_read();
        assert_eq!(
            read_value,
            Some(test_value),
            "Clipboard read should return written value"
        );
    }

    #[test]
    #[ignore] // Ignore by default as it requires clipboard access
    fn test_clipboard_handles_unicode() {
        let platform = get_platform();

        let unicode_text = "Hello 世界 🎉 مرحبا";
        let write_result = platform.clipboard_write(unicode_text);
        assert!(write_result.is_ok(), "Should handle unicode in clipboard");

        let read_value = platform.clipboard_read();
        assert_eq!(read_value, Some(unicode_text.to_string()));
    }

    // ==================== Performance Tests ====================

    #[test]
    fn test_app_discovery_performance() {
        let platform = get_platform();

        let start = Instant::now();
        let apps = platform.discover_apps();
        let duration = start.elapsed();

        println!("App discovery: {} apps in {:?}", apps.len(), duration);

        // App discovery should complete within 5 seconds (generous for slow systems)
        assert!(
            duration.as_secs() < 5,
            "App discovery took too long: {:?}",
            duration
        );
    }

    #[test]
    fn test_config_dir_performance() {
        let platform = get_platform();

        let start = Instant::now();
        for _ in 0..1000 {
            let _ = platform.config_dir();
        }
        let duration = start.elapsed();

        // 1000 calls should complete within 100ms
        assert!(
            duration.as_millis() < 100,
            "config_dir is too slow: {:?} for 1000 calls",
            duration
        );
    }

    // ==================== Platform-Specific Behavior Tests ====================

    #[test]
    fn test_platform_name_matches_os() {
        let platform_name = name();

        #[cfg(target_os = "linux")]
        assert_eq!(platform_name, "Linux");

        #[cfg(target_os = "macos")]
        assert_eq!(platform_name, "macOS");

        #[cfg(target_os = "windows")]
        assert_eq!(platform_name, "Windows");
    }

    #[test]
    fn test_app_entry_debug_format() {
        let app = AppEntry {
            id: "test-app".to_string(),
            name: "Test App".to_string(),
            exec: "/usr/bin/test".to_string(),
            icon: Some("/path/to/icon.png".to_string()),
            description: Some("A test application".to_string()),
            keywords: vec!["test".to_string(), "app".to_string()],
        };

        // Should be debuggable without panic
        let debug_str = format!("{:?}", app);
        assert!(debug_str.contains("Test App"));
    }

    #[test]
    fn test_system_command_enum() {
        // Ensure all variants exist and are comparable
        assert_eq!(SystemCommand::Lock, SystemCommand::Lock);
        assert_ne!(SystemCommand::Lock, SystemCommand::Sleep);
        assert_ne!(SystemCommand::Sleep, SystemCommand::Logout);
        assert_ne!(SystemCommand::Restart, SystemCommand::Shutdown);
    }
}
