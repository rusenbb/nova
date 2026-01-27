mod config;
mod core;
mod error;
mod executor;
mod platform;
mod search;
mod services;
mod settings;

use gdk::prelude::*;
use gdk::Screen;
use glib::ControlFlow;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, CssProvider, Entry, EventBox, Label, ListBox, ListBoxRow,
    Orientation, StyleContext,
};
use services::AppIndex;
use std::cell::RefCell;
use std::env;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

const APP_ID: &str = "com.rusen.nova";

use core::search::SearchResult;
use executor::{ExecutionAction, SystemCommand};
use platform::AppEntry;
use services::{CustomCommandsIndex, Extension, ExtensionIndex, ExtensionKind, ScriptOutputMode};

/// Represents the current command mode state
#[derive(Debug, Clone, Default)]
struct CommandModeState {
    /// The active extension (if in command mode)
    active_extension: Option<Extension>,
}

impl CommandModeState {
    fn enter_mode(&mut self, extension: Extension) {
        self.active_extension = Some(extension);
    }

    fn exit_mode(&mut self) {
        self.active_extension = None;
    }

    fn is_active(&self) -> bool {
        self.active_extension.is_some()
    }
}

/// Consolidated UI state to reduce RefCell sprawl
///
/// This struct holds all mutable UI state that needs to be shared across
/// GTK event handlers. By consolidating state here, we reduce the number
/// of individual Rc<RefCell<T>> variables from 15+ to just one.
struct UIState {
    // Widget references (GTK widgets are internally ref-counted, so Clone is cheap)
    window: ApplicationWindow,
    entry: Entry,
    results_list: ListBox,
    command_pill: Label,

    // Mutable state
    is_visible: bool,
    selected_index: i32,
    current_results: Vec<SearchResult>,
    command_mode: CommandModeState,
    is_clearing: bool, // Guard to prevent callback loops when programmatically clearing entry
    last_toggle: Instant,
}

impl UIState {
    /// Update the results list with new search results
    fn update_results(&mut self, results: Vec<SearchResult>) {
        render_results_list(&self.results_list, &results);
        self.current_results = results;
        self.selected_index = 0;
        if let Some(row) = self.results_list.row_at_index(0) {
            self.results_list.select_row(Some(&row));
        }
    }

    /// Clear entry text with the is_clearing guard to prevent callback loops
    fn clear_entry(&mut self) {
        self.is_clearing = true;
        self.entry.set_text("");
        self.is_clearing = false;
    }

    /// Update command pill visibility and text based on command mode state
    fn update_command_pill(&self) {
        if let Some(ref ext) = self.command_mode.active_extension {
            self.command_pill.set_text(ext.pill_text());
            self.command_pill.set_visible(true);
            self.entry
                .set_placeholder_text(Some(&format!("Search {}...", ext.name)));
        } else {
            self.command_pill.set_visible(false);
            self.entry.set_placeholder_text(Some("Search apps..."));
        }
    }

    /// Enter command mode for the given extension
    fn enter_command_mode(&mut self, extension: Extension) {
        self.command_mode.enter_mode(extension);
        self.clear_entry();
        self.update_command_pill();
    }

    /// Exit command mode and reset to normal search
    fn exit_command_mode(&mut self) {
        self.command_mode.exit_mode();
        self.update_command_pill();
    }

    /// Navigate selection up/down
    fn navigate_selection(&mut self, delta: i32) {
        let n_items = self.results_list.children().len() as i32;
        if n_items > 0 {
            self.selected_index = (self.selected_index + delta).rem_euclid(n_items);
            if let Some(row) = self.results_list.row_at_index(self.selected_index) {
                self.results_list.select_row(Some(&row));
            }
        }
    }

    /// Get the currently selected search result
    fn selected_result(&self) -> Option<&SearchResult> {
        self.current_results.get(self.selected_index as usize)
    }

    /// Hide the window, reset state, and optionally save config
    fn hide_window(&mut self, config: &config::Config) {
        // Exit command mode if active
        self.exit_command_mode();
        self.clear_entry();
        self.window.hide();
        self.is_visible = false;
        // Save config position
        if let Err(e) = config.save() {
            eprintln!("[Nova] Failed to save config: {}", e);
        }
    }

    /// Show the window, position it, and prepare for input
    fn show_window(&mut self, config: &config::Config) {
        // Ensure command mode is reset
        self.exit_command_mode();

        // Position window (use saved position or center)
        position_window(&self.window, config);

        // Show and present
        self.window.show_all();
        self.window.present_with_time(0);

        // Ensure focus goes to entry
        self.entry.set_text("");
        self.entry.grab_focus();

        // Force the window to be active
        if let Some(gdk_window) = self.window.window() {
            gdk_window.focus(0);
            gdk_window.raise();
        }

        self.is_visible = true;
        self.last_toggle = Instant::now();
    }
}

/// Type alias for the shared UI state handle
type UIStateHandle = Rc<RefCell<UIState>>;

