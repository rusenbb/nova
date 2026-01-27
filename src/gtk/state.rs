//! UI state management for GTK.

use crate::config;
use crate::core::search::SearchResult;
use crate::executor::{ExecutionAction, SystemCommand};
use crate::services::Extension;
use gtk::prelude::*;
use gtk::{ApplicationWindow, Entry, Label, ListBox};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use super::window::{position_window, render_results_list};

/// Represents the current command mode state
#[derive(Debug, Clone, Default)]
pub struct CommandModeState {
    /// The active extension (if in command mode)
    pub active_extension: Option<Extension>,
}

impl CommandModeState {
    pub fn enter_mode(&mut self, extension: Extension) {
        self.active_extension = Some(extension);
    }

    pub fn exit_mode(&mut self) {
        self.active_extension = None;
    }

    pub fn is_active(&self) -> bool {
        self.active_extension.is_some()
    }
}

/// Consolidated UI state to reduce RefCell sprawl
///
/// This struct holds all mutable UI state that needs to be shared across
/// GTK event handlers. By consolidating state here, we reduce the number
/// of individual Rc<RefCell<T>> variables from 15+ to just one.
pub struct UIState {
    // Widget references (GTK widgets are internally ref-counted, so Clone is cheap)
    pub window: ApplicationWindow,
    pub entry: Entry,
    pub results_list: ListBox,
    pub command_pill: Label,

    // Mutable state
    pub is_visible: bool,
    pub selected_index: i32,
    pub current_results: Vec<SearchResult>,
    pub command_mode: CommandModeState,
    pub is_clearing: bool, // Guard to prevent callback loops when programmatically clearing entry
    pub last_toggle: Instant,
}

impl UIState {
    /// Update the results list with new search results
    pub fn update_results(&mut self, results: Vec<SearchResult>) {
        render_results_list(&self.results_list, &results);
        self.current_results = results;
        self.selected_index = 0;
        if let Some(row) = self.results_list.row_at_index(0) {
            self.results_list.select_row(Some(&row));
        }
    }

    /// Clear entry text with the is_clearing guard to prevent callback loops
    pub fn clear_entry(&mut self) {
        self.is_clearing = true;
        self.entry.set_text("");
        self.is_clearing = false;
    }

    /// Update command pill visibility and text based on command mode state
    pub fn update_command_pill(&self) {
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
    pub fn enter_command_mode(&mut self, extension: Extension) {
        self.command_mode.enter_mode(extension);
        self.clear_entry();
        self.update_command_pill();
    }

    /// Exit command mode and reset to normal search
    pub fn exit_command_mode(&mut self) {
        self.command_mode.exit_mode();
        self.update_command_pill();
    }

    /// Navigate selection up/down
    pub fn navigate_selection(&mut self, delta: i32) {
        let n_items = self.results_list.children().len() as i32;
        if n_items > 0 {
            self.selected_index = (self.selected_index + delta).rem_euclid(n_items);
            if let Some(row) = self.results_list.row_at_index(self.selected_index) {
                self.results_list.select_row(Some(&row));
            }
        }
    }

    /// Get the currently selected search result
    #[allow(dead_code)]
    pub fn selected_result(&self) -> Option<&SearchResult> {
        self.current_results.get(self.selected_index as usize)
    }

    /// Hide the window, reset state, and optionally save config
    pub fn hide_window(&mut self, config: &config::Config) {
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
    pub fn show_window(&mut self, config: &config::Config) {
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
pub type UIStateHandle = Rc<RefCell<UIState>>;

/// Get the action to perform when a search result is executed
pub fn result_to_action(result: &SearchResult) -> ExecutionAction {
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
