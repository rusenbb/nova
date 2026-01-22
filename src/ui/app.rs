//! Main Nova application using iced.

use super::style;
use super::theme::NovaTheme;
use crate::config::Config;
use crate::core::{SearchEngine, SearchResult};
use crate::executor::{execute, ExecutionAction, ExecutionResult};
use crate::platform::{self, AppEntry, Platform};
use crate::services::clipboard::ClipboardHistory;
use crate::services::custom_commands::CustomCommandsIndex;
use crate::services::extension::{Extension, ExtensionKind};
use crate::services::extensions::{get_extensions_dir, ExtensionManager};

use iced::keyboard::{self, Key};
use iced::widget::{column, container, row, scrollable, text, text_input, Column, Space};
use iced::window;
use iced::{Element, Length, Subscription, Task};

/// The main Nova application state.
pub struct NovaApp {
    // Platform abstraction
    platform: Box<dyn Platform>,

    // Configuration
    config: Config,
    theme: NovaTheme,

    // Search state
    search_query: String,
    search_engine: SearchEngine,
    results: Vec<SearchResult>,
    selected_index: usize,

    // Data sources
    apps: Vec<AppEntry>,
    custom_commands: CustomCommandsIndex,
    extension_manager: ExtensionManager,
    clipboard_history: ClipboardHistory,

    // Command mode
    command_mode: Option<Extension>,

    // Window state
    visible: bool,
    input_id: text_input::Id,
}

/// Messages that the application can handle.
#[derive(Debug, Clone)]
pub enum Message {
    // Input events
    SearchChanged(String),
    InputSubmit,

    // Navigation
    SelectNext,
    SelectPrevious,
    Execute,
    ExecuteSelected,

    // Window management
    ToggleWindow,
    ShowWindow,
    HideWindow,
    WindowFocused,
    WindowUnfocused,

    // Mode changes
    EnterCommandMode,
    ExitCommandMode,
    TabPressed,

    // Keyboard events
    KeyPressed(Key),
    EscapePressed,

    // Clipboard polling
    PollClipboard,

    // Settings
    OpenSettings,

    // Lifecycle
    Quit,
    NoOp,
}

impl NovaApp {
    /// Create a new Nova application.
    pub fn new() -> (Self, Task<Message>) {
        let platform = platform::current();
        let config = Config::load();
        let theme = NovaTheme::by_name(&config.appearance.theme);

        // Load apps from platform
        let apps = platform.discover_apps();
        println!("[Nova] Discovered {} applications", apps.len());

        // Load custom commands
        let custom_commands = CustomCommandsIndex::new(&config);

        // Load extensions
        let extension_manager = ExtensionManager::load(&get_extensions_dir());

        let app = Self {
            platform,
            config,
            theme,
            search_query: String::new(),
            search_engine: SearchEngine::new(),
            results: Vec::new(),
            selected_index: 0,
            apps,
            custom_commands,
            extension_manager,
            clipboard_history: ClipboardHistory::new(50),
            command_mode: None,
            visible: true,
            input_id: text_input::Id::unique(),
        };

        // Focus the input on startup
        let task = text_input::focus(app.input_id.clone());

        (app, task)
    }

