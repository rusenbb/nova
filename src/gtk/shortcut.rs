//! Keyboard shortcut configuration for GTK/GNOME.

use crate::config;
use std::env;
use std::process::Command;

pub fn get_nova_binary_path() -> String {
    env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "nova".to_string())
}

fn set_shortcut_quiet(shortcut: &str) -> Result<(), String> {
    set_shortcut_impl(shortcut, false)
}

pub fn set_shortcut(shortcut: &str) -> Result<(), String> {
    set_shortcut_impl(shortcut, true)
}

fn set_shortcut_impl(shortcut: &str, verbose: bool) -> Result<(), String> {
    let nova_path = get_nova_binary_path();

    // GNOME custom keybindings use a path-based schema
    let binding_path = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/nova/";

    // First, add our binding to the list of custom keybindings
    let output = Command::new("gsettings")
        .args([
            "get",
            "org.gnome.settings-daemon.plugins.media-keys",
            "custom-keybindings",
        ])
        .output()
        .map_err(|e| format!("Failed to get current keybindings: {}", e))?;

    let current = String::from_utf8_lossy(&output.stdout);
    let current = current.trim();

    // Check if our binding is already in the list
    if !current.contains(binding_path) {
        let new_list = if current == "@as []" || current.is_empty() {
            format!("['{}']", binding_path)
        } else {
            // Remove trailing ] and add our path
            let trimmed = current.trim_end_matches(']');
            format!("{}, '{}']", trimmed, binding_path)
        };

        Command::new("gsettings")
            .args([
                "set",
                "org.gnome.settings-daemon.plugins.media-keys",
                "custom-keybindings",
                &new_list,
            ])
            .status()
            .map_err(|e| format!("Failed to update keybindings list: {}", e))?;
    }

    // Set the custom keybinding properties
    let schema = "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding";
    let schema_path = format!("{}:{}", schema, binding_path);

    Command::new("gsettings")
        .args(["set", &schema_path, "name", "Nova Launcher"])
        .status()
        .map_err(|e| format!("Failed to set name: {}", e))?;

    Command::new("gsettings")
        .args(["set", &schema_path, "command", &nova_path])
        .status()
        .map_err(|e| format!("Failed to set command: {}", e))?;

    Command::new("gsettings")
        .args(["set", &schema_path, "binding", shortcut])
        .status()
        .map_err(|e| format!("Failed to set binding: {}", e))?;

    if verbose {
        println!("[Nova] Shortcut set to: {}", shortcut);
        println!("[Nova] Command: {}", nova_path);
        println!("\nCommon shortcuts:");
        println!("  <Super>space     - Super+Space (may conflict with GNOME)");
        println!("  <Alt>space       - Alt+Space (recommended)");
        println!("  <Control>space   - Ctrl+Space");
        println!("  <Super><Alt>n    - Super+Alt+N");
    } else {
        println!("[Nova] Configured shortcut: {}", shortcut);
    }

    Ok(())
}

pub fn print_help() {
    println!("Nova - Keyboard-driven productivity launcher");
    println!();
    println!("USAGE:");
    println!("    nova                              Start Nova (or toggle if already running)");
    println!("    nova --settings                   Open settings window");
    println!("    nova --set-shortcut KEY           Set the global keyboard shortcut");
    println!("    nova --help                       Show this help message");
    println!();
    println!("EXTENSION DEVELOPMENT:");
    println!("    nova create extension NAME        Create a new extension");
    println!("    nova dev [PATH]                   Run extension with hot reload");
    println!("    nova build [PATH]                 Build extension for distribution");
    println!("    nova install SOURCE               Install extension from source");
    println!();
    println!("SHORTCUT FORMAT:");
    println!("    <Super>space     - Super+Space");
    println!("    <Alt>space       - Alt+Space (recommended, no GNOME conflicts)");
    println!("    <Control>space   - Ctrl+Space");
    println!("    <Super><Alt>n    - Super+Alt+N");
    println!();
    println!("EXAMPLES:");
    println!("    nova --set-shortcut '<Alt>space'");
    println!("    nova create extension my-extension");
}

/// Ensure the keyboard shortcut is configured in GNOME
pub fn ensure_shortcut_configured() {
    let cfg = config::Config::load();
    let hotkey = &cfg.general.hotkey;
    let nova_path = get_nova_binary_path();

    // Check if our binding already exists with correct shortcut and command
    let binding_path = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/nova/";
    let schema = "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding";
    let schema_path = format!("{}:{}", schema, binding_path);

    // Check current binding
    let current_binding = Command::new("gsettings")
        .args(["get", &schema_path, "binding"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    let current_binding = current_binding.trim().trim_matches('\'');

    // Check current command path
    let current_command = Command::new("gsettings")
        .args(["get", &schema_path, "command"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    let current_command = current_command.trim().trim_matches('\'');

    // Only skip if BOTH binding and command are correct
    if current_binding == hotkey && current_command == nova_path {
        return;
    }

    // Configure the shortcut silently
    if let Err(e) = set_shortcut_quiet(hotkey) {
        eprintln!("[Nova] Warning: Could not set keyboard shortcut: {}", e);
        eprintln!(
            "[Nova] You may need to configure it manually in GNOME Settings > Keyboard > Shortcuts"
        );
    }
}
