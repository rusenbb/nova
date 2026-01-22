//! Settings window for Nova using iced.
//!
//! This module provides a settings interface for configuring Nova's behavior,
//! appearance, aliases, and quicklinks.

use crate::config::{AliasConfig, Config, QuicklinkConfig};
use crate::ui::style;
use crate::ui::theme::NovaTheme;

use iced::widget::{
    button, column, container, horizontal_rule, pick_list, row, scrollable, slider, text,
    text_input, toggler, Column, Space,
};
use iced::{Alignment, Element, Length};

/// Available theme names for the picker.
const THEMES: &[&str] = &[
    "catppuccin-mocha",
    "catppuccin-macchiato",
    "catppuccin-frappe",
    "catppuccin-latte",
    "nord",
    "dracula",
    "gruvbox-dark",
    "tokyo-night",
    "one-dark",
];

/// Settings tab selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    General,
    Appearance,
    Aliases,
    Quicklinks,
}

impl std::fmt::Display for SettingsTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingsTab::General => write!(f, "General"),
            SettingsTab::Appearance => write!(f, "Appearance"),
            SettingsTab::Aliases => write!(f, "Aliases"),
            SettingsTab::Quicklinks => write!(f, "Quicklinks"),
        }
    }
}

/// Settings window state.
pub struct SettingsWindow {
    config: Config,
    theme: NovaTheme,
    active_tab: SettingsTab,

    // Form state
    max_results: u32,
    opacity: f64,
    selected_theme: String,
    autostart: bool,

    // Alias editing
    aliases: Vec<AliasConfig>,
    editing_alias: Option<usize>,
    new_alias_keyword: String,
    new_alias_name: String,
    new_alias_target: String,

    // Quicklink editing
    quicklinks: Vec<QuicklinkConfig>,
    editing_quicklink: Option<usize>,
    new_quicklink_keyword: String,
    new_quicklink_name: String,
    new_quicklink_url: String,

    // Dirty state
    has_changes: bool,
}

/// Messages for the settings window.
#[derive(Debug, Clone)]
pub enum SettingsMessage {
    // Tab navigation
    TabSelected(SettingsTab),

    // General settings
    MaxResultsChanged(u32),
    AutostartToggled(bool),

    // Appearance settings
    ThemeSelected(String),
    OpacityChanged(f64),

    // Alias management
    EditAlias(usize),
    DeleteAlias(usize),
    NewAliasKeywordChanged(String),
    NewAliasNameChanged(String),
    NewAliasTargetChanged(String),
    SaveAlias,
    CancelAliasEdit,

    // Quicklink management
    EditQuicklink(usize),
    DeleteQuicklink(usize),
    NewQuicklinkKeywordChanged(String),
    NewQuicklinkNameChanged(String),
    NewQuicklinkUrlChanged(String),
    SaveQuicklink,
    CancelQuicklinkEdit,

    // Window actions
    Save,
    Cancel,
}

/// Result from the settings window.
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum SettingsResult {
    /// Settings were saved, returns updated config.
    Saved(Config),
    /// Settings were cancelled.
    Cancelled,
}

impl SettingsWindow {
    /// Create a new settings window with the current config.
    pub fn new(config: Config) -> Self {
        let theme = NovaTheme::by_name(&config.appearance.theme);

        Self {
            max_results: config.behavior.max_results,
            opacity: config.appearance.opacity,
            selected_theme: config.appearance.theme.clone(),
            autostart: config.behavior.autostart,
            aliases: config.aliases.clone(),
            quicklinks: config.quicklinks.clone(),
            config,
            theme,
            active_tab: SettingsTab::General,
            editing_alias: None,
            new_alias_keyword: String::new(),
            new_alias_name: String::new(),
            new_alias_target: String::new(),
            editing_quicklink: None,
            new_quicklink_keyword: String::new(),
            new_quicklink_name: String::new(),
            new_quicklink_url: String::new(),
            has_changes: false,
        }
    }