/// Convert a services::AppEntry to platform::AppEntry
fn to_platform_app(app: &services::AppEntry) -> AppEntry {
    AppEntry {
        id: app.id.clone(),
        name: app.name.clone(),
        exec: app.exec.clone(),
        icon: app.icon.clone(),
        description: app.description.clone(),
        keywords: app.keywords.clone(),
    }
}

/// Get the action to perform when a search result is executed
fn result_to_action(result: &SearchResult) -> ExecutionAction {
    match result {
        SearchResult::App(app) => ExecutionAction::LaunchApp { app: app.clone() },
        SearchResult::Command { id, .. } => match id.as_str() {
            "nova:settings" => ExecutionAction::OpenSettings,
            "nova:quit" => ExecutionAction::Quit,
            "system:lock" => ExecutionAction::SystemCommand {
                command: SystemCommand::Lock,
            },
            "system:sleep" => ExecutionAction::SystemCommand {
                command: SystemCommand::Sleep,
            },
            "system:logout" => ExecutionAction::SystemCommand {
                command: SystemCommand::Logout,
            },
            "system:restart" => ExecutionAction::SystemCommand {
                command: SystemCommand::Restart,
            },
            "system:shutdown" => ExecutionAction::SystemCommand {
                command: SystemCommand::Shutdown,
            },
            _ => ExecutionAction::NeedsInput,
        },
        SearchResult::Alias { target, .. } => ExecutionAction::RunShellCommand {
            command: target.clone(),
        },
        SearchResult::Quicklink { url, has_query, .. } => {
            if *has_query {
                ExecutionAction::NeedsInput
            } else {
                ExecutionAction::OpenUrl { url: url.clone() }
            }
        }
        SearchResult::QuicklinkWithQuery { resolved_url, .. } => ExecutionAction::OpenUrl {
            url: resolved_url.clone(),
        },
        SearchResult::Script {
            path,
            has_argument,
            output_mode,
            ..
        } => {
            if *has_argument {
                ExecutionAction::NeedsInput
            } else {
                ExecutionAction::RunScript {
                    path: path.clone(),
                    argument: None,
                    output_mode: output_mode.clone(),
                }
            }
        }
        SearchResult::ScriptWithArgument {
            path,
            argument,
            output_mode,
            ..
        } => ExecutionAction::RunScript {
            path: path.clone(),
            argument: Some(argument.clone()),
            output_mode: output_mode.clone(),
        },
        SearchResult::ExtensionCommand { command } => {
            if command.has_argument {
                ExecutionAction::NeedsInput
            } else {
                ExecutionAction::RunExtensionCommand {
                    command: command.clone(),
                    argument: None,
                }
            }
        }
        SearchResult::ExtensionCommandWithArg { command, argument } => {
            ExecutionAction::RunExtensionCommand {
                command: command.clone(),
                argument: Some(argument.clone()),
            }
        }
        SearchResult::DenoCommand {
            extension_id,
            command_id,
            ..
        } => ExecutionAction::RunDenoCommand {
            extension_id: extension_id.clone(),
            command_id: command_id.clone(),
            argument: None,
        },
        SearchResult::DenoCommandWithArg {
            extension_id,
            command_id,
            argument,
            ..
        } => ExecutionAction::RunDenoCommand {
            extension_id: extension_id.clone(),
            command_id: command_id.clone(),
            argument: Some(argument.clone()),
        },
        SearchResult::Calculation { result, expression } => {
            let value = result.trim_start_matches("= ");
            ExecutionAction::CopyToClipboard {
                content: value.to_string(),
                notification: format!("{} = {}", expression, value),
            }
        }
        SearchResult::ClipboardItem {
            content, preview, ..
        } => ExecutionAction::CopyToClipboard {
            content: content.clone(),
            notification: preview.clone(),
        },
        SearchResult::FileResult { path, .. } => {
            let full_path = if path.starts_with("~/") {
                dirs::home_dir()
                    .map(|h| format!("{}{}", h.display(), &path[1..]))
                    .unwrap_or_else(|| path.clone())
            } else {
                path.clone()
            };
            ExecutionAction::OpenFile { path: full_path }
        }
        SearchResult::EmojiResult { emoji, name, .. } => ExecutionAction::CopyToClipboard {
            content: emoji.clone(),
            notification: name.clone(),
        },
        SearchResult::UnitConversion { result, display } => ExecutionAction::CopyToClipboard {
            content: result.clone(),
            notification: display.clone(),
        },
    }
}

fn get_system_commands() -> Vec<SearchResult> {
    vec![
        // Nova commands
        SearchResult::Command {
            id: "nova:settings".to_string(),
            name: "Settings".to_string(),
            description: "Open Nova settings".to_string(),
        },
        SearchResult::Command {
            id: "nova:quit".to_string(),
            name: "Quit Nova".to_string(),
            description: "Close Nova completely".to_string(),
        },
        // System commands
        SearchResult::Command {
            id: "system:lock".to_string(),
            name: "Lock Screen".to_string(),
            description: "Lock the screen".to_string(),
        },
        SearchResult::Command {
            id: "system:sleep".to_string(),
            name: "Sleep".to_string(),
            description: "Put computer to sleep".to_string(),
        },
        SearchResult::Command {
            id: "system:logout".to_string(),
            name: "Log Out".to_string(),
            description: "Log out of current session".to_string(),
        },
        SearchResult::Command {
            id: "system:restart".to_string(),
            name: "Restart".to_string(),
            description: "Restart the computer".to_string(),
        },
        SearchResult::Command {
            id: "system:shutdown".to_string(),
            name: "Shut Down".to_string(),
            description: "Shut down the computer".to_string(),
        },
    ]
}

