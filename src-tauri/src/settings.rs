use crate::config::{set_autostart, AliasConfig, Config, QuicklinkConfig};
use glib::clone;
use gtk::prelude::*;
use gtk::{
    Adjustment, Application, ApplicationWindow, Box as GtkBox, Button, ColorButton,
    ComboBoxText, CssProvider, Dialog, DialogFlags, Entry, Grid, Label, ListBox, ListBoxRow,
    Notebook, Orientation, ResponseType, Scale, ScrolledWindow, SpinButton, StyleContext, Switch,
};
use pango::EllipsizeMode;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;

const SETTINGS_CSS: &str = r#"
    .settings-window {
        background-color: #1e1e2e;
    }

    .settings-header {
        background-color: #181825;
        padding: 16px;
        border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    }

    .settings-title {
        font-size: 18px;
        font-weight: 600;
        color: #cdd6f4;
    }

    .settings-content {
        padding: 16px;
    }

    .settings-section {
        margin-bottom: 24px;
    }

    .settings-section-title {
        font-size: 14px;
        font-weight: 600;
        color: #cba6f7;
        margin-bottom: 12px;
    }

    .settings-row {
        margin-bottom: 12px;
    }

    .settings-label {
        color: #a6adc8;
        font-size: 13px;
    }

    .settings-entry {
        background-color: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 6px;
        padding: 8px 12px;
        color: #cdd6f4;
        min-width: 200px;
    }

    .settings-entry:focus {
        border-color: #cba6f7;
    }

    .settings-button {
        background-color: rgba(203, 166, 247, 0.2);
        color: #cba6f7;
        border: 1px solid rgba(203, 166, 247, 0.3);
        border-radius: 6px;
        padding: 8px 16px;
    }

    .settings-button:hover {
        background-color: rgba(203, 166, 247, 0.3);
    }

    .settings-button-primary {
        background-color: #cba6f7;
        color: #1e1e2e;
        border: none;
        border-radius: 6px;
        padding: 8px 24px;
        font-weight: 600;
    }

    .settings-button-primary:hover {
        background-color: #d4beff;
    }

    .settings-footer {
        background-color: #181825;
        padding: 12px 16px;
        border-top: 1px solid rgba(255, 255, 255, 0.1);
    }

    separator {
        background-color: rgba(255, 255, 255, 0.1);
        margin: 16px 0;
    }

    .settings-list {
        background-color: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 8px;
    }

    .settings-list row {
        padding: 8px 12px;
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    }

    .settings-list row:last-child {
        border-bottom: none;
    }

    .settings-list-item-title {
        color: #cdd6f4;
        font-size: 13px;
        font-weight: 500;
    }

    .settings-list-item-subtitle {
        color: #6c7086;
        font-size: 11px;
    }

    .settings-list-empty {
        color: #6c7086;
        font-size: 12px;
        padding: 16px;
    }

    .settings-button-small {
        background-color: transparent;
        color: #a6adc8;
        border: none;
        padding: 4px 8px;
        min-width: 0;
        min-height: 0;
    }

    .settings-button-small:hover {
        color: #cdd6f4;
        background-color: rgba(255, 255, 255, 0.1);
    }

    .settings-button-add {
        background-color: rgba(166, 227, 161, 0.2);
        color: #a6e3a1;
        border: 1px solid rgba(166, 227, 161, 0.3);
        border-radius: 6px;
        padding: 6px 12px;
        font-size: 12px;
    }

    .settings-button-add:hover {
        background-color: rgba(166, 227, 161, 0.3);
    }

    .dialog-content {
        padding: 16px;
        background-color: #1e1e2e;
    }

    .dialog-label {
        color: #a6adc8;
        font-size: 12px;
        margin-bottom: 4px;
    }

    .dialog-entry {
        background-color: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 6px;
        padding: 8px 12px;
        color: #cdd6f4;
        margin-bottom: 12px;
    }

    notebook {
        background-color: #1e1e2e;
    }

    notebook header {
        background-color: #181825;
        border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    }

    notebook header tabs {
        background-color: transparent;
    }

    notebook header tab {
        background-color: transparent;
        color: #6c7086;
        padding: 8px 16px;
        border: none;
        border-bottom: 2px solid transparent;
    }

    notebook header tab:checked {
        color: #cba6f7;
        border-bottom: 2px solid #cba6f7;
        background-color: rgba(203, 166, 247, 0.1);
    }

    notebook header tab:hover {
        color: #cdd6f4;
        background-color: rgba(255, 255, 255, 0.05);
    }

    .tab-content {
        padding: 16px;
    }