    /// Update the settings window.
    pub fn update(&mut self, message: SettingsMessage) -> Option<SettingsResult> {
        match message {
            SettingsMessage::TabSelected(tab) => {
                self.active_tab = tab;
                // Clear any editing state when switching tabs
                self.editing_alias = None;
                self.editing_quicklink = None;
                self.clear_alias_form();
                self.clear_quicklink_form();
            }

            SettingsMessage::MaxResultsChanged(value) => {
                self.max_results = value;
                self.has_changes = true;
            }

            SettingsMessage::AutostartToggled(value) => {
                self.autostart = value;
                self.has_changes = true;
            }

            SettingsMessage::ThemeSelected(theme) => {
                self.selected_theme = theme.clone();
                self.theme = NovaTheme::by_name(&theme);
                self.has_changes = true;
            }

            SettingsMessage::OpacityChanged(value) => {
                self.opacity = value;
                self.has_changes = true;
            }

            // Alias management
            SettingsMessage::EditAlias(index) => {
                if let Some(alias) = self.aliases.get(index) {
                    self.editing_alias = Some(index);
                    self.new_alias_keyword = alias.keyword.clone();
                    self.new_alias_name = alias.name.clone();
                    self.new_alias_target = alias.target.clone();
                }
            }

            SettingsMessage::DeleteAlias(index) => {
                if index < self.aliases.len() {
                    self.aliases.remove(index);
                    self.has_changes = true;
                }
            }

            SettingsMessage::NewAliasKeywordChanged(value) => {
                self.new_alias_keyword = value;
            }

            SettingsMessage::NewAliasNameChanged(value) => {
                self.new_alias_name = value;
            }

            SettingsMessage::NewAliasTargetChanged(value) => {
                self.new_alias_target = value;
            }

            SettingsMessage::SaveAlias => {
                if !self.new_alias_keyword.trim().is_empty()
                    && !self.new_alias_name.trim().is_empty()
                    && !self.new_alias_target.trim().is_empty()
                {
                    let alias = AliasConfig {
                        keyword: self.new_alias_keyword.trim().to_string(),
                        name: self.new_alias_name.trim().to_string(),
                        target: self.new_alias_target.trim().to_string(),
                        icon: None,
                    };

                    if let Some(index) = self.editing_alias {
                        self.aliases[index] = alias;
                    } else {
                        self.aliases.push(alias);
                    }

                    self.has_changes = true;
                    self.editing_alias = None;
                    self.clear_alias_form();
                }
            }

            SettingsMessage::CancelAliasEdit => {
                self.editing_alias = None;
                self.clear_alias_form();
            }

            // Quicklink management
            SettingsMessage::EditQuicklink(index) => {
                if let Some(quicklink) = self.quicklinks.get(index) {
                    self.editing_quicklink = Some(index);
                    self.new_quicklink_keyword = quicklink.keyword.clone();
                    self.new_quicklink_name = quicklink.name.clone();
                    self.new_quicklink_url = quicklink.url.clone();
                }
            }

            SettingsMessage::DeleteQuicklink(index) => {
                if index < self.quicklinks.len() {
                    self.quicklinks.remove(index);
                    self.has_changes = true;
                }
            }

            SettingsMessage::NewQuicklinkKeywordChanged(value) => {
                self.new_quicklink_keyword = value;
            }

            SettingsMessage::NewQuicklinkNameChanged(value) => {
                self.new_quicklink_name = value;
            }

            SettingsMessage::NewQuicklinkUrlChanged(value) => {
                self.new_quicklink_url = value;
            }

            SettingsMessage::SaveQuicklink => {
                if !self.new_quicklink_keyword.trim().is_empty()
                    && !self.new_quicklink_name.trim().is_empty()
                    && !self.new_quicklink_url.trim().is_empty()
                {
                    let quicklink = QuicklinkConfig {
                        keyword: self.new_quicklink_keyword.trim().to_string(),
                        name: self.new_quicklink_name.trim().to_string(),
                        url: self.new_quicklink_url.trim().to_string(),
                        icon: None,
                    };

                    if let Some(index) = self.editing_quicklink {
                        self.quicklinks[index] = quicklink;
                    } else {
                        self.quicklinks.push(quicklink);
                    }

                    self.has_changes = true;
                    self.editing_quicklink = None;
                    self.clear_quicklink_form();
                }
            }

            SettingsMessage::CancelQuicklinkEdit => {
                self.editing_quicklink = None;
                self.clear_quicklink_form();
            }

            SettingsMessage::Save => {
                // Update config with new values
                let mut config = self.config.clone();
                config.behavior.max_results = self.max_results;
                config.behavior.autostart = self.autostart;
                config.appearance.theme = self.selected_theme.clone();
                config.appearance.opacity = self.opacity;
                config.aliases = self.aliases.clone();
                config.quicklinks = self.quicklinks.clone();

                // Save to disk
                if let Err(e) = config.save() {
                    eprintln!("[Nova] Failed to save config: {}", e);
                }

                return Some(SettingsResult::Saved(config));
            }

            SettingsMessage::Cancel => {
                return Some(SettingsResult::Cancelled);
            }
        }

        None
    }