fn search_with_commands(
    app_index: &services::AppIndex,
    custom_commands: &CustomCommandsIndex,
    extension_manager: &services::ExtensionManager,
    clipboard_history: &services::clipboard::ClipboardHistory,
    query: &str,
    max_results: usize,
) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    // Split query into keyword and remaining text (e.g., "ghs react hooks" -> "ghs", "react hooks")
    let query_parts: Vec<&str> = query.splitn(2, ' ').collect();
    let keyword = query_parts[0].to_lowercase();
    let remaining_query = query_parts.get(1).map(|s| s.to_string());

    // 1. Check for exact alias match (highest priority)
    for alias in &custom_commands.aliases {
        if alias.keyword.to_lowercase() == keyword {
            results.push(SearchResult::Alias {
                keyword: alias.keyword.clone(),
                name: alias.name.clone(),
                target: alias.target.clone(),
            });
        } else if alias.keyword.to_lowercase().contains(&query_lower)
            || alias.name.to_lowercase().contains(&query_lower)
        {
            results.push(SearchResult::Alias {
                keyword: alias.keyword.clone(),
                name: alias.name.clone(),
                target: alias.target.clone(),
            });
        }
    }

    // 1.5. Calculator - try to evaluate as math expression
    if let Some(calc_result) = services::calculator::evaluate(query) {
        let formatted = services::calculator::format_result(calc_result);
        results.push(SearchResult::Calculation {
            expression: query.to_string(),
            result: format!("= {}", formatted),
        });
    }

    // 1.5.1. Unit converter - try to parse as unit conversion
    if query.contains(" to ") {
        if let Some(conversion) = services::units::convert(query) {
            results.push(SearchResult::UnitConversion {
                display: conversion.display(),
                result: conversion.result(),
            });
        }
    }

    // 1.6. Clipboard history - trigger on "clip", "clipboard", "paste", "history"
    let clipboard_keywords = ["clip", "clipboard", "paste", "history"];
    if clipboard_keywords
        .iter()
        .any(|kw| query_lower.starts_with(kw))
    {
        // Extract optional filter after the keyword
        let filter = query_parts.get(1).map(|s| s.to_lowercase());

        let items = if let Some(ref f) = filter {
            clipboard_history.search(f)
        } else {
            clipboard_history.all()
        };

        for (idx, entry) in items.iter().take(10).enumerate() {
            results.push(SearchResult::ClipboardItem {
                index: idx,
                content: entry.content.clone(),
                preview: entry.preview(60),
                time_ago: entry.time_ago(),
            });
        }
    }

    // 1.7. File search - trigger on ~ or / prefix
    if query.starts_with('~') || query.starts_with('/') {
        for entry in services::file_search::search_files(query, 10) {
            let icon_prefix = if entry.is_dir { "[D] " } else { "" };
            results.push(SearchResult::FileResult {
                name: format!("{}{}", icon_prefix, entry.display_name()),
                path: entry.display_path(),
                is_dir: entry.is_dir,
            });
        }
    }

    // 1.8. Emoji picker - trigger on : prefix
    if query.starts_with(':') && query.len() > 1 {
        let emoji_query = &query[1..]; // Strip the : prefix
        for emoji in services::emoji::search(emoji_query, 10) {
            results.push(SearchResult::EmojiResult {
                emoji: emoji.char.to_string(),
                name: format!("{} {}", emoji.char, emoji.name()),
                aliases: emoji.aliases(),
            });
        }
    }

    // 2. Check for quicklink matches
    for quicklink in &custom_commands.quicklinks {
        let ql_keyword = quicklink.keyword.to_lowercase();

        if ql_keyword == keyword {
            // Exact keyword match
            if quicklink.has_query_placeholder() {
                if let Some(ref q) = remaining_query {
                    // User provided a query after keyword
                    results.push(SearchResult::QuicklinkWithQuery {
                        keyword: quicklink.keyword.clone(),
                        name: format!("{}: {}", quicklink.name, q),
                        url: quicklink.url.clone(),
                        query: q.clone(),
                        resolved_url: quicklink.resolve_url(q),
                    });
                } else {
                    // Show as hint that query is expected
                    results.push(SearchResult::Quicklink {
                        keyword: quicklink.keyword.clone(),
                        name: format!("{} (type to search)", quicklink.name),
                        url: quicklink.url.clone(),
                        has_query: true,
                    });
                }
            } else {
                // Simple quicklink (no query)
                results.push(SearchResult::Quicklink {
                    keyword: quicklink.keyword.clone(),
                    name: quicklink.name.clone(),
                    url: quicklink.url.clone(),
                    has_query: false,
                });
            }
        } else if ql_keyword.starts_with(&keyword)
            || quicklink.name.to_lowercase().contains(&query_lower)
        {
            // Partial match - show as suggestion
            results.push(SearchResult::Quicklink {
                keyword: quicklink.keyword.clone(),
                name: quicklink.name.clone(),
                url: quicklink.url.clone(),
                has_query: quicklink.has_query_placeholder(),
            });
        }
    }

    // 3. Search scripts
    for script in &custom_commands.scripts {
        let matches = script.name.to_lowercase().contains(&query_lower)
            || script.id.to_lowercase().contains(&query_lower)
            || script
                .keywords
                .iter()
                .any(|k| k.to_lowercase().contains(&query_lower));

        if matches {
            if script.has_argument {
                if let Some(ref arg) = remaining_query {
                    results.push(SearchResult::ScriptWithArgument {
                        id: script.id.clone(),
                        name: format!("{}: {}", script.name, arg),
                        description: script.description.clone(),
                        path: script.path.clone(),
                        argument: arg.clone(),
                        output_mode: script.output_mode.clone(),
                    });
                } else {
                    results.push(SearchResult::Script {
                        id: script.id.clone(),
                        name: format!("{} (type argument)", script.name),
                        description: script.description.clone(),
                        path: script.path.clone(),
                        has_argument: true,
                        output_mode: script.output_mode.clone(),
                    });
                }
            } else {
                results.push(SearchResult::Script {
                    id: script.id.clone(),
                    name: script.name.clone(),
                    description: script.description.clone(),
                    path: script.path.clone(),
                    has_argument: false,
                    output_mode: script.output_mode.clone(),
                });
            }
        }
    }

    // 4. System commands
    for cmd in get_system_commands() {
        if cmd.name().to_lowercase().contains(&query_lower)
            || cmd
                .description()
                .map(|d| d.to_lowercase().contains(&query_lower))
                .unwrap_or(false)
        {
            results.push(cmd);
        }
    }

    // 5. Extension commands (before apps so they're not truncated)
    for cmd in extension_manager.search_commands(&query_lower) {
        let cmd_keyword = cmd.keyword.to_lowercase();

        if cmd_keyword == keyword {
            // Exact keyword match
            if cmd.has_argument {
                if let Some(ref arg) = remaining_query {
                    results.push(SearchResult::ExtensionCommandWithArg {
                        command: cmd.clone(),
                        argument: arg.clone(),
                    });
                } else {
                    results.push(SearchResult::ExtensionCommand {
                        command: cmd.clone(),
                    });
                }
            } else {
                results.push(SearchResult::ExtensionCommand {
                    command: cmd.clone(),
                });
            }
        } else if cmd_keyword.starts_with(&keyword)
            || cmd.name.to_lowercase().contains(&query_lower)
        {
            results.push(SearchResult::ExtensionCommand {
                command: cmd.clone(),
            });
        }
    }

    // 6. App results (last since there are many)
    for app in app_index.search(query) {
        results.push(SearchResult::App(to_platform_app(app)));
    }

    // Limit total results
    results.truncate(max_results);
    results
}