"#;

pub fn show_settings_window(app: &Application) {
    let config = Rc::new(RefCell::new(Config::load()));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Nova Settings")
        .default_width(550)
        .default_height(700)
        .resizable(true)
        .build();

    window.style_context().add_class("settings-window");

    // Load CSS
    let provider = CssProvider::new();
    provider.load_from_data(SETTINGS_CSS.as_bytes()).expect("Failed to load CSS");
    if let Some(screen) = gdk::Screen::default() {
        StyleContext::add_provider_for_screen(
            &screen,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    // Main container
    let main_box = GtkBox::new(Orientation::Vertical, 0);

    // Create notebook (tabs)
    let notebook = Notebook::new();
    notebook.set_tab_pos(gtk::PositionType::Top);

    // ==================== TAB 1: General ====================
    let general_tab = GtkBox::new(Orientation::Vertical, 16);
    general_tab.style_context().add_class("tab-content");

    // Keyboard Shortcut
    let shortcut_section = create_section("Keyboard Shortcut");
    let shortcut_row = GtkBox::new(Orientation::Horizontal, 12);

    let shortcut_entry = Entry::new();
    shortcut_entry.set_text(&config.borrow().general.hotkey);
    shortcut_entry.set_editable(false);
    shortcut_entry.style_context().add_class("settings-entry");
    shortcut_entry.set_hexpand(true);

    let record_button = Button::with_label("Record");
    record_button.style_context().add_class("settings-button");

    let is_recording = Rc::new(RefCell::new(false));

    record_button.connect_clicked(clone!(@weak shortcut_entry, @strong is_recording => move |btn| {
        *is_recording.borrow_mut() = true;
        shortcut_entry.set_text("Press shortcut keys...");
        shortcut_entry.grab_focus();
        btn.set_sensitive(false);
    }));

    shortcut_entry.connect_key_press_event(clone!(@strong is_recording, @weak record_button => @default-return glib::Propagation::Proceed, move |entry, event| {
        if !*is_recording.borrow() {
            return glib::Propagation::Proceed;
        }

        let key = event.keyval();
        let modifiers = event.state();

        if is_modifier_key(key) {
            return glib::Propagation::Stop;
        }

        let shortcut = format_shortcut(modifiers, key);
        entry.set_text(&shortcut);
        *is_recording.borrow_mut() = false;
        record_button.set_sensitive(true);

        glib::Propagation::Stop
    }));

    shortcut_row.pack_start(&shortcut_entry, true, true, 0);
    shortcut_row.pack_start(&record_button, false, false, 0);
    shortcut_section.pack_start(&shortcut_row, false, false, 0);
    general_tab.pack_start(&shortcut_section, false, false, 0);

    // Behavior
    let behavior_section = create_section("Behavior");
    let behavior_grid = Grid::new();
    behavior_grid.set_row_spacing(12);
    behavior_grid.set_column_spacing(16);

    let autostart_label = create_label("Auto-start on login");
    let autostart_switch = Switch::new();
    autostart_switch.set_active(config.borrow().behavior.autostart);
    autostart_switch.set_halign(gtk::Align::Start);

    behavior_grid.attach(&autostart_label, 0, 0, 1, 1);
    behavior_grid.attach(&autostart_switch, 1, 0, 1, 1);

    let max_results_label = create_label("Max results");
    let max_results_adj = Adjustment::new(
        config.borrow().behavior.max_results as f64,
        1.0, 20.0, 1.0, 5.0, 0.0
    );
    let max_results_spin = SpinButton::new(Some(&max_results_adj), 1.0, 0);
    max_results_spin.set_halign(gtk::Align::Start);

    behavior_grid.attach(&max_results_label, 0, 1, 1, 1);
    behavior_grid.attach(&max_results_spin, 1, 1, 1, 1);

    behavior_section.pack_start(&behavior_grid, false, false, 0);
    general_tab.pack_start(&behavior_section, false, false, 0);

    notebook.append_page(&general_tab, Some(&Label::new(Some("General"))));

    // ==================== TAB 2: Appearance ====================
    let appearance_tab = GtkBox::new(Orientation::Vertical, 16);
    appearance_tab.style_context().add_class("tab-content");

    let appearance_section = create_section("Appearance");
    let appearance_grid = Grid::new();
    appearance_grid.set_row_spacing(12);
    appearance_grid.set_column_spacing(16);

    let theme_label = create_label("Theme");
    let theme_combo = ComboBoxText::new();
    theme_combo.append(Some("catppuccin-mocha"), "Catppuccin Mocha");
    theme_combo.append(Some("catppuccin-latte"), "Catppuccin Latte");
    theme_combo.append(Some("nord"), "Nord");
    theme_combo.set_active_id(Some(&config.borrow().appearance.theme));
    theme_combo.set_hexpand(true);

    appearance_grid.attach(&theme_label, 0, 0, 1, 1);
    appearance_grid.attach(&theme_combo, 1, 0, 1, 1);

    let accent_label = create_label("Accent Color");
    let accent_color = parse_color(&config.borrow().appearance.accent_color);
    let color_button = ColorButton::with_rgba(&accent_color);
    color_button.set_use_alpha(false);

    appearance_grid.attach(&accent_label, 0, 1, 1, 1);
    appearance_grid.attach(&color_button, 1, 1, 1, 1);

    let opacity_label = create_label("Opacity");
    let opacity_box = GtkBox::new(Orientation::Horizontal, 8);
    let opacity_adj = Adjustment::new(
        config.borrow().appearance.opacity,
        0.5, 1.0, 0.01, 0.1, 0.0
    );
    let opacity_scale = Scale::new(Orientation::Horizontal, Some(&opacity_adj));
    opacity_scale.set_hexpand(true);
    opacity_scale.set_draw_value(false);

    let opacity_value_label = Label::new(Some(&format!("{}%", (config.borrow().appearance.opacity * 100.0) as i32)));
    opacity_value_label.style_context().add_class("settings-label");

    opacity_adj.connect_value_changed(clone!(@weak opacity_value_label => move |adj| {
        opacity_value_label.set_text(&format!("{}%", (adj.value() * 100.0) as i32));
    }));

    opacity_box.pack_start(&opacity_scale, true, true, 0);
    opacity_box.pack_start(&opacity_value_label, false, false, 0);

    appearance_grid.attach(&opacity_label, 0, 2, 1, 1);
    appearance_grid.attach(&opacity_box, 1, 2, 1, 1);

    appearance_section.pack_start(&appearance_grid, false, false, 0);
    appearance_tab.pack_start(&appearance_section, false, false, 0);

    notebook.append_page(&appearance_tab, Some(&Label::new(Some("Appearance"))));

    // ==================== TAB 3: Aliases ====================
    let aliases_tab = GtkBox::new(Orientation::Vertical, 16);
    aliases_tab.style_context().add_class("tab-content");

    let aliases_section = create_section("Aliases");
    let aliases_desc = Label::new(Some("Short keywords to launch apps quickly (e.g., \"ff\" → Firefox)"));
    aliases_desc.style_context().add_class("settings-label");
    aliases_desc.set_halign(gtk::Align::Start);
    aliases_desc.set_line_wrap(true);
    aliases_section.pack_start(&aliases_desc, false, false, 0);

    let aliases_list = ListBox::new();
    aliases_list.style_context().add_class("settings-list");
    aliases_list.set_selection_mode(gtk::SelectionMode::None);

    let aliases_data: Rc<RefCell<Vec<AliasConfig>>> = Rc::new(RefCell::new(config.borrow().aliases.clone()));
    refresh_aliases_list(&aliases_list, &aliases_data.borrow());

    let aliases_scroll = ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
    aliases_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    aliases_scroll.set_min_content_height(200);
    aliases_scroll.add(&aliases_list);
    aliases_section.pack_start(&aliases_scroll, true, true, 0);

    let aliases_buttons = GtkBox::new(Orientation::Horizontal, 8);
    aliases_buttons.set_margin_top(8);
    let add_alias_btn = Button::with_label("+ Add Alias");
    add_alias_btn.style_context().add_class("settings-button-add");

    let aliases_list_for_add = aliases_list.clone();
    let aliases_data_for_add = aliases_data.clone();
    let window_for_alias = window.clone();
    add_alias_btn.connect_clicked(move |_| {
        if let Some((keyword, name, target)) = show_alias_dialog(&window_for_alias, None) {
            aliases_data_for_add.borrow_mut().push(AliasConfig {
                keyword,
                name,
                target,
                icon: None,
            });
            refresh_aliases_list(&aliases_list_for_add, &aliases_data_for_add.borrow());
        }
    });

    aliases_buttons.pack_start(&add_alias_btn, false, false, 0);
    aliases_section.pack_start(&aliases_buttons, false, false, 0);
    aliases_tab.pack_start(&aliases_section, true, true, 0);

    notebook.append_page(&aliases_tab, Some(&Label::new(Some("Aliases"))));

    // ==================== TAB 4: Quicklinks ====================
    let quicklinks_tab = GtkBox::new(Orientation::Vertical, 16);
    quicklinks_tab.style_context().add_class("tab-content");

    let quicklinks_section = create_section("Quicklinks");
    let quicklinks_desc = Label::new(Some("URL shortcuts with optional {query} for search"));
    quicklinks_desc.style_context().add_class("settings-label");
    quicklinks_desc.set_halign(gtk::Align::Start);
    quicklinks_desc.set_line_wrap(true);
    quicklinks_section.pack_start(&quicklinks_desc, false, false, 0);

    let quicklinks_list = ListBox::new();
    quicklinks_list.style_context().add_class("settings-list");
    quicklinks_list.set_selection_mode(gtk::SelectionMode::None);

    let quicklinks_data: Rc<RefCell<Vec<QuicklinkConfig>>> = Rc::new(RefCell::new(config.borrow().quicklinks.clone()));
    refresh_quicklinks_list(&quicklinks_list, &quicklinks_data.borrow());

    let quicklinks_scroll = ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
    quicklinks_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    quicklinks_scroll.set_min_content_height(200);
    quicklinks_scroll.add(&quicklinks_list);
    quicklinks_section.pack_start(&quicklinks_scroll, true, true, 0);

    let quicklinks_buttons = GtkBox::new(Orientation::Horizontal, 8);
    quicklinks_buttons.set_margin_top(8);
    let add_quicklink_btn = Button::with_label("+ Add Quicklink");
    add_quicklink_btn.style_context().add_class("settings-button-add");

    let quicklinks_list_for_add = quicklinks_list.clone();
    let quicklinks_data_for_add = quicklinks_data.clone();
    let window_for_quicklink = window.clone();
    add_quicklink_btn.connect_clicked(move |_| {
        if let Some((keyword, name, url)) = show_quicklink_dialog(&window_for_quicklink, None) {
            quicklinks_data_for_add.borrow_mut().push(QuicklinkConfig {
                keyword,
                name,
                url,
                icon: None,
            });
            refresh_quicklinks_list(&quicklinks_list_for_add, &quicklinks_data_for_add.borrow());
        }
    });

    quicklinks_buttons.pack_start(&add_quicklink_btn, false, false, 0);
    quicklinks_section.pack_start(&quicklinks_buttons, false, false, 0);
    quicklinks_tab.pack_start(&quicklinks_section, true, true, 0);

    notebook.append_page(&quicklinks_tab, Some(&Label::new(Some("Quicklinks"))));

    // ==================== TAB 5: Scripts ====================
    let scripts_tab = GtkBox::new(Orientation::Vertical, 16);
    scripts_tab.style_context().add_class("tab-content");

    let scripts_section = create_section("Script Commands");
    let scripts_desc = Label::new(Some("Shell scripts in ~/.config/nova/scripts/ with # nova: headers"));
    scripts_desc.style_context().add_class("settings-label");
    scripts_desc.set_halign(gtk::Align::Start);
    scripts_desc.set_line_wrap(true);
    scripts_section.pack_start(&scripts_desc, false, false, 0);

    let scripts_info = Label::new(Some("Scripts are automatically detected from your scripts folder.\nAdd metadata headers to customize how they appear:"));
    scripts_info.style_context().add_class("settings-list-item-subtitle");
    scripts_info.set_halign(gtk::Align::Start);
    scripts_info.set_line_wrap(true);
    scripts_section.pack_start(&scripts_info, false, false, 0);

    let example_label = Label::new(Some("# nova: name = \"My Script\"\n# nova: description = \"What it does\"\n# nova: output = \"clipboard\"  (or notification, silent)"));
    example_label.style_context().add_class("settings-entry");
    example_label.set_halign(gtk::Align::Start);
    example_label.set_margin_top(8);
    example_label.set_selectable(true);
    scripts_section.pack_start(&example_label, false, false, 0);

    let open_scripts_btn = Button::with_label("Open Scripts Folder");
    open_scripts_btn.style_context().add_class("settings-button");
    open_scripts_btn.set_halign(gtk::Align::Start);
    open_scripts_btn.set_margin_top(12);
    open_scripts_btn.connect_clicked(|_| {
        let scripts_dir = shellexpand::tilde("~/.config/nova/scripts").to_string();
        let _ = std::fs::create_dir_all(&scripts_dir);
        let _ = Command::new("xdg-open").arg(&scripts_dir).spawn();
    });
    scripts_section.pack_start(&open_scripts_btn, false, false, 0);

    scripts_tab.pack_start(&scripts_section, false, false, 0);

    notebook.append_page(&scripts_tab, Some(&Label::new(Some("Scripts"))));

    // Add notebook to main box
    main_box.pack_start(&notebook, true, true, 0);

    // Footer with buttons
    let footer = GtkBox::new(Orientation::Horizontal, 12);
    footer.style_context().add_class("settings-footer");
    footer.set_halign(gtk::Align::End);

    let cancel_button = Button::with_label("Cancel");
    cancel_button.style_context().add_class("settings-button");

    let save_button = Button::with_label("Save");
    save_button.style_context().add_class("settings-button-primary");

    footer.pack_start(&cancel_button, false, false, 0);
    footer.pack_start(&save_button, false, false, 0);

    main_box.pack_start(&footer, false, false, 0);

    // Cancel button closes window
    cancel_button.connect_clicked(clone!(@weak window => move |_| {
        window.close();
    }));

    // Save button saves config and closes
    let aliases_data_for_save = aliases_data.clone();
    let quicklinks_data_for_save = quicklinks_data.clone();
    save_button.connect_clicked(clone!(
        @weak window,
        @weak shortcut_entry,
        @weak theme_combo,
        @weak color_button,
        @weak opacity_adj,
        @weak autostart_switch,
        @weak max_results_adj,
        @strong config
        => move |_| {
            let mut cfg = config.borrow_mut();

            // Update config from widgets
            cfg.general.hotkey = shortcut_entry.text().to_string();

            if let Some(theme_id) = theme_combo.active_id() {
                cfg.appearance.theme = theme_id.to_string();
            }

            let rgba = color_button.rgba();
            cfg.appearance.accent_color = format!(
                "#{:02x}{:02x}{:02x}",
                (rgba.red() * 255.0) as u8,
                (rgba.green() * 255.0) as u8,
                (rgba.blue() * 255.0) as u8
            );

            cfg.appearance.opacity = opacity_adj.value();
            cfg.behavior.autostart = autostart_switch.is_active();
            cfg.behavior.max_results = max_results_adj.value() as u32;

            // Update aliases and quicklinks from UI data
            cfg.aliases = aliases_data_for_save.borrow().clone();
            cfg.quicklinks = quicklinks_data_for_save.borrow().clone();

            // Save config
            if let Err(e) = cfg.save() {
                eprintln!("[Nova] Failed to save config: {}", e);
            } else {
                println!("[Nova] Config saved");
            }

            // Update GNOME shortcut
            let hotkey = cfg.general.hotkey.clone();
            drop(cfg); // Release borrow before Command

            if let Err(e) = update_gnome_shortcut(&hotkey) {
                eprintln!("[Nova] Failed to update shortcut: {}", e);
            }

            // Update autostart
            if let Err(e) = set_autostart(autostart_switch.is_active()) {
                eprintln!("[Nova] Failed to update autostart: {}", e);
            }

            window.close();
        }
    ));

    window.add(&main_box);
    window.show_all();
}

fn create_section(title: &str) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 8);
    section.style_context().add_class("settings-section");

    let label = Label::new(Some(title));
    label.style_context().add_class("settings-section-title");
    label.set_halign(gtk::Align::Start);

    section.pack_start(&label, false, false, 0);
    section
}

