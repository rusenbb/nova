//! Window building and rendering for GTK frontend.

use crate::settings;
use gdk::prelude::*;
use gdk::Screen;
use glib::ControlFlow;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, CssProvider, Entry, EventBox, Label, ListBox, ListBoxRow,
    Orientation, StyleContext,
};
use nova::config;
use nova::core::search::{SearchEngine, SearchResult};
use nova::platform;
use nova::services::{self, AppIndex, CustomCommandsIndex, ExtensionIndex};
use std::cell::RefCell;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::process::Command;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use super::exec::{
    copy_to_clipboard, execute_extension_command, execute_script, open_url, show_notification,
};
use super::ipc::get_socket_path;
use super::state::{result_to_action, CommandModeState, UIState, UIStateHandle};
use nova::executor::ExecutionAction;

pub fn build_ui(app: &Application) {
    // Load config (stored in Rc<RefCell> for runtime updates like position)
    let config = Rc::new(RefCell::new(config::Config::load()));
    let max_results = config.borrow().behavior.max_results as usize;

    // Ensure autostart state matches config
    if let Err(e) = config::set_autostart(config.borrow().behavior.autostart) {
        eprintln!("[Nova] Failed to set autostart: {}", e);
    }

    // Initialize search engine, app index, custom commands, extensions, and clipboard history
    let search_engine = Rc::new(SearchEngine::new());
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
    let search_engine_search = search_engine.clone();
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
                    let results =
                        search_engine_search.search_in_command_mode(&ext, "", max_results);
                    state.update_results(results);
                    return;
                }
            }
        }

        // Perform search based on mode
        let results = if let Some(ref ext) = state.command_mode.active_extension {
            search_engine_search.search_in_command_mode(ext, &query, max_results)
        } else {
            search_engine_search.search(
                app_index_search.entries(),
                &custom_commands_search.borrow(),
                &extension_manager_search,
                &clipboard_history_search.borrow(),
                None, // GTK frontend doesn't use frecency yet
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
    let search_engine_for_key = search_engine.clone();
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
                                    let results = search_engine_for_key.search_in_command_mode(
                                        &ext,
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
                    let results = search_engine_for_key.search(
                        app_index_for_key.entries(),
                        &custom_commands_for_key.borrow(),
                        &extension_manager_for_key,
                        &clipboard_history_for_key.borrow(),
                        None,
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
                    let results = search_engine_for_key.search(
                        app_index_for_key.entries(),
                        &custom_commands_for_key.borrow(),
                        &extension_manager_for_key,
                        &clipboard_history_for_key.borrow(),
                        None,
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
                            let plat = platform::current();
                            if let Err(e) = plat.launch_app(&app) {
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
                            let plat = platform::current();
                            if let Err(e) = plat.system_command(command) {
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
        for mut stream in listener.incoming().flatten() {
            let mut buf = [0u8; 6];
            if stream.read(&mut buf).is_ok() && &buf == b"toggle" {
                let _ = tx.send("toggle".to_string());
                let _ = stream.write_all(b"ok");
            }
        }
    });

    // Poll for IPC messages (toggle window visibility)
    let ui_state_for_ipc = ui_state.clone();
    let search_engine_for_ipc = search_engine.clone();
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
                let results = search_engine_for_ipc.search(
                    app_index_for_ipc.entries(),
                    &custom_commands_for_ipc.borrow(),
                    &extension_manager_for_ipc,
                    &clipboard_history_for_ipc.borrow(),
                    None,
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
        let plat = platform::current();
        if let Some(content) = plat.clipboard_read() {
            clipboard_history.borrow_mut().poll_with_content(&content);
        }
        ControlFlow::Continue
    });

    println!("[Nova] Started - Super+Space to toggle");
}

/// Render search results into the GTK ListBox widget
pub fn render_results_list(list: &ListBox, results: &[SearchResult]) {
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

pub fn position_window(window: &ApplicationWindow, config: &config::Config) {
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