/// Search within a specific command mode context
fn search_in_command_mode(
    mode_state: &CommandModeState,
    query: &str,
    _max_results: usize,
) -> Vec<SearchResult> {
    let Some(ref ext) = mode_state.active_extension else {
        return Vec::new();
    };

    match &ext.kind {
        ExtensionKind::Quicklink { url, .. } => {
            if query.is_empty() {
                // Show hint when no query entered yet
                vec![SearchResult::Quicklink {
                    keyword: ext.keyword.clone(),
                    name: format!("Type to search {}", ext.name),
                    url: url.clone(),
                    has_query: true,
                }]
            } else {
                // Show resolved result with query
                let resolved = url.replace("{query}", &urlencoding::encode(query));
                vec![SearchResult::QuicklinkWithQuery {
                    keyword: ext.keyword.clone(),
                    name: format!("{}: {}", ext.name, query),
                    url: url.clone(),
                    query: query.to_string(),
                    resolved_url: resolved,
                }]
            }
        }
        ExtensionKind::Script {
            path,
            output_mode,
            description,
            ..
        } => {
            if query.is_empty() {
                vec![SearchResult::Script {
                    id: ext.keyword.clone(),
                    name: format!("{} (type argument)", ext.name),
                    description: description.clone(),
                    path: path.clone(),
                    has_argument: true,
                    output_mode: output_mode.clone(),
                }]
            } else {
                vec![SearchResult::ScriptWithArgument {
                    id: ext.keyword.clone(),
                    name: format!("{}: {}", ext.name, query),
                    description: description.clone(),
                    path: path.clone(),
                    argument: query.to_string(),
                    output_mode: output_mode.clone(),
                }]
            }
        }
        ExtensionKind::Alias { target } => {
            // Aliases don't take queries, just show the alias
            vec![SearchResult::Alias {
                keyword: ext.keyword.clone(),
                name: ext.name.clone(),
                target: target.clone(),
            }]
        }
    }
}