fn create_label(text: &str) -> Label {
    let label = Label::new(Some(text));
    label.style_context().add_class("settings-label");
    label.set_halign(gtk::Align::Start);
    label.set_width_chars(18);
    label
}

fn is_modifier_key(key: gdk::keys::Key) -> bool {
    matches!(
        key,
        gdk::keys::constants::Shift_L
            | gdk::keys::constants::Shift_R
            | gdk::keys::constants::Control_L
            | gdk::keys::constants::Control_R
            | gdk::keys::constants::Alt_L
            | gdk::keys::constants::Alt_R
            | gdk::keys::constants::Super_L
            | gdk::keys::constants::Super_R
            | gdk::keys::constants::Meta_L
            | gdk::keys::constants::Meta_R
    )
}

fn format_shortcut(modifiers: gdk::ModifierType, key: gdk::keys::Key) -> String {
    let mut result = String::new();

    if modifiers.contains(gdk::ModifierType::SUPER_MASK) {
        result.push_str("<Super>");
    }
    if modifiers.contains(gdk::ModifierType::CONTROL_MASK) {
        result.push_str("<Control>");
    }
    if modifiers.contains(gdk::ModifierType::MOD1_MASK) {
        result.push_str("<Alt>");
    }
    if modifiers.contains(gdk::ModifierType::SHIFT_MASK) {
        result.push_str("<Shift>");
    }

    // Get key name using the key's name() method
    if let Some(name) = key.name() {
        result.push_str(&name);
    }

    result
}

