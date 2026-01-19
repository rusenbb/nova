mod config;
mod services;
mod settings;

use gdk::prelude::*;
use gdk::Screen;
use glib::ControlFlow;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Entry, EventBox, ListBox, ListBoxRow, Label, CssProvider, StyleContext, Orientation};
use services::AppIndex;
use std::cell::RefCell;
use std::env;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use std::thread;
use std::sync::mpsc;
use std::time::Instant;

const APP_ID: &str = "com.rusen.nova";

use services::{CustomCommandsIndex, ScriptOutputMode, Extension, ExtensionIndex, ExtensionKind};

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

// Search results that appear in the launcher
#[derive(Debug, Clone)]
enum SearchResult {
    App(services::AppEntry),
    Command { id: String, name: String, description: String },
    Alias { keyword: String, name: String, target: String },
    Quicklink { keyword: String, name: String, url: String, has_query: bool },
    QuicklinkWithQuery { keyword: String, name: String, url: String, query: String, resolved_url: String },
    Script { id: String, name: String, description: String, path: PathBuf, has_argument: bool, output_mode: ScriptOutputMode },
    ScriptWithArgument { id: String, name: String, description: String, path: PathBuf, argument: String, output_mode: ScriptOutputMode },
}

impl SearchResult {
    fn name(&self) -> &str {
        match self {
            SearchResult::App(app) => &app.name,
            SearchResult::Command { name, .. } => name,
            SearchResult::Alias { name, .. } => name,
            SearchResult::Quicklink { name, .. } => name,
            SearchResult::QuicklinkWithQuery { name, .. } => name,
            SearchResult::Script { name, .. } => name,
            SearchResult::ScriptWithArgument { name, .. } => name,
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            SearchResult::App(app) => app.description.as_deref(),
            SearchResult::Command { description, .. } => Some(description),
            SearchResult::Alias { target, .. } => Some(target),
            SearchResult::Quicklink { url, .. } => Some(url),
            SearchResult::QuicklinkWithQuery { resolved_url, .. } => Some(resolved_url),
            SearchResult::Script { description, .. } => {
                if description.is_empty() { None } else { Some(description) }
            }
            SearchResult::ScriptWithArgument { description, .. } => {
                if description.is_empty() { None } else { Some(description) }
            }
        }
    }

    fn id(&self) -> &str {
        match self {
            SearchResult::App(app) => &app.id,
            SearchResult::Command { id, .. } => id,
            SearchResult::Alias { keyword, .. } => keyword,
            SearchResult::Quicklink { keyword, .. } => keyword,
            SearchResult::QuicklinkWithQuery { keyword, .. } => keyword,
            SearchResult::Script { id, .. } => id,
            SearchResult::ScriptWithArgument { id, .. } => id,
        }
    }

    fn result_type(&self) -> &'static str {
        match self {
            SearchResult::App(_) => "app",
            SearchResult::Command { .. } => "command",
            SearchResult::Alias { .. } => "alias",
            SearchResult::Quicklink { .. } | SearchResult::QuicklinkWithQuery { .. } => "quicklink",
            SearchResult::Script { .. } | SearchResult::ScriptWithArgument { .. } => "script",
        }
    }
}

fn get_system_commands() -> Vec<SearchResult> {
    vec![
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
    ]
}