    /// Update the application state based on a message.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SearchChanged(query) => {
                self.search_query = query;
                self.update_search();
                self.selected_index = 0;
                Task::none()
            }

            Message::SelectNext => {
                if !self.results.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.results.len();
                }
                Task::none()
            }

            Message::SelectPrevious => {
                if !self.results.is_empty() {
                    self.selected_index = if self.selected_index == 0 {
                        self.results.len() - 1
                    } else {
                        self.selected_index - 1
                    };
                }
                Task::none()
            }

            Message::Execute | Message::InputSubmit | Message::ExecuteSelected => {
                self.execute_selected()
            }

            Message::TabPressed => {
                // Enter command mode if result needs input
                if let Some(result) = self.results.get(self.selected_index) {
                    if result.needs_input() {
                        return self.enter_command_mode_for_result(result.clone());
                    }
                }
                Task::none()
            }

            Message::EnterCommandMode => {
                if let Some(result) = self.results.get(self.selected_index).cloned() {
                    return self.enter_command_mode_for_result(result);
                }
                Task::none()
            }

            Message::ExitCommandMode | Message::EscapePressed => {
                if self.command_mode.is_some() {
                    self.command_mode = None;
                    self.search_query.clear();
                    self.update_search();
                    Task::none()
                } else if !self.search_query.is_empty() {
                    self.search_query.clear();
                    self.update_search();
                    Task::none()
                } else {
                    self.hide_window()
                }
            }

            Message::KeyPressed(key) => self.handle_key_press(key),

            Message::ToggleWindow => {
                if self.visible {
                    self.hide_window()
                } else {
                    self.show_window()
                }
            }

            Message::ShowWindow => self.show_window(),

            Message::HideWindow => self.hide_window(),

            Message::WindowFocused => {
                self.visible = true;
                text_input::focus(self.input_id.clone())
            }

            Message::WindowUnfocused => {
                // Optionally hide on focus loss
                // self.hide_window()
                Task::none()
            }

            Message::PollClipboard => {
                if let Some(content) = self.platform.clipboard_read() {
                    self.clipboard_history.add(content);
                }
                Task::none()
            }

            Message::OpenSettings => {
                // TODO: Open settings window
                println!("[Nova] Settings requested");
                Task::none()
            }

            Message::Quit => window::get_latest().and_then(window::close),

            Message::NoOp => Task::none(),
        }
    }

    /// Create the view for the application.
    pub fn view(&self) -> Element<'_, Message> {
        let max_results = self.config.behavior.max_results as usize;

        // Build the search input (with optional command mode pill)
        let search_input = self.build_search_input();

        // Build the results list
        let results_list = self.build_results_list(max_results);

        // Main container
        let content = column![search_input, results_list].spacing(8).padding(12);

        container(content)
            .style(|_| style::main_container(&self.theme, self.config.appearance.opacity as f32))
            .width(Length::Fixed(self.config.appearance.window_width as f32))
            .into()
    }

    /// Handle subscriptions (keyboard, timers, etc.).
    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key, modifiers| match key.as_ref() {
            Key::Named(keyboard::key::Named::Escape) => Some(Message::EscapePressed),
            Key::Named(keyboard::key::Named::ArrowUp) => Some(Message::SelectPrevious),
            Key::Named(keyboard::key::Named::ArrowDown) => Some(Message::SelectNext),
            Key::Named(keyboard::key::Named::Enter) => Some(Message::Execute),
            Key::Named(keyboard::key::Named::Tab) => Some(Message::TabPressed),
            Key::Named(keyboard::key::Named::Backspace) if modifiers.command() => {
                Some(Message::ExitCommandMode)
            }
            _ => None,
        })
    }

    /// Get the window title.
    pub fn title(&self) -> String {
        "Nova".to_string()
    }

    // --- Private methods ---

    fn build_search_input(&self) -> Element<'_, Message> {
        let input = text_input("Search apps, commands, files...", &self.search_query)
            .id(self.input_id.clone())
            .on_input(Message::SearchChanged)
            .on_submit(Message::InputSubmit)
            .padding(12)
            .size(18)
            .style(|_, status| {
                style::search_input(&self.theme, status == text_input::Status::Focused)
            });

        if let Some(ref ext) = self.command_mode {
            // Show command mode pill
            let pill = container(text(&ext.keyword).size(13).color(self.theme.accent))
                .padding([4, 10])
                .style(|_| style::command_pill(&self.theme));

            row![pill, input]
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .into()
        } else {
            input.into()
        }
    }

    fn build_results_list(&self, max_results: usize) -> Element<'_, Message> {
        let results: Vec<Element<Message>> = self
            .results
            .iter()
            .take(max_results)
            .enumerate()
            .map(|(idx, result)| self.build_result_row(idx, result))
            .collect();

        if results.is_empty() {
            return Space::with_height(0).into();
        }

        let list = Column::with_children(results).spacing(2);

        scrollable(list)
            .style(|_, _| style::results_scrollable(&self.theme))
            .height(Length::Shrink)
            .into()
    }

    fn build_result_row(&self, idx: usize, result: &SearchResult) -> Element<'static, Message> {
        let selected = idx == self.selected_index;
        let theme = self.theme.clone();

        let name_text = result.name().to_string();
        let desc_text = result.description().map(|s| s.to_string());

        let name = text(name_text).size(15).color(theme.text);

        let description = desc_text
            .map(|d| text(d).size(13).color(theme.subtext))
            .unwrap_or_else(|| text("").size(13).color(theme.subtext));

        let content = column![name, description].spacing(2);

        let theme_for_style = theme.clone();
        container(content)
            .padding([8, 12])
            .width(Length::Fill)
            .style(move |_| style::result_row(&theme_for_style, selected))
            .into()
    }

    fn update_search(&mut self) {
        let max_results = self.config.behavior.max_results as usize;

        if let Some(ref ext) = self.command_mode {
            self.results =
                self.search_engine
                    .search_in_command_mode(ext, &self.search_query, max_results);
        } else {
            self.results = self.search_engine.search(
                &self.apps,
                &self.custom_commands,
                &self.extension_manager,
                &self.clipboard_history,
                &self.search_query,
                max_results,
            );
        }
    }

    fn execute_selected(&mut self) -> Task<Message> {
        let Some(result) = self.results.get(self.selected_index).cloned() else {
            return Task::none();
        };

        // Check if needs input and enter command mode instead
        if result.needs_input() && self.command_mode.is_none() {
            return self.enter_command_mode_for_result(result);
        }

        // Convert SearchResult to ExecutionAction
        let action = self.result_to_action(&result);

        // Execute the action
        let exec_result = execute(
            &action,
            self.platform.as_ref(),
            Some(&self.extension_manager),
        );

        match exec_result {
            ExecutionResult::Success => self.reset_and_hide(),
            ExecutionResult::SuccessKeepOpen => {
                self.search_query.clear();
                self.command_mode = None;
                self.update_search();
                Task::none()
            }
            ExecutionResult::OpenSettings => Task::done(Message::OpenSettings),
            ExecutionResult::Quit => Task::done(Message::Quit),
            ExecutionResult::Error(e) => {
                eprintln!("[Nova] Execution error: {}", e);
                let _ = self.platform.show_notification("Nova Error", &e);
                Task::none()
            }
            ExecutionResult::NeedsInput => self.enter_command_mode_for_result(result),
        }
    }

    fn result_to_action(&self, result: &SearchResult) -> ExecutionAction {
        match result {
            SearchResult::App(app) => ExecutionAction::LaunchApp { app: app.clone() },

            SearchResult::Command { id, .. } => {
                if id == "nova:settings" {
                    ExecutionAction::OpenSettings
                } else if id == "nova:quit" {
                    ExecutionAction::Quit
                } else if let Some(cmd) = SearchEngine::parse_system_command(id) {
                    ExecutionAction::SystemCommand { command: cmd }
                } else {
                    ExecutionAction::NeedsInput
                }
            }

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

            SearchResult::Calculation { result, .. } => ExecutionAction::CopyToClipboard {
                content: result.trim_start_matches("= ").to_string(),
                notification: "Calculation result copied".to_string(),
            },

            SearchResult::ClipboardItem { content, .. } => ExecutionAction::CopyToClipboard {
                content: content.clone(),
                notification: "Clipboard item copied".to_string(),
            },

            SearchResult::FileResult { path, .. } => {
                ExecutionAction::OpenFile { path: path.clone() }
            }

            SearchResult::EmojiResult { emoji, .. } => ExecutionAction::CopyToClipboard {
                content: emoji.clone(),
                notification: format!("{} copied", emoji),
            },

            SearchResult::UnitConversion { result, .. } => ExecutionAction::CopyToClipboard {
                content: result.clone(),
                notification: "Conversion result copied".to_string(),
            },
        }
    }

    fn enter_command_mode_for_result(&mut self, result: SearchResult) -> Task<Message> {
        // Create an Extension from the result for command mode
        let extension = match &result {
            SearchResult::Quicklink {
                keyword, name, url, ..
            } => Some(Extension {
                keyword: keyword.clone(),
                name: name.clone(),
                icon: None,
                color: None,
                kind: ExtensionKind::Quicklink {
                    url: url.clone(),
                    has_query: true,
                },
            }),
            SearchResult::Script {
                id,
                name,
                description,
                path,
                output_mode,
                ..
            } => Some(Extension {
                keyword: id.clone(),
                name: name.clone(),
                icon: None,
                color: None,
                kind: ExtensionKind::Script {
                    path: path.clone(),
                    has_argument: true,
                    output_mode: output_mode.clone(),
                    description: description.clone(),
                },
            }),
            SearchResult::ExtensionCommand { command } if command.has_argument => Some(Extension {
                keyword: command.keyword.clone(),
                name: command.name.clone(),
                icon: None,
                color: None,
                kind: ExtensionKind::Script {
                    path: command.script_path.clone(),
                    has_argument: true,
                    output_mode: crate::services::custom_commands::ScriptOutputMode::Silent,
                    description: command.description.clone(),
                },
            }),
            _ => None,
        };

        if let Some(ext) = extension {
            self.command_mode = Some(ext);
            self.search_query.clear();
            self.update_search();
        }

        Task::none()
    }

    fn handle_key_press(&mut self, _key: Key) -> Task<Message> {
        // Additional key handling if needed
        Task::none()
    }

    fn show_window(&mut self) -> Task<Message> {
        self.visible = true;
        self.search_query.clear();
        self.command_mode = None;
        self.update_search();
        text_input::focus(self.input_id.clone())
    }

    fn hide_window(&mut self) -> Task<Message> {
        self.visible = false;
        self.search_query.clear();
        self.command_mode = None;
        self.results.clear();
        // In iced, we'd minimize or hide the window
        // For now, just update state
        Task::none()
    }

    fn reset_and_hide(&mut self) -> Task<Message> {
        self.search_query.clear();
        self.command_mode = None;
        self.results.clear();
        self.selected_index = 0;
        self.hide_window()
    }
}

impl Default for NovaApp {
    fn default() -> Self {
        Self::new().0
    }
}