fn parse_color(hex: &str) -> gdk::RGBA {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(203) as f64 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(166) as f64 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(247) as f64 / 255.0;
    gdk::RGBA::new(r, g, b, 1.0)
}

fn update_gnome_shortcut(shortcut: &str) -> Result<(), String> {
    let nova_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "nova".to_string());

    let binding_path = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/nova/";
    let schema = "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding";
    let schema_path = format!("{}:{}", schema, binding_path);

    // Get current keybindings list
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.settings-daemon.plugins.media-keys", "custom-keybindings"])
        .output()
        .map_err(|e| format!("Failed to get keybindings: {}", e))?;

    let current = String::from_utf8_lossy(&output.stdout);
    let current = current.trim();

    // Add our binding if not present
    if !current.contains(binding_path) {
        let new_list = if current == "@as []" || current.is_empty() {
            format!("['{}']", binding_path)
        } else {
            let trimmed = current.trim_end_matches(']');
            format!("{}, '{}']", trimmed, binding_path)
        };

        Command::new("gsettings")
            .args(["set", "org.gnome.settings-daemon.plugins.media-keys", "custom-keybindings", &new_list])
            .status()
            .map_err(|e| format!("Failed to update keybindings list: {}", e))?;
    }

    // Set the binding properties
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

    println!("[Nova] Shortcut updated to: {}", shortcut);
    Ok(())
}