    /// Build the view for the settings window.
    pub fn view(&self) -> Element<'_, SettingsMessage> {
        let sidebar = self.build_sidebar();
        let content = self.build_content();
        let footer = self.build_footer();

        let main_content = row![sidebar, content].spacing(0);

        let layout = column![main_content, footer].spacing(0);

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| style::settings_container(&self.theme))
            .into()
    }

    fn build_sidebar(&self) -> Element<'_, SettingsMessage> {
        let tabs = [
            SettingsTab::General,
            SettingsTab::Appearance,
            SettingsTab::Aliases,
            SettingsTab::Quicklinks,
        ];

        let tab_buttons: Vec<Element<'_, SettingsMessage>> = tabs
            .iter()
            .map(|tab| {
                let is_active = *tab == self.active_tab;
                let label = tab.to_string();

                button(text(label).size(14))
                    .padding([12, 20])
                    .width(Length::Fill)
                    .style(move |_, status| style::sidebar_button(&self.theme, is_active, status))
                    .on_press(SettingsMessage::TabSelected(*tab))
                    .into()
            })
            .collect();

        let sidebar_content = Column::with_children(tab_buttons).spacing(4).padding(8);

        container(sidebar_content)
            .width(Length::Fixed(150.0))
            .height(Length::Fill)
            .style(|_| style::sidebar_container(&self.theme))
            .into()
    }

    fn build_content(&self) -> Element<'_, SettingsMessage> {
        let content = match self.active_tab {
            SettingsTab::General => self.build_general_tab(),
            SettingsTab::Appearance => self.build_appearance_tab(),
            SettingsTab::Aliases => self.build_aliases_tab(),
            SettingsTab::Quicklinks => self.build_quicklinks_tab(),
        };

        container(scrollable(content).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }

    fn build_general_tab(&self) -> Element<'_, SettingsMessage> {
        let title = text("General Settings").size(20).color(self.theme.text);

        let max_results_label = text("Max Results").size(14).color(self.theme.subtext);
        let max_results_slider = slider(1..=20, self.max_results as i32, |v| {
            SettingsMessage::MaxResultsChanged(v as u32)
        })
        .width(200);
        let max_results_value = text(format!("{}", self.max_results))
            .size(14)
            .color(self.theme.text);
        let max_results_row = row![
            max_results_label,
            Space::with_width(20),
            max_results_slider,
            Space::with_width(10),
            max_results_value
        ]
        .align_y(Alignment::Center);

        let autostart_label = text("Auto-start on login")
            .size(14)
            .color(self.theme.subtext);
        let autostart_toggle = toggler(self.autostart).on_toggle(SettingsMessage::AutostartToggled);
        let autostart_row = row![
            autostart_label,
            Space::with_width(Length::Fill),
            autostart_toggle
        ]
        .align_y(Alignment::Center);

        column![
            title,
            Space::with_height(20),
            max_results_row,
            Space::with_height(16),
            autostart_row,
        ]
        .spacing(8)
        .into()
    }

    fn build_appearance_tab(&self) -> Element<'_, SettingsMessage> {
        let title = text("Appearance").size(20).color(self.theme.text);

        // Theme picker
        let theme_label = text("Theme").size(14).color(self.theme.subtext);
        let theme_options: Vec<String> = THEMES.iter().map(|s| s.to_string()).collect();
        let theme_picker = pick_list(
            theme_options,
            Some(self.selected_theme.clone()),
            SettingsMessage::ThemeSelected,
        )
        .width(200);
        let theme_row =
            row![theme_label, Space::with_width(20), theme_picker].align_y(Alignment::Center);

        // Opacity slider
        let opacity_label = text("Opacity").size(14).color(self.theme.subtext);
        let opacity_slider = slider(0.5..=1.0, self.opacity as f32, |v| {
            SettingsMessage::OpacityChanged(v as f64)
        })
        .width(200)
        .step(0.05);
        let opacity_value = text(format!("{}%", (self.opacity * 100.0) as i32))
            .size(14)
            .color(self.theme.text);
        let opacity_row = row![
            opacity_label,
            Space::with_width(20),
            opacity_slider,
            Space::with_width(10),
            opacity_value
        ]
        .align_y(Alignment::Center);

        // Theme preview
        let preview_label = text("Preview").size(14).color(self.theme.subtext);
        let preview_box = container(
            column![
                text("Sample Result").size(15).color(self.theme.text),
                text("This is how results will look")
                    .size(13)
                    .color(self.theme.subtext),
            ]
            .spacing(4),
        )
        .padding(12)
        .style(|_| style::result_row(&self.theme, true));

        column![
            title,
            Space::with_height(20),
            theme_row,
            Space::with_height(16),
            opacity_row,
            Space::with_height(24),
            preview_label,
            Space::with_height(8),
            preview_box,
        ]
        .spacing(8)
        .into()
    }

    fn build_aliases_tab(&self) -> Element<'_, SettingsMessage> {
        let title = text("Aliases").size(20).color(self.theme.text);
        let description = text("Short keywords to launch apps quickly")
            .size(13)
            .color(self.theme.subtext);

        // Alias list
        let alias_items: Vec<Element<'_, SettingsMessage>> = self
            .aliases
            .iter()
            .enumerate()
            .map(|(i, alias)| self.build_alias_row(i, alias))
            .collect();

        let alias_list = if alias_items.is_empty() {
            column![text("No aliases configured")
                .size(13)
                .color(self.theme.subtext)]
        } else {
            Column::with_children(alias_items).spacing(4)
        };

        // Add/Edit form
        let form = self.build_alias_form();

        column![
            title,
            description,
            Space::with_height(16),
            alias_list,
            Space::with_height(16),
            horizontal_rule(1),
            Space::with_height(16),
            form,
        ]
        .spacing(8)
        .into()
    }

    fn build_alias_row(
        &self,
        index: usize,
        alias: &AliasConfig,
    ) -> Element<'static, SettingsMessage> {
        let title_text = format!("{} -> {}", alias.keyword, alias.name);
        let target_text = alias.target.clone();
        let theme = self.theme.clone();

        let info = column![
            text(title_text).size(14).color(theme.text),
            text(target_text).size(12).color(theme.subtext),
        ]
        .spacing(2);

        let theme_for_edit = theme.clone();
        let edit_btn = button(text("Edit").size(12))
            .padding([4, 8])
            .style(move |_, status| style::small_button(&theme_for_edit, status))
            .on_press(SettingsMessage::EditAlias(index));

        let theme_for_delete = theme.clone();
        let delete_btn = button(text("Delete").size(12))
            .padding([4, 8])
            .style(move |_, status| style::danger_button(&theme_for_delete, status))
            .on_press(SettingsMessage::DeleteAlias(index));

        let buttons = row![edit_btn, delete_btn].spacing(8);

        container(row![info, Space::with_width(Length::Fill), buttons].align_y(Alignment::Center))
            .padding(12)
            .style(move |_| style::list_item(&theme))
            .into()
    }

    fn build_alias_form(&self) -> Element<'_, SettingsMessage> {
        let form_title = if self.editing_alias.is_some() {
            "Edit Alias"
        } else {
            "Add New Alias"
        };

        let keyword_input = text_input("Keyword (e.g., ff)", &self.new_alias_keyword)
            .on_input(SettingsMessage::NewAliasKeywordChanged)
            .padding(8)
            .width(150);

        let name_input = text_input("Name (e.g., Firefox)", &self.new_alias_name)
            .on_input(SettingsMessage::NewAliasNameChanged)
            .padding(8)
            .width(150);

        let target_input = text_input("Command (e.g., firefox)", &self.new_alias_target)
            .on_input(SettingsMessage::NewAliasTargetChanged)
            .padding(8)
            .width(200);

        let save_btn = button(text("Save").size(13))
            .padding([8, 16])
            .style(|_, status| style::primary_button(&self.theme, status))
            .on_press(SettingsMessage::SaveAlias);

        let cancel_btn = button(text("Cancel").size(13))
            .padding([8, 16])
            .style(|_, status| style::small_button(&self.theme, status))
            .on_press(SettingsMessage::CancelAliasEdit);

        let buttons = if self.editing_alias.is_some() {
            row![save_btn, cancel_btn].spacing(8)
        } else {
            row![save_btn]
        };

        column![
            text(form_title).size(14).color(self.theme.accent),
            Space::with_height(8),
            row![keyword_input, name_input, target_input].spacing(8),
            Space::with_height(8),
            buttons,
        ]
        .spacing(4)
        .into()
    }

    fn build_quicklinks_tab(&self) -> Element<'_, SettingsMessage> {
        let title = text("Quicklinks").size(20).color(self.theme.text);
        let description = text("URL shortcuts with optional {query} for search")
            .size(13)
            .color(self.theme.subtext);

        // Quicklink list
        let quicklink_items: Vec<Element<'_, SettingsMessage>> = self
            .quicklinks
            .iter()
            .enumerate()
            .map(|(i, ql)| self.build_quicklink_row(i, ql))
            .collect();

        let quicklink_list = if quicklink_items.is_empty() {
            column![text("No quicklinks configured")
                .size(13)
                .color(self.theme.subtext)]
        } else {
            Column::with_children(quicklink_items).spacing(4)
        };

        // Add/Edit form
        let form = self.build_quicklink_form();

        column![
            title,
            description,
            Space::with_height(16),
            quicklink_list,
            Space::with_height(16),
            horizontal_rule(1),
            Space::with_height(16),
            form,
        ]
        .spacing(8)
        .into()
    }

    fn build_quicklink_row(
        &self,
        index: usize,
        quicklink: &QuicklinkConfig,
    ) -> Element<'static, SettingsMessage> {
        let has_query = if quicklink.url.contains("{query}") {
            " (search)"
        } else {
            ""
        };

        let title_text = format!("{} -> {}{}", quicklink.keyword, quicklink.name, has_query);
        let url_text = quicklink.url.clone();
        let theme = self.theme.clone();

        let info = column![
            text(title_text).size(14).color(theme.text),
            text(url_text).size(12).color(theme.subtext),
        ]
        .spacing(2);

        let theme_for_edit = theme.clone();
        let edit_btn = button(text("Edit").size(12))
            .padding([4, 8])
            .style(move |_, status| style::small_button(&theme_for_edit, status))
            .on_press(SettingsMessage::EditQuicklink(index));

        let theme_for_delete = theme.clone();
        let delete_btn = button(text("Delete").size(12))
            .padding([4, 8])
            .style(move |_, status| style::danger_button(&theme_for_delete, status))
            .on_press(SettingsMessage::DeleteQuicklink(index));

        let buttons = row![edit_btn, delete_btn].spacing(8);

        container(row![info, Space::with_width(Length::Fill), buttons].align_y(Alignment::Center))
            .padding(12)
            .style(move |_| style::list_item(&theme))
            .into()
    }

    fn build_quicklink_form(&self) -> Element<'_, SettingsMessage> {
        let form_title = if self.editing_quicklink.is_some() {
            "Edit Quicklink"
        } else {
            "Add New Quicklink"
        };

        let keyword_input = text_input("Keyword (e.g., gh)", &self.new_quicklink_keyword)
            .on_input(SettingsMessage::NewQuicklinkKeywordChanged)
            .padding(8)
            .width(120);

        let name_input = text_input("Name (e.g., GitHub)", &self.new_quicklink_name)
            .on_input(SettingsMessage::NewQuicklinkNameChanged)
            .padding(8)
            .width(150);

        let url_input = text_input("URL (use {query} for search)", &self.new_quicklink_url)
            .on_input(SettingsMessage::NewQuicklinkUrlChanged)
            .padding(8)
            .width(250);

        let save_btn = button(text("Save").size(13))
            .padding([8, 16])
            .style(|_, status| style::primary_button(&self.theme, status))
            .on_press(SettingsMessage::SaveQuicklink);

        let cancel_btn = button(text("Cancel").size(13))
            .padding([8, 16])
            .style(|_, status| style::small_button(&self.theme, status))
            .on_press(SettingsMessage::CancelQuicklinkEdit);

        let buttons = if self.editing_quicklink.is_some() {
            row![save_btn, cancel_btn].spacing(8)
        } else {
            row![save_btn]
        };

        column![
            text(form_title).size(14).color(self.theme.accent),
            Space::with_height(8),
            row![keyword_input, name_input].spacing(8),
            url_input,
            Space::with_height(8),
            buttons,
        ]
        .spacing(4)
        .into()
    }

    fn build_footer(&self) -> Element<'_, SettingsMessage> {
        let cancel_btn = button(text("Cancel").size(14))
            .padding([10, 20])
            .style(|_, status| style::small_button(&self.theme, status))
            .on_press(SettingsMessage::Cancel);

        let save_btn = button(text("Save").size(14))
            .padding([10, 20])
            .style(|_, status| style::primary_button(&self.theme, status))
            .on_press(SettingsMessage::Save);

        let buttons = row![cancel_btn, save_btn].spacing(12);

        container(row![Space::with_width(Length::Fill), buttons].padding([12, 20]))
            .width(Length::Fill)
            .style(|_| style::footer_container(&self.theme))
            .into()
    }

    fn clear_alias_form(&mut self) {
        self.new_alias_keyword.clear();
        self.new_alias_name.clear();
        self.new_alias_target.clear();
    }

    fn clear_quicklink_form(&mut self) {
        self.new_quicklink_keyword.clear();
        self.new_quicklink_name.clear();
        self.new_quicklink_url.clear();
    }
}