// Execution helpers
fn open_url(url: &str) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map_err(|e| format!("Failed to open URL: {}", e))?;
    Ok(())
}

fn execute_script(
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
fn execute_extension_command(
    extension_manager: &Rc<services::ExtensionManager>,
    command: &services::LoadedCommand,
    argument: Option<&String>,
) -> Result<(), String> {
    use services::OutputMode;

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

fn show_notification(title: &str, body: &str) -> Result<(), String> {
    Command::new("notify-send")
        .args([title, body])
        .spawn()
        .map_err(|e| format!("Failed to show notification: {}", e))?;
    Ok(())
}

fn copy_to_clipboard(content: &str) -> Result<(), String> {
    use std::io::Write;
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

fn get_socket_path() -> PathBuf {
    let runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(runtime_dir).join("nova.sock")
}

fn try_send_toggle() -> bool {
    let socket_path = get_socket_path();
    if let Ok(mut stream) = UnixStream::connect(&socket_path) {
        let _ = stream.write_all(b"toggle");
        let mut response = [0u8; 2];
        let _ = stream.read_exact(&mut response);
        return true;
    }
    false
}

fn get_nova_binary_path() -> String {
    env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "nova".to_string())
}

fn set_shortcut_quiet(shortcut: &str) -> Result<(), String> {
    set_shortcut_impl(shortcut, false)
}

fn set_shortcut(shortcut: &str) -> Result<(), String> {
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

fn print_help() {
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

fn main() {
    // Try CLI commands first (create, dev, build, install)
    match nova::cli::run() {
        Ok(true) => return, // CLI handled the command
        Ok(false) => {}     // No CLI command, continue to GTK
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    let args: Vec<String> = env::args().collect();

    // Handle legacy CLI arguments (--settings, --set-shortcut)
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_help();
                return;
            }
            "--settings" => {
                let app = Application::builder()
                    .application_id(&format!("{}.settings", APP_ID))
                    .build();

                app.connect_activate(|app| {
                    settings::show_settings_window(app);
                });

                // Pass empty args to avoid GTK parsing our custom args
                app.run_with_args(&[] as &[&str]);
                return;
            }
            "--set-shortcut" => {
                if args.len() < 3 {
                    eprintln!("Error: --set-shortcut requires a shortcut argument");
                    eprintln!("Example: nova --set-shortcut '<Alt>space'");
                    std::process::exit(1);
                }
                match set_shortcut(&args[2]) {
                    Ok(()) => return,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            arg if arg.starts_with('-') => {
                // Unknown flag - clap already handled valid subcommands
                eprintln!("Unknown argument: {}", args[1]);
                eprintln!("Run 'nova --help' for usage information");
                std::process::exit(1);
            }
            _ => {
                // Positional arg without subcommand - show help
                eprintln!("Unknown command: {}", args[1]);
                eprintln!("Run 'nova --help' for usage information");
                std::process::exit(1);
            }
        }
    }

    if try_send_toggle() {
        println!("[Nova] Sent toggle to existing instance");
        return;
    }

    // Ensure keyboard shortcut is configured on startup
    ensure_shortcut_configured();

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);
    app.run();
}

/// Ensure the keyboard shortcut is configured in GNOME
fn ensure_shortcut_configured() {
    let config = config::Config::load();
    let hotkey = &config.general.hotkey;
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
fn build_ui(app: &Application) {
    // Load config (stored in Rc<RefCell> for runtime updates like position)
    let config = Rc::new(RefCell::new(config::Config::load()));
    let max_results = config.borrow().behavior.max_results as usize;

    // Ensure autostart state matches config
    if let Err(e) = config::set_autostart(config.borrow().behavior.autostart) {
        eprintln!("[Nova] Failed to set autostart: {}", e);
    }

    // Initialize app index, custom commands, extensions, and clipboard history
    let app_index = Rc::new(AppIndex::new());
    let custom_commands = Rc::new(RefCell::new(CustomCommandsIndex::new(&config.borrow())));
    let extension_manager = Rc::new(services::ExtensionManager::load(
        &services::get_extensions_dir(),
    ));
    let clipboard_history = Rc::new(RefCell::new(services::clipboard::ClipboardHistory::new(50)));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Nova")
        .default_width(600)
        .default_height(400)
        .decorated(false)
        .resizable(false)
        .build();

    // Set RGBA visual for transparency
    if let Some(screen) = WidgetExt::screen(&window) {
        if let Some(visual) = screen.rgba_visual() {
            window.set_visual(Some(&visual));
        }
    }
    window.set_app_paintable(true);

    // Window manager hints for launcher behavior
    window.set_type_hint(gdk::WindowTypeHint::Dialog);
    window.set_skip_taskbar_hint(true);
    window.set_skip_pager_hint(true);
    window.set_keep_above(true);
    window.set_focus_on_map(true);
    window.set_accept_focus(true);

    // Load CSS from appearance settings
    let provider = CssProvider::new();
    let css = config::generate_css(&config.borrow().appearance);
    provider
        .load_from_data(css.as_bytes())
        .expect("Failed to load CSS");
    if let Some(screen) = Screen::default() {
        StyleContext::add_provider_for_screen(
            &screen,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );
    }

    // Main container wrapped in EventBox for drag support
    let event_box = EventBox::new();
    event_box.set_above_child(false); // Allow clicks to pass through to children

    let container = gtk::Box::new(Orientation::Vertical, 0);
    container.style_context().add_class("nova-container");

    // Command mode pill (initially hidden)
    let command_pill = Label::new(None);
    command_pill.style_context().add_class("nova-command-pill");
    command_pill.set_visible(false);
    command_pill.set_no_show_all(true);

    // Search entry
    let entry = Entry::new();
    entry.set_placeholder_text(Some("Search apps..."));
    entry.style_context().add_class("nova-entry-in-container");

    // Container for pill + entry (replaces the old nova-entry styling)
    let entry_container = gtk::Box::new(Orientation::Horizontal, 0);
    entry_container
        .style_context()
        .add_class("nova-entry-container");
    entry_container.pack_start(&command_pill, false, false, 0);
    entry_container.pack_start(&entry, true, true, 0);

    // Results list
    let results_list = ListBox::new();
    results_list.style_context().add_class("nova-results");
    results_list.set_selection_mode(gtk::SelectionMode::Single);

    container.pack_start(&entry_container, false, false, 0);
    container.pack_start(&results_list, true, true, 0);
    event_box.add(&container);
    window.add(&event_box);

    // Enable dragging the window by clicking anywhere on the container background
    let window_for_drag = window.clone();
    event_box.connect_button_press_event(move |_, event| {
        if event.button() == 1 {
            // Left click - start window drag
            window_for_drag.begin_move_drag(
                event.button() as i32,
                event.root().0 as i32,
                event.root().1 as i32,
                event.time(),
            );
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });

    // Save window position when it moves
    let config_for_configure = config.clone();
    window.connect_configure_event(move |window, _event| {
        // Get current position and save to config
        let (x, y) = window.position();
        let mut cfg = config_for_configure.borrow_mut();
        if cfg.appearance.window_x != Some(x) || cfg.appearance.window_y != Some(y) {
            cfg.appearance.window_x = Some(x);
            cfg.appearance.window_y = Some(y);
            // Save config (debounced by writing on hide instead for performance)
        }
        false
    });

    // Consolidated UI state (reduces 15+ Rc<RefCell> to just one)
    let ui_state: UIStateHandle = Rc::new(RefCell::new(UIState {
        window: window.clone(),
        entry: entry.clone(),
        results_list: results_list.clone(),
        command_pill: command_pill.clone(),
        is_visible: false,
        selected_index: 0,
        current_results: Vec::new(),
        command_mode: CommandModeState::default(),
        is_clearing: false,
        last_toggle: Instant::now(),
    }));

    // Config reference for use in handlers
    let config_ref = config.clone();

    // Extension index for fast keyword lookup
    let extension_index = Rc::new(RefCell::new(ExtensionIndex::from_custom_commands(
        &custom_commands.borrow(),
        &config.borrow().aliases,
        &config.borrow().quicklinks,
    )));

    // Update results when search text changes
    let ui_state_for_search = ui_state.clone();
    let app_index_search = app_index.clone();
    let custom_commands_search = custom_commands.clone();
    let extension_manager_search = extension_manager.clone();
    let clipboard_history_search = clipboard_history.clone();
    let extension_index_for_search = extension_index.clone();
    entry.connect_changed(move |entry| {
        let mut state = ui_state_for_search.borrow_mut();

        // Skip if we're clearing the entry programmatically (prevents RefCell conflicts)
        if state.is_clearing {
            return;
        }

        let query = entry.text().to_string();

        // Check for command mode entry: "keyword " pattern (space at end)
        if !state.command_mode.is_active() && query.ends_with(' ') && query.len() > 1 {
            let keyword = query.trim();
            if let Some(ext) = extension_index_for_search.borrow().get_by_keyword(keyword) {
                if ext.accepts_query() {
                    // Enter command mode using UIState method
                    state.enter_command_mode(ext.clone());

                    // Show empty state results for command mode
                    let results = search_in_command_mode(&state.command_mode, "", max_results);
                    state.update_results(results);
                    return;
                }
            }
        }

        // Perform search based on mode
        let results = if state.command_mode.is_active() {
            search_in_command_mode(&state.command_mode, &query, max_results)
        } else {
            search_with_commands(
                &app_index_search,
                &custom_commands_search.borrow(),
                &extension_manager_search,
                &clipboard_history_search.borrow(),
                &query,
                max_results,
            )
        };

        state.update_results(results);
    });

    // Handle keyboard events
    let ui_state_for_key = ui_state.clone();
    let app_for_key = app.clone();
    let config_for_key = config_ref.clone();
    let app_index_for_key = app_index.clone();
    let custom_commands_for_key = custom_commands.clone();
    let extension_index_for_key = extension_index.clone();
    let extension_manager_for_key = extension_manager.clone();
    let clipboard_history_for_key = clipboard_history.clone();

    entry.connect_key_press_event(move |_entry_widget, event| {
        let key = event.keyval();

        match key {
            gdk::keys::constants::Tab | gdk::keys::constants::ISO_Left_Tab => {
                let mut state = ui_state_for_key.borrow_mut();
                // Tab enters command mode for selected extension (if it accepts queries)
                if !state.command_mode.is_active() {
                    let selected_idx = state.selected_index as usize;
                    if let Some(result) = state.current_results.get(selected_idx).cloned() {
                        // Check if this result is an extension that accepts queries
                        let keyword = match &result {
                            SearchResult::Quicklink {
                                keyword,
                                has_query: true,
                                ..
                            } => Some(keyword.clone()),
                            SearchResult::Script {
                                id,
                                has_argument: true,
                                ..
                            } => Some(id.clone()),
                            _ => None,
                        };

                        if let Some(kw) = keyword {
                            if let Some(ext) = extension_index_for_key.borrow().get_by_keyword(&kw)
                            {
                                if ext.accepts_query() {
                                    state.enter_command_mode(ext.clone());
                                    let results = search_in_command_mode(
                                        &state.command_mode,
                                        "",
                                        max_results,
                                    );
                                    state.update_results(results);
                                }
                            }
                        }
                    }
                }
                return glib::Propagation::Stop;
            }
            gdk::keys::constants::BackSpace => {
                let mut state = ui_state_for_key.borrow_mut();
                if state.command_mode.is_active() && state.entry.text().is_empty() {
                    state.exit_command_mode();
                    let results = search_with_commands(
                        &app_index_for_key,
                        &custom_commands_for_key.borrow(),
                        &extension_manager_for_key,
                        &clipboard_history_for_key.borrow(),
                        "",
                        max_results,
                    );
                    state.update_results(results);
                    return glib::Propagation::Stop;
                }
                return glib::Propagation::Proceed;
            }
            gdk::keys::constants::Escape => {
                let mut state = ui_state_for_key.borrow_mut();
                if state.command_mode.is_active() {
                    // First Escape: exit command mode, don't hide window
                    state.exit_command_mode();
                    state.clear_entry();
                    let results = search_with_commands(
                        &app_index_for_key,
                        &custom_commands_for_key.borrow(),
                        &extension_manager_for_key,
                        &clipboard_history_for_key.borrow(),
                        "",
                        max_results,
                    );
                    state.update_results(results);
                } else {
                    // Hide window
                    state.hide_window(&config_for_key.borrow());
                }
                return glib::Propagation::Stop;
            }
            gdk::keys::constants::Return | gdk::keys::constants::KP_Enter => {
                // Clone needed data before borrowing state
                let selected_result = {
                    let state = ui_state_for_key.borrow();
                    state
                        .current_results
                        .get(state.selected_index as usize)
                        .cloned()
                };

                if let Some(result) = selected_result {
                    let do_hide = || {
                        ui_state_for_key
                            .borrow_mut()
                            .hide_window(&config_for_key.borrow());
                    };

                    match result_to_action(&result) {
                        ExecutionAction::LaunchApp { app } => {
                            // Use platform trait to launch the app
                            let platform = platform::current();
                            if let Err(e) = platform.launch_app(&app) {
                                eprintln!("[Nova] Launch error for {}: {}", app.name, e);
                            } else {
                                do_hide();
                            }
                        }
                        ExecutionAction::OpenSettings => {
                            do_hide();
                            let app_clone = app_for_key.clone();
                            glib::idle_add_local_once(move || {
                                settings::show_settings_window(&app_clone);
                            });
                        }
                        ExecutionAction::Quit => {
                            do_hide();
                            std::process::exit(0);
                        }
                        ExecutionAction::SystemCommand { command } => {
                            do_hide();
                            let platform = platform::current();
                            if let Err(e) = platform.system_command(command) {
                                eprintln!("[Nova] System command failed: {}", e);
                            }
                        }
                        ExecutionAction::RunShellCommand { command } => {
                            do_hide();
                            let _ = Command::new("sh").args(["-c", &command]).spawn();
                        }
                        ExecutionAction::OpenUrl { url } => {
                            do_hide();
                            let _ = open_url(&url);
                        }
                        ExecutionAction::RunScript {
                            path,
                            argument,
                            output_mode,
                        } => {
                            do_hide();
                            let _ = execute_script(&path, argument.as_ref(), &output_mode);
                        }
                        ExecutionAction::RunExtensionCommand { command, argument } => {
                            do_hide();
                            let _ = execute_extension_command(
                                &extension_manager_for_key,
                                &command,
                                argument.as_ref(),
                            );
                        }
                        ExecutionAction::RunDenoCommand {
                            extension_id,
                            command_id,
                            ..
                        } => {
                            // Deno command execution - show notification for now
                            // TODO: Integrate with Deno extension host
                            do_hide();
                            let _ = show_notification(
                                "Deno Extension",
                                &format!("Running {}:{}", extension_id, command_id),
                            );
                        }
                        ExecutionAction::CopyToClipboard {
                            content,
                            notification,
                        } => {
                            do_hide();
                            if copy_to_clipboard(&content).is_ok() {
                                let _ = show_notification("Copied", &notification);
                            }
                        }
                        ExecutionAction::OpenFile { path } => {
                            do_hide();
                            let _ = Command::new("xdg-open").arg(&path).spawn();
                        }
                        ExecutionAction::NeedsInput => {
                            // Don't hide - waiting for user input
                        }
                    }
                }
                return glib::Propagation::Stop;
            }
            gdk::keys::constants::Up | gdk::keys::constants::KP_Up => {
                ui_state_for_key.borrow_mut().navigate_selection(-1);
                return glib::Propagation::Stop;
            }
            gdk::keys::constants::Down | gdk::keys::constants::KP_Down => {
                ui_state_for_key.borrow_mut().navigate_selection(1);
                return glib::Propagation::Stop;
            }
            _ => {}
        }
        glib::Propagation::Proceed
    });

    // Note: We intentionally don't hide on focus-out because it races with toggle.
    // Window hides via: Escape key, toggle shortcut, or after launching an app.

    // IPC listener
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || {
        let socket_path = get_socket_path();
        let _ = std::fs::remove_file(&socket_path);
        let listener = match UnixListener::bind(&socket_path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[Nova] Failed to bind socket: {:?}", e);
                return;
            }
        };
        println!("[Nova] IPC listener started");
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut buf = [0u8; 6];
                if stream.read(&mut buf).is_ok() && &buf == b"toggle" {
                    let _ = tx.send("toggle".to_string());
                    let _ = stream.write_all(b"ok");
                }
            }
        }
    });

    // Poll for IPC messages (toggle window visibility)
    let ui_state_for_ipc = ui_state.clone();
    let app_index_for_ipc = app_index.clone();
    let custom_commands_for_ipc = custom_commands.clone();
    let extension_manager_for_ipc = extension_manager.clone();
    let clipboard_history_for_ipc = clipboard_history.clone();
    let config_for_ipc = config_ref.clone();

    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        if let Ok(_msg) = rx.try_recv() {
            let mut state = ui_state_for_ipc.borrow_mut();

            if state.is_visible {
                // Hide window
                state.hide_window(&config_for_ipc.borrow());
            } else {
                // Show initial results (apps only when empty query)
                let results = search_with_commands(
                    &app_index_for_ipc,
                    &custom_commands_for_ipc.borrow(),
                    &extension_manager_for_ipc,
                    &clipboard_history_for_ipc.borrow(),
                    "",
                    max_results,
                );
                state.update_results(results);

                // Show window
                state.show_window(&config_for_ipc.borrow());
            }
        }
        ControlFlow::Continue
    });

    // Poll clipboard for changes (every 500ms)
    glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
        clipboard_history.borrow_mut().poll();
        ControlFlow::Continue
    });

    println!("[Nova] Started - Super+Space to toggle");
}