fn refresh_aliases_list(list: &ListBox, aliases: &[AliasConfig]) {
    // Clear existing rows
    for child in list.children() {
        list.remove(&child);
    }

    if aliases.is_empty() {
        let empty_label = Label::new(Some("No aliases configured"));
        empty_label.style_context().add_class("settings-list-empty");
        let row = ListBoxRow::new();
        row.add(&empty_label);
        row.set_selectable(false);
        list.add(&row);
    } else {
        for alias in aliases {
            let row = ListBoxRow::new();
            let hbox = GtkBox::new(Orientation::Horizontal, 8);

            let info_box = GtkBox::new(Orientation::Vertical, 2);
            info_box.set_hexpand(true);

            let title = Label::new(Some(&format!("{} → {}", alias.keyword, alias.name)));
            title.style_context().add_class("settings-list-item-title");
            title.set_halign(gtk::Align::Start);

            let subtitle = Label::new(Some(&alias.target));
            subtitle.style_context().add_class("settings-list-item-subtitle");
            subtitle.set_halign(gtk::Align::Start);

            info_box.pack_start(&title, false, false, 0);
            info_box.pack_start(&subtitle, false, false, 0);

            hbox.pack_start(&info_box, true, true, 0);

            row.add(&hbox);
            row.set_selectable(false);
            list.add(&row);
        }
    }

    list.show_all();
}

