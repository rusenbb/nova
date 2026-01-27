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
