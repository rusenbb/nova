use std::sync::Arc;

use iced::widget::{column, container, scrollable, text, text_input, Column};
use iced::{keyboard, Element, Length, Subscription, Task, Theme};

use nova_core::{
    CommandModeState, Config, ExecutionAction, PlatformAppEntry, SearchEngine, SearchResult,
};
use nova_platform::Platform;

use crate::execute;
use crate::style;
use crate::widgets;

/// The main application state
pub struct Nova {
    config: Config,
    search_engine: SearchEngine,
    platform: Arc<Platform>,
    apps: Vec<PlatformAppEntry>,

    // UI state
    query: String,
    results: Vec<SearchResult>,
    selected_index: usize,
    is_visible: bool,
    command_mode: CommandModeState,
    settings_open: bool,
    clipboard_history: nova_core::services::clipboard::ClipboardHistory,
}

/// Messages that drive the application
#[derive(Debug, Clone)]
pub enum Message {
    QueryChanged(String),
    KeyPressed(keyboard::Key, keyboard::Modifiers),
    ExecuteSelected,
    SelectIndex(usize),
    Hide,
    Show,
    Toggle,
    ClipboardChanged(String),
    IpcReceived(String),
    SettingsToggle,
    SettingsSaved(Config),
    Noop,
}

impl Nova {
    pub fn new(
        config: Config,
        platform: Platform,
        apps: Vec<PlatformAppEntry>,
    ) -> (Self, Task<Message>) {
        let search_engine = SearchEngine::new(&config);
        let max_results = config.behavior.max_results as usize;
        let clipboard_history =
            nova_core::services::clipboard::ClipboardHistory::new(50);

        let mut nova = Self {
            config,
            search_engine,
            platform: Arc::new(platform),
            apps,
            query: String::new(),
            results: Vec::new(),
            selected_index: 0,
            is_visible: true,
            command_mode: CommandModeState::default(),
            settings_open: false,
            clipboard_history,
        };

        // Initial search with empty query to show default results
        nova.perform_search(max_results);

        (nova, text_input::focus(text_input::Id::new("search_input")))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::QueryChanged(query) => {
                self.query = query;
                self.selected_index = 0;
                self.perform_search(self.config.behavior.max_results as usize);
                Task::none()
            }
            Message::KeyPressed(key, _modifiers) => match key {
                keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                    if !self.results.is_empty() {
                        self.selected_index =
                            (self.selected_index + 1).min(self.results.len() - 1);
                    }
                    Task::none()
                }
                keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                    self.selected_index = self.selected_index.saturating_sub(1);
                    Task::none()
                }
                keyboard::Key::Named(keyboard::key::Named::Escape) => {
                    if self.settings_open {
                        self.settings_open = false;
                        Task::none()
                    } else if self.command_mode.is_active() {
                        self.command_mode.exit_mode();
                        self.query.clear();
                        self.selected_index = 0;
                        self.perform_search(self.config.behavior.max_results as usize);
                        Task::none()
                    } else {
                        self.hide()
                    }
                }
                keyboard::Key::Named(keyboard::key::Named::Enter) => {
                    self.execute_selected()
                }
                keyboard::Key::Named(keyboard::key::Named::Tab) => {
                    self.try_enter_command_mode();
                    Task::none()
                }
                keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                    if self.query.is_empty() && self.command_mode.is_active() {
                        self.command_mode.exit_mode();
                        self.perform_search(self.config.behavior.max_results as usize);
                    }
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::ExecuteSelected => self.execute_selected(),
            Message::SelectIndex(index) => {
                self.selected_index = index;
                self.execute_selected()
            }
            Message::Hide => self.hide(),
            Message::Show => self.show(),
            Message::Toggle => {
                if self.is_visible {
                    self.hide()
                } else {
                    self.show()
                }
            }
            Message::ClipboardChanged(content) => {
                self.clipboard_history.check_and_add(content);
                Task::none()
            }
            Message::IpcReceived(msg) => {
                if msg.trim() == "toggle" {
                    if self.is_visible {
                        self.hide()
                    } else {
                        self.show()
                    }
                } else {
                    Task::none()
                }
            }
            Message::SettingsToggle => {
                self.settings_open = !self.settings_open;
                Task::none()
            }
            Message::SettingsSaved(config) => {
                self.config = config.clone();
                self.search_engine = SearchEngine::new(&config);
                if let Err(e) = config.save() {
                    eprintln!("[Nova] Failed to save config: {}", e);
                }
                self.settings_open = false;
                Task::none()
            }
            Message::Noop => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.settings_open {
            return crate::settings::view(&self.config);
        }

        let theme_palette = nova_core::get_theme_palette(&self.config.appearance.theme);