fn refresh_quicklinks_list(list: &ListBox, quicklinks: &[QuicklinkConfig]) {
    // Clear existing rows
    for child in list.children() {
        list.remove(&child);
    }

    if quicklinks.is_empty() {
        let empty_label = Label::new(Some("No quicklinks configured"));
        empty_label.style_context().add_class("settings-list-empty");
        let row = ListBoxRow::new();
        row.add(&empty_label);
        row.set_selectable(false);
        list.add(&row);
    } else {
        for quicklink in quicklinks {
            let row = ListBoxRow::new();
            let hbox = GtkBox::new(Orientation::Horizontal, 8);

            let info_box = GtkBox::new(Orientation::Vertical, 2);
            info_box.set_hexpand(true);

            let has_query = if quicklink.url.contains("{query}") { " (search)" } else { "" };
            let title = Label::new(Some(&format!("{} → {}{}", quicklink.keyword, quicklink.name, has_query)));
            title.style_context().add_class("settings-list-item-title");
            title.set_halign(gtk::Align::Start);

            let subtitle = Label::new(Some(&quicklink.url));
            subtitle.style_context().add_class("settings-list-item-subtitle");
            subtitle.set_halign(gtk::Align::Start);
            subtitle.set_ellipsize(EllipsizeMode::End);
            subtitle.set_max_width_chars(50);

            info_box.pack_start(&title, false, false, 0);
            info_box.pack_start(&subtitle, false, false, 0);

            hbox.pack_start(&info_box, true, true, 0);

            row.add(&hbox);
            row.set_selectable(false);
            list.add(&row);
        }
    }

    list.show_all();
}