fn search_with_commands(
    app_index: &services::AppIndex,
    custom_commands: &CustomCommandsIndex,
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

    // 5. App results
    for app in app_index.search(query) {
        results.push(SearchResult::App(app.clone()));
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
        ExtensionKind::Script { path, output_mode, description, .. } => {
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
/// Parse a hex color string like "#cba6f7" to (r, g, b)
fn parse_hex_color(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(203);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(166);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(247);
        (r, g, b)
    } else {
        (203, 166, 247) // Default to catppuccin mauve
    }
}

/// Get theme colors based on theme name
fn get_theme_colors(theme: &str) -> (&'static str, &'static str, &'static str) {
    // Returns (background_rgb, text_color, subtext_color)
    match theme {
        "catppuccin-mocha" => ("30, 30, 46", "#cdd6f4", "#6c7086"),
        "catppuccin-macchiato" => ("36, 39, 58", "#cad3f5", "#6e738d"),
        "catppuccin-frappe" => ("48, 52, 70", "#c6d0f5", "#737994"),
        "catppuccin-latte" => ("239, 241, 245", "#4c4f69", "#6c6f85"),
        "nord" => ("46, 52, 64", "#eceff4", "#4c566a"),
        "dracula" => ("40, 42, 54", "#f8f8f2", "#6272a4"),
        "gruvbox-dark" => ("40, 40, 40", "#ebdbb2", "#928374"),
        "tokyo-night" => ("26, 27, 38", "#c0caf5", "#565f89"),
        "one-dark" => ("40, 44, 52", "#abb2bf", "#5c6370"),
        _ => ("30, 30, 46", "#cdd6f4", "#6c7086"), // Default to catppuccin-mocha
    }
}

/// Generate CSS with appearance settings from config
fn generate_css(config: &config::AppearanceConfig) -> String {
    let (bg_rgb, text_color, subtext_color) = get_theme_colors(&config.theme);
    let (accent_r, accent_g, accent_b) = parse_hex_color(&config.accent_color);
    let opacity = config.opacity;

    format!(r#"
    window {{
        background-color: transparent;
    }}

    .nova-container {{
        background-color: rgba({bg_rgb}, {opacity});
        border-radius: 12px;
        padding: 12px;
        border: 1px solid rgba(255, 255, 255, 0.1);
    }}

    .nova-entry {{
        background-color: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 8px;
        padding: 12px 16px;
        font-size: 18px;
        color: {text_color};
        min-width: 550px;
    }}

    .nova-entry:focus {{
        border-color: rgba({accent_r}, {accent_g}, {accent_b}, 0.5);
        outline: none;
    }}

    .nova-results {{
        background-color: transparent;
        margin-top: 8px;
    }}

    .nova-results row {{
        padding: 8px 12px;
        border-radius: 6px;
        background-color: transparent;
    }}

    .nova-results row:selected {{
        background-color: rgba({accent_r}, {accent_g}, {accent_b}, 0.35);
        border-left: 3px solid rgba({accent_r}, {accent_g}, {accent_b}, 0.9);
    }}

    .nova-result-name {{
        font-size: 15px;
        font-weight: 500;
        color: {text_color};
    }}

    .nova-result-desc {{
        font-size: 12px;
        color: {subtext_color};
        margin-top: 2px;
    }}

    .nova-entry-container {{
        background-color: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 8px;
        padding: 8px 12px;
    }}

    .nova-entry-container:focus-within {{
        border-color: rgba({accent_r}, {accent_g}, {accent_b}, 0.5);
    }}

    .nova-command-pill {{
        background-color: rgba({accent_r}, {accent_g}, {accent_b}, 0.25);
        border-radius: 4px;
        padding: 4px 10px;
        margin-right: 8px;
        font-size: 13px;
        font-weight: 500;
        color: {accent_color};
    }}

    .nova-entry-in-container {{
        background-color: transparent;
        border: none;
        padding: 4px 4px;
        font-size: 18px;
        color: {text_color};
        min-width: 450px;
    }}

    .nova-entry-in-container:focus {{
        outline: none;
        border: none;
    }}
"#,
    bg_rgb = bg_rgb,
    opacity = opacity,
    text_color = text_color,
    subtext_color = subtext_color,
    accent_r = accent_r,
    accent_g = accent_g,
    accent_b = accent_b,
    accent_color = config.accent_color,
    )
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

fn set_shortcut(shortcut: &str) -> Result<(), String> {
    let nova_path = get_nova_binary_path();

    // GNOME custom keybindings use a path-based schema
    let binding_path = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/nova/";

    // First, add our binding to the list of custom keybindings
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.settings-daemon.plugins.media-keys", "custom-keybindings"])
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
            .args(["set", "org.gnome.settings-daemon.plugins.media-keys", "custom-keybindings", &new_list])
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

    println!("[Nova] Shortcut set to: {}", shortcut);
    println!("[Nova] Command: {}", nova_path);
    println!("\nCommon shortcuts:");
    println!("  <Super>space     - Super+Space (may conflict with GNOME)");
    println!("  <Alt>space       - Alt+Space (recommended)");
    println!("  <Control>space   - Ctrl+Space");
    println!("  <Super><Alt>n    - Super+Alt+N");

    Ok(())
}

fn print_help() {
    println!("Nova - Keyboard-driven productivity launcher for Linux");
    println!();
    println!("USAGE:");
    println!("    nova                      Start Nova (or toggle if already running)");
    println!("    nova --settings           Open settings window");
    println!("    nova --set-shortcut KEY   Set the global keyboard shortcut");
    println!("    nova --help               Show this help message");
    println!();
    println!("SHORTCUT FORMAT:");
    println!("    <Super>space     - Super+Space");
    println!("    <Alt>space       - Alt+Space (recommended, no GNOME conflicts)");
    println!("    <Control>space   - Ctrl+Space");
    println!("    <Super><Alt>n    - Super+Alt+N");
    println!();
    println!("EXAMPLES:");
    println!("    nova --set-shortcut '<Alt>space'");
    println!("    nova --set-shortcut '<Super><Alt>n'");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Handle CLI arguments
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
            _ => {
                eprintln!("Unknown argument: {}", args[1]);
                eprintln!("Run 'nova --help' for usage information");
                std::process::exit(1);
            }
        }
    }

    if try_send_toggle() {
        println!("[Nova] Sent toggle to existing instance");
        return;
    }

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run();
}

/// Update the command pill visibility and text based on command mode state
fn update_command_pill(pill: &Label, entry: &Entry, mode_state: &CommandModeState) {
    if let Some(ref ext) = mode_state.active_extension {
        pill.set_text(ext.pill_text());
        pill.set_visible(true);
        entry.set_placeholder_text(Some(&format!("Search {}...", ext.name)));
    } else {
        pill.set_visible(false);
        entry.set_placeholder_text(Some("Search apps..."));
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

    // Initialize app index and custom commands
    let app_index = Rc::new(AppIndex::new());
    let custom_commands = Rc::new(RefCell::new(CustomCommandsIndex::new(&config.borrow())));

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
    let css = generate_css(&config.borrow().appearance);
    provider.load_from_data(css.as_bytes()).expect("Failed to load CSS");
    if let Some(screen) = Screen::default() {
        StyleContext::add_provider_for_screen(
            &screen,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
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
    entry_container.style_context().add_class("nova-entry-container");
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

    // State
    let window_ref = Rc::new(RefCell::new(window.clone()));
    let is_visible = Rc::new(RefCell::new(false));
    let entry_ref = Rc::new(RefCell::new(entry.clone()));
    let results_ref = Rc::new(RefCell::new(results_list.clone()));
    let selected_index = Rc::new(RefCell::new(0i32));
    let last_toggle = Rc::new(RefCell::new(Instant::now()));
    let is_clearing = Rc::new(RefCell::new(false)); // Guard to prevent callback loops
    let config_ref = config.clone(); // For use in toggle handler

    // Command mode state
    let command_mode = Rc::new(RefCell::new(CommandModeState::default()));
    let command_pill_ref = Rc::new(RefCell::new(command_pill.clone()));

    // Extension index for fast keyword lookup
    let extension_index = Rc::new(RefCell::new(ExtensionIndex::from_custom_commands(
        &custom_commands.borrow(),
        &config.borrow().aliases,
        &config.borrow().quicklinks,
    )));

    // Store current search results for selection handling
    let current_results: Rc<RefCell<Vec<SearchResult>>> = Rc::new(RefCell::new(Vec::new()));

    // Update results when search text changes
    let app_index_search = app_index.clone();
    let custom_commands_search = custom_commands.clone();
    let results_for_search = results_ref.clone();
    let selected_for_search = selected_index.clone();
    let is_clearing_for_search = is_clearing.clone();
    let current_results_for_search = current_results.clone();
    let command_mode_for_search = command_mode.clone();
    let extension_index_for_search = extension_index.clone();
    let command_pill_for_search = command_pill_ref.clone();
    entry.connect_changed(move |entry| {
        // Skip if we're clearing the entry programmatically (prevents RefCell conflicts)
        if *is_clearing_for_search.borrow() {
            return;
        }

        let query = entry.text().to_string();
        let mut mode = command_mode_for_search.borrow_mut();

        // Check for command mode entry: "keyword " pattern (space at end)
        if !mode.is_active() && query.ends_with(' ') && query.len() > 1 {
            let keyword = query.trim();
            if let Some(ext) = extension_index_for_search.borrow().get_by_keyword(keyword) {
                if ext.accepts_query() {
                    // Enter command mode
                    mode.enter_mode(ext.clone());
                    drop(mode);

                    // Clear the entry (pill shows the keyword now)
                    *is_clearing_for_search.borrow_mut() = true;
                    entry.set_text("");
                    *is_clearing_for_search.borrow_mut() = false;

                    // Update pill visibility
                    update_command_pill(
                        &command_pill_for_search.borrow(),
                        entry,
                        &command_mode_for_search.borrow(),
                    );

                    // Show empty state results for command mode
                    let results = search_in_command_mode(&command_mode_for_search.borrow(), "", max_results);
                    update_results_list_v2(&results_for_search.borrow(), &results);
                    *current_results_for_search.borrow_mut() = results;
                    *selected_for_search.borrow_mut() = 0;
                    if let Some(row) = results_for_search.borrow().row_at_index(0) {
                        results_for_search.borrow().select_row(Some(&row));
                    }
                    return;
                }
            }
        }

        // Perform search based on mode
        let results = if mode.is_active() {
            drop(mode);
            search_in_command_mode(&command_mode_for_search.borrow(), &query, max_results)
        } else {
            drop(mode);
            search_with_commands(&app_index_search, &custom_commands_search.borrow(), &query, max_results)
        };

        update_results_list_v2(&results_for_search.borrow(), &results);
        *current_results_for_search.borrow_mut() = results;
        *selected_for_search.borrow_mut() = 0;
        if let Some(row) = results_for_search.borrow().row_at_index(0) {
            results_for_search.borrow().select_row(Some(&row));
        }
    });

    // Handle keyboard events
    let window_for_key = window_ref.clone();
    let visible_for_key = is_visible.clone();
    let results_for_key = results_ref.clone();
    let selected_for_key = selected_index.clone();
    let current_results_for_key = current_results.clone();
    let is_clearing_for_key = is_clearing.clone();
    let app_for_key = app.clone();
    let config_for_key = config_ref.clone();
    let command_mode_for_key = command_mode.clone();
    let command_pill_for_key = command_pill_ref.clone();
    let app_index_for_key = app_index.clone();
    let custom_commands_for_key = custom_commands.clone();
    let extension_index_for_key = extension_index.clone();

    entry.connect_key_press_event(move |entry_widget, event| {
        let key = event.keyval();
        let results_list = results_for_key.borrow();
        let mut selected = selected_for_key.borrow_mut();

        match key {
            gdk::keys::constants::Tab | gdk::keys::constants::ISO_Left_Tab => {
                // Tab enters command mode for selected extension (if it accepts queries)
                // Also prevents Tab from navigating to other UI elements
                let mode = command_mode_for_key.borrow();
                if !mode.is_active() {
                    let selected_idx = *selected as usize;
                    drop(mode);
                    drop(results_list);
                    drop(selected);

                    let results = current_results_for_key.borrow();
                    if let Some(result) = results.get(selected_idx) {
                        // Check if this result is an extension that accepts queries
                        let keyword = match result {
                            SearchResult::Quicklink { keyword, has_query: true, .. } => Some(keyword.clone()),
                            SearchResult::Script { id, has_argument: true, .. } => Some(id.clone()),
                            _ => None,
                        };

                        if let Some(kw) = keyword {
                            drop(results);
                            // Look up the extension and enter command mode
                            if let Some(ext) = extension_index_for_key.borrow().get_by_keyword(&kw) {
                                if ext.accepts_query() {
                                    command_mode_for_key.borrow_mut().enter_mode(ext.clone());

                                    // Clear entry and update pill
                                    *is_clearing_for_key.borrow_mut() = true;
                                    entry_widget.set_text("");
                                    *is_clearing_for_key.borrow_mut() = false;

                                    update_command_pill(
                                        &command_pill_for_key.borrow(),
                                        entry_widget,
                                        &command_mode_for_key.borrow(),
                                    );

                                    // Show command mode results
                                    let new_results = search_in_command_mode(&command_mode_for_key.borrow(), "", max_results);
                                    update_results_list_v2(&results_for_key.borrow(), &new_results);
                                    *current_results_for_key.borrow_mut() = new_results;
                                    *selected_for_key.borrow_mut() = 0;
                                    if let Some(row) = results_for_key.borrow().row_at_index(0) {
                                        results_for_key.borrow().select_row(Some(&row));
                                    }
                                }
                            }
                        }
                    }
                }
                // Always stop Tab from propagating (prevents focus moving to other widgets)
                return glib::Propagation::Stop;
            }
            gdk::keys::constants::BackSpace => {
                // Exit command mode if backspace with empty entry
                let mode = command_mode_for_key.borrow();
                if mode.is_active() && entry_widget.text().is_empty() {
                    drop(results_list);
                    drop(selected);
                    drop(mode);

                    // Exit command mode
                    command_mode_for_key.borrow_mut().exit_mode();
                    update_command_pill(
                        &command_pill_for_key.borrow(),
                        entry_widget,
                        &command_mode_for_key.borrow(),
                    );

                    // Refresh search with normal mode
                    let results = search_with_commands(
                        &app_index_for_key,
                        &custom_commands_for_key.borrow(),
                        "",
                        max_results,
                    );
                    update_results_list_v2(&results_for_key.borrow(), &results);
                    *current_results_for_key.borrow_mut() = results;
                    *selected_for_key.borrow_mut() = 0;
                    if let Some(row) = results_for_key.borrow().row_at_index(0) {
                        results_for_key.borrow().select_row(Some(&row));
                    }

                    return glib::Propagation::Stop;
                }
                return glib::Propagation::Proceed;
            }
            gdk::keys::constants::Escape => {
                let mode = command_mode_for_key.borrow();
                let in_command_mode = mode.is_active();
                drop(mode);
                drop(results_list);
                drop(selected);

                if in_command_mode {
                    // First Escape: exit command mode, don't hide window
                    command_mode_for_key.borrow_mut().exit_mode();
                    *is_clearing_for_key.borrow_mut() = true;
                    entry_widget.set_text("");
                    *is_clearing_for_key.borrow_mut() = false;
                    update_command_pill(
                        &command_pill_for_key.borrow(),
                        entry_widget,
                        &command_mode_for_key.borrow(),
                    );

                    // Refresh search with normal mode
                    let results = search_with_commands(
                        &app_index_for_key,
                        &custom_commands_for_key.borrow(),
                        "",
                        max_results,
                    );
                    update_results_list_v2(&results_for_key.borrow(), &results);
                    *current_results_for_key.borrow_mut() = results;
                    *selected_for_key.borrow_mut() = 0;
                    if let Some(row) = results_for_key.borrow().row_at_index(0) {
                        results_for_key.borrow().select_row(Some(&row));
                    }
                } else {
                    // Second Escape (or first when not in command mode): hide window
                    *is_clearing_for_key.borrow_mut() = true;
                    entry_widget.set_text("");
                    *is_clearing_for_key.borrow_mut() = false;
                    window_for_key.borrow().hide();
                    *visible_for_key.borrow_mut() = false;
                    // Save position to config
                    if let Err(e) = config_for_key.borrow().save() {
                        eprintln!("[Nova] Failed to save config: {}", e);
                    }
                }
                return glib::Propagation::Stop;
            }
            gdk::keys::constants::Return | gdk::keys::constants::KP_Enter => {
                let selected_idx = *selected as usize;
                drop(results_list);
                drop(selected);

                let results = current_results_for_key.borrow();
                if let Some(result) = results.get(selected_idx) {
                    // Helper closure to hide window, reset command mode, and save config
                    let hide_window = || {
                        // Exit command mode if active
                        command_mode_for_key.borrow_mut().exit_mode();
                        update_command_pill(
                            &command_pill_for_key.borrow(),
                            entry_widget,
                            &command_mode_for_key.borrow(),
                        );

                        *is_clearing_for_key.borrow_mut() = true;
                        entry_widget.set_text("");
                        *is_clearing_for_key.borrow_mut() = false;
                        window_for_key.borrow().hide();
                        *visible_for_key.borrow_mut() = false;
                        // Save position to config
                        if let Err(e) = config_for_key.borrow().save() {
                            eprintln!("[Nova] Failed to save config: {}", e);
                        }
                    };

                    match result {
                        SearchResult::App(app) => {
                            if let Err(e) = app.launch() {
                                eprintln!("[Nova] Launch error: {}", e);
                            } else {
                                hide_window();
                            }
                        }
                        SearchResult::Command { id, .. } => {
                            hide_window();
                            match id.as_str() {
                                "nova:settings" => {
                                    let app_clone = app_for_key.clone();
                                    glib::idle_add_local_once(move || {
                                        settings::show_settings_window(&app_clone);
                                    });
                                }
                                "nova:quit" => {
                                    std::process::exit(0);
                                }
                                _ => {}
                            }
                        }
                        SearchResult::Alias { target, .. } => {
                            hide_window();
                            // Try to launch as shell command
                            if let Err(e) = Command::new("sh")
                                .args(["-c", target])
                                .spawn()
                            {
                                eprintln!("[Nova] Alias launch error: {}", e);
                            }
                        }
                        SearchResult::Quicklink { url, has_query, .. } => {
                            // Don't open if query is expected but not provided
                            if !*has_query {
                                hide_window();
                                if let Err(e) = open_url(url) {
                                    eprintln!("[Nova] Quicklink error: {}", e);
                                }
                            }
                        }
                        SearchResult::QuicklinkWithQuery { resolved_url, .. } => {
                            hide_window();
                            if let Err(e) = open_url(resolved_url) {
                                eprintln!("[Nova] Quicklink error: {}", e);
                            }
                        }
                        SearchResult::Script { path, has_argument, output_mode, .. } => {
                            // Don't execute if argument is expected but not provided
                            if !*has_argument {
                                hide_window();
                                if let Err(e) = execute_script(path, None, output_mode) {
                                    eprintln!("[Nova] Script error: {}", e);
                                }
                            }
                        }
                        SearchResult::ScriptWithArgument { path, argument, output_mode, .. } => {
                            hide_window();
                            if let Err(e) = execute_script(path, Some(argument), output_mode) {
                                eprintln!("[Nova] Script error: {}", e);
                            }
                        }
                    }
                }
                return glib::Propagation::Stop;
            }
            gdk::keys::constants::Up | gdk::keys::constants::KP_Up => {
                let n_items = results_list.children().len() as i32;
                if n_items > 0 {
                    *selected = (*selected - 1).rem_euclid(n_items);
                    if let Some(row) = results_list.row_at_index(*selected) {
                        results_list.select_row(Some(&row));
                    }
                }
                return glib::Propagation::Stop;
            }
            gdk::keys::constants::Down | gdk::keys::constants::KP_Down => {
                let n_items = results_list.children().len() as i32;
                if n_items > 0 {
                    *selected = (*selected + 1).rem_euclid(n_items);
                    if let Some(row) = results_list.row_at_index(*selected) {
                        results_list.select_row(Some(&row));
                    }
                }
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

    // Poll for IPC messages
    let window_for_ipc = window_ref.clone();
    let visible_for_ipc = is_visible.clone();
    let entry_for_ipc = entry_ref.clone();
    let results_for_ipc = results_ref.clone();
    let app_index_for_ipc = app_index.clone();
    let custom_commands_for_ipc = custom_commands.clone();
    let selected_for_ipc = selected_index.clone();
    let last_toggle_for_ipc = last_toggle.clone();
    let is_clearing_for_ipc = is_clearing.clone();
    let current_results_for_ipc = current_results.clone();
    let config_for_ipc = config_ref.clone();
    let command_mode_for_ipc = command_mode.clone();
    let command_pill_for_ipc = command_pill_ref.clone();

    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        if let Ok(_msg) = rx.try_recv() {
            // Record toggle timestamp for focus-out debounce
            *last_toggle_for_ipc.borrow_mut() = Instant::now();

            let window = window_for_ipc.borrow();
            let entry = entry_for_ipc.borrow();
            let mut visible = visible_for_ipc.borrow_mut();

            if *visible {
                // Reset command mode when hiding
                command_mode_for_ipc.borrow_mut().exit_mode();
                update_command_pill(
                    &command_pill_for_ipc.borrow(),
                    &entry,
                    &command_mode_for_ipc.borrow(),
                );

                *is_clearing_for_ipc.borrow_mut() = true;
                entry.set_text("");
                *is_clearing_for_ipc.borrow_mut() = false;
                window.hide();
                *visible = false;

                // Save position to config file when hiding
                if let Err(e) = config_for_ipc.borrow().save() {
                    eprintln!("[Nova] Failed to save config: {}", e);
                }
            } else {
                // Ensure command mode is reset when showing
                command_mode_for_ipc.borrow_mut().exit_mode();
                update_command_pill(
                    &command_pill_for_ipc.borrow(),
                    &entry,
                    &command_mode_for_ipc.borrow(),
                );

                // Show initial results (apps only when empty query)
                let results = search_with_commands(&app_index_for_ipc, &custom_commands_for_ipc.borrow(), "", max_results);
                update_results_list_v2(&results_for_ipc.borrow(), &results);
                *current_results_for_ipc.borrow_mut() = results;
                *selected_for_ipc.borrow_mut() = 0;
                if let Some(row) = results_for_ipc.borrow().row_at_index(0) {
                    results_for_ipc.borrow().select_row(Some(&row));
                }

                // Position window (use saved position or center)
                position_window(&window, &config_for_ipc.borrow());

                // Show and present with current timestamp to bypass focus-stealing prevention
                window.show_all();

                // Use present_with_time - 0 means "current time" in X11
                window.present_with_time(0);

                // Ensure focus goes to entry
                entry.set_text("");
                entry.grab_focus();

                // Force the window to be active
                if let Some(gdk_window) = window.window() {
                    gdk_window.focus(0);
                    gdk_window.raise();
                }

                *visible = true;
            }
        }
        ControlFlow::Continue
    });

    println!("[Nova] Started - Super+Space to toggle");
}

fn update_results_list(list: &ListBox, results: &[&services::AppEntry]) {
    // Clear existing rows
    for child in list.children() {
        list.remove(&child);
    }

    // Add new rows
    for app in results {
        let row = ListBoxRow::new();
        let hbox = gtk::Box::new(Orientation::Vertical, 2);
        hbox.set_margin_start(4);
        hbox.set_margin_end(4);

        let name_label = Label::new(Some(&app.name));
        name_label.set_halign(gtk::Align::Start);
        name_label.style_context().add_class("nova-result-name");

        hbox.pack_start(&name_label, false, false, 0);

        if let Some(desc) = &app.description {
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

fn update_results_list_v2(list: &ListBox, results: &[SearchResult]) {
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