        // Search input
        let input = text_input("Search...", &self.query)
            .id(text_input::Id::new("search_input"))
            .on_input(Message::QueryChanged)
            .on_submit(Message::ExecuteSelected)
            .size(18)
            .padding(12);

        // Command mode pill
        let search_row = if let Some(ref ext) = self.command_mode.active_extension {
            let pill = container(
                text(ext.pill_text())
                    .size(13),
            )
            .padding([2, 8])
            .style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba8(
                    theme_palette.accent.0,
                    theme_palette.accent.1,
                    theme_palette.accent.2,
                    0.25,
                ))),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

            iced::widget::row![pill, input].spacing(8).into()
        } else {
            Element::from(input)
        };

        // Results list
        let results_column: Column<Message> = self
            .results
            .iter()
            .enumerate()
            .fold(Column::new().spacing(0), |col, (i, result)| {
                col.push(widgets::result_row(
                    result,
                    i == self.selected_index,
                    &theme_palette,
                    i,
                ))
            });

        let results_scrollable = scrollable(results_column)
            .height(Length::Fill);

        let content = column![search_row, results_scrollable]
            .spacing(4)
            .padding(8);

        let bg = theme_palette.background;
        let opacity = self.config.appearance.opacity as f32;

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba8(
                    bg.0, bg.1, bg.2, opacity,
                ))),
                border: iced::Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keyboard_sub = keyboard::on_key_press(|key, modifiers| {
            match &key {
                keyboard::Key::Named(
                    keyboard::key::Named::ArrowDown
                    | keyboard::key::Named::ArrowUp
                    | keyboard::key::Named::Escape
                    | keyboard::key::Named::Tab,
                ) => Some(Message::KeyPressed(key, modifiers)),
                keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                    Some(Message::KeyPressed(key, modifiers))
                }
                _ => None,
            }
        });

        let ipc_sub = crate::subscriptions::ipc_listener();
        let clipboard_sub = crate::subscriptions::clipboard_poll(self.platform.clone());

        Subscription::batch([keyboard_sub, ipc_sub, clipboard_sub])
    }

    pub fn theme(&self) -> Theme {
        style::theme::nova_theme(&self.config)
    }

    fn perform_search(&mut self, max_results: usize) {
        if self.command_mode.is_active() {
            self.results = self.search_engine.search_in_command_mode(
                &self.command_mode,
                &self.query,
                max_results,
            );
        } else {
            self.results = self.search_engine.search(
                &self.apps,
                &self.clipboard_history,
                &self.query,
                max_results,
            );
        }
    }

    fn execute_selected(&mut self) -> Task<Message> {
        if self.results.is_empty() {
            return Task::none();
        }

        let result = &self.results[self.selected_index];
        let action = result.execution_action();

        // Check if this should enter command mode
        if matches!(action, ExecutionAction::NeedsInput) {
            self.try_enter_command_mode();
            return Task::none();
        }

        execute::run_action(action, &self.platform, &self.config)
    }

    fn try_enter_command_mode(&mut self) {
        if self.results.is_empty() || self.command_mode.is_active() {
            return;
        }

        let result = &self.results[self.selected_index];

        // Check for extension keyword match
        let keyword = match result {
            SearchResult::Quicklink { keyword, has_query, .. } if *has_query => {
                Some(keyword.clone())
            }
            SearchResult::Script { id, has_argument, .. } if *has_argument => {
                Some(id.clone())
            }
            SearchResult::ExtensionCommand { command } if command.has_argument => {
                Some(command.keyword.clone())
            }
            _ => None,
        };

        if let Some(keyword) = keyword {
            if let Some(ext) = self.search_engine.extension_index.get_by_keyword(&keyword) {
                self.command_mode.enter_mode(ext.clone());
                self.query.clear();
                self.selected_index = 0;
                self.perform_search(self.config.behavior.max_results as usize);
            }
        }
    }

    fn hide(&mut self) -> Task<Message> {
        self.is_visible = false;
        self.query.clear();
        self.selected_index = 0;
        self.command_mode.exit_mode();
        // Iced 0.13 has no set_visible; minimize as a proxy for hiding
        iced::window::get_oldest()
            .and_then(|id| iced::window::minimize(id, true))
    }

    fn show(&mut self) -> Task<Message> {
        self.is_visible = true;
        self.perform_search(self.config.behavior.max_results as usize);
        Task::batch([
            iced::window::get_oldest()
                .and_then(|id| {
                    Task::batch([
                        iced::window::minimize(id, false),
                        iced::window::gain_focus(id),
                    ])
                }),
            text_input::focus(text_input::Id::new("search_input")),
        ])
    }
}