fn show_alias_dialog(parent: &ApplicationWindow, existing: Option<&AliasConfig>) -> Option<(String, String, String)> {
    let dialog = Dialog::with_buttons(
        Some(if existing.is_some() { "Edit Alias" } else { "Add Alias" }),
        Some(parent),
        DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
        &[("Cancel", ResponseType::Cancel), ("Save", ResponseType::Accept)],
    );
    dialog.set_default_size(400, -1);

    let content_area = dialog.content_area();
    content_area.style_context().add_class("dialog-content");
    content_area.set_spacing(4);

    // Keyword
    let keyword_label = Label::new(Some("Keyword (e.g., \"ff\")"));
    keyword_label.style_context().add_class("dialog-label");
    keyword_label.set_halign(gtk::Align::Start);
    content_area.pack_start(&keyword_label, false, false, 0);

    let keyword_entry = Entry::new();
    keyword_entry.style_context().add_class("dialog-entry");
    if let Some(alias) = existing {
        keyword_entry.set_text(&alias.keyword);
    }
    content_area.pack_start(&keyword_entry, false, false, 0);

    // Name
    let name_label = Label::new(Some("Display Name (e.g., \"Firefox\")"));
    name_label.style_context().add_class("dialog-label");
    name_label.set_halign(gtk::Align::Start);
    content_area.pack_start(&name_label, false, false, 0);

    let name_entry = Entry::new();
    name_entry.style_context().add_class("dialog-entry");
    if let Some(alias) = existing {
        name_entry.set_text(&alias.name);
    }
    content_area.pack_start(&name_entry, false, false, 0);

    // Target
    let target_label = Label::new(Some("Command (e.g., \"firefox\" or \"code .\")"));
    target_label.style_context().add_class("dialog-label");
    target_label.set_halign(gtk::Align::Start);
    content_area.pack_start(&target_label, false, false, 0);

    let target_entry = Entry::new();
    target_entry.style_context().add_class("dialog-entry");
    if let Some(alias) = existing {
        target_entry.set_text(&alias.target);
    }
    content_area.pack_start(&target_entry, false, false, 0);

    dialog.show_all();

    let response = dialog.run();
    let result = if response == ResponseType::Accept {
        let keyword = keyword_entry.text().to_string().trim().to_string();
        let name = name_entry.text().to_string().trim().to_string();
        let target = target_entry.text().to_string().trim().to_string();

        if !keyword.is_empty() && !name.is_empty() && !target.is_empty() {
            Some((keyword, name, target))
        } else {
            None
        }
    } else {
        None
    };

    unsafe { dialog.destroy(); }
    result
}