/// Render search results into the GTK ListBox widget
fn render_results_list(list: &ListBox, results: &[SearchResult]) {
    // Clear existing rows
    for child in list.children() {
        list.remove(&child);
    }

    // Add new rows
    for result in results {
        let row = ListBoxRow::new();
        let hbox = gtk::Box::new(Orientation::Vertical, 2);
        hbox.set_margin_start(4);
        hbox.set_margin_end(4);

        let name_label = Label::new(Some(result.name()));
        name_label.set_halign(gtk::Align::Start);
        name_label.style_context().add_class("nova-result-name");

        hbox.pack_start(&name_label, false, false, 0);

        if let Some(desc) = result.description() {
            let desc_label = Label::new(Some(desc));
            desc_label.set_halign(gtk::Align::Start);
            desc_label.set_ellipsize(pango::EllipsizeMode::End);
            desc_label.style_context().add_class("nova-result-desc");
            hbox.pack_start(&desc_label, false, false, 0);
        }

        row.add(&hbox);
        row.show_all();
        list.add(&row);
    }
}

fn position_window(window: &ApplicationWindow, config: &config::Config) {
    // Use saved position if available, otherwise center
    if let (Some(x), Some(y)) = (config.appearance.window_x, config.appearance.window_y) {
        window.move_(x, y);
    } else {
        // Default: center horizontally, 1/5 from top
        if let Some(screen) = WidgetExt::screen(window) {
            let display = screen.display();
            if let Some(monitor) = display.primary_monitor() {
                let geometry = monitor.geometry();
                let (width, _height) = window.size();
                let x = geometry.x() + (geometry.width() - width) / 2;
                let y = geometry.y() + (geometry.height() / 5);
                window.move_(x, y);
            }
        }
    }
}