fn show_quicklink_dialog(parent: &ApplicationWindow, existing: Option<&QuicklinkConfig>) -> Option<(String, String, String)> {
    let dialog = Dialog::with_buttons(
        Some(if existing.is_some() { "Edit Quicklink" } else { "Add Quicklink" }),
        Some(parent),
        DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
        &[("Cancel", ResponseType::Cancel), ("Save", ResponseType::Accept)],
    );
    dialog.set_default_size(450, -1);

    let content_area = dialog.content_area();
    content_area.style_context().add_class("dialog-content");
    content_area.set_spacing(4);

    // Keyword
    let keyword_label = Label::new(Some("Keyword (e.g., \"gh\" or \"ghs\")"));
    keyword_label.style_context().add_class("dialog-label");
    keyword_label.set_halign(gtk::Align::Start);
    content_area.pack_start(&keyword_label, false, false, 0);

    let keyword_entry = Entry::new();
    keyword_entry.style_context().add_class("dialog-entry");
    if let Some(ql) = existing {
        keyword_entry.set_text(&ql.keyword);
    }
    content_area.pack_start(&keyword_entry, false, false, 0);

    // Name
    let name_label = Label::new(Some("Display Name (e.g., \"GitHub Search\")"));
    name_label.style_context().add_class("dialog-label");
    name_label.set_halign(gtk::Align::Start);
    content_area.pack_start(&name_label, false, false, 0);

    let name_entry = Entry::new();
    name_entry.style_context().add_class("dialog-entry");
    if let Some(ql) = existing {
        name_entry.set_text(&ql.name);
    }
    content_area.pack_start(&name_entry, false, false, 0);

    // URL
    let url_label = Label::new(Some("URL (use {query} for search, e.g., \"https://github.com/search?q={query}\")"));
    url_label.style_context().add_class("dialog-label");
    url_label.set_halign(gtk::Align::Start);
    content_area.pack_start(&url_label, false, false, 0);

    let url_entry = Entry::new();
    url_entry.style_context().add_class("dialog-entry");
    if let Some(ql) = existing {
        url_entry.set_text(&ql.url);
    }
    content_area.pack_start(&url_entry, false, false, 0);

    dialog.show_all();

    let response = dialog.run();
    let result = if response == ResponseType::Accept {
        let keyword = keyword_entry.text().to_string().trim().to_string();
        let name = name_entry.text().to_string().trim().to_string();
        let url = url_entry.text().to_string().trim().to_string();

        if !keyword.is_empty() && !name.is_empty() && !url.is_empty() {
            Some((keyword, name, url))
        } else {
            None
        }
    } else {
        None
    };

    unsafe { dialog.destroy(); }
    result
}
