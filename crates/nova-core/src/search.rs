use std::path::PathBuf;

use crate::config::Config;
use crate::executor::{ExecutionAction, SystemCommand};
use crate::services::*;

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

/// Search results that appear in the launcher
#[derive(Debug, Clone)]
pub enum SearchResult {
    App {
        id: String,
        name: String,
        exec: String,
        icon: Option<String>,
        description: Option<String>,
    },
    Command {
        id: String,
        name: String,
        description: String,
    },
    Alias {
        keyword: String,
        name: String,
        target: String,
    },
    Quicklink {
        keyword: String,
        name: String,
        url: String,
        has_query: bool,
    },
    QuicklinkWithQuery {
        keyword: String,
        name: String,
        url: String,
        query: String,
        resolved_url: String,
    },
    Script {
        id: String,
        name: String,
        description: String,
        path: PathBuf,
        has_argument: bool,
        output_mode: ScriptOutputMode,
    },
    ScriptWithArgument {
        id: String,
        name: String,
        description: String,
        path: PathBuf,
        argument: String,
        output_mode: ScriptOutputMode,
    },
    ExtensionCommand {
        command: LoadedCommand,
    },
    ExtensionCommandWithArg {
        command: LoadedCommand,
        argument: String,
    },
    Calculation {
        expression: String,
        result: String,
    },
    ClipboardItem {
        index: usize,
        content: String,
        preview: String,
        time_ago: String,
    },
    FileResult {
        name: String,
        path: String,
        is_dir: bool,
    },
    EmojiResult {
        emoji: String,
        name: String,
        aliases: String,
    },
    UnitConversion {
        display: String,
        result: String,
    },
}

impl SearchResult {
    pub fn name(&self) -> &str {
        match self {
            SearchResult::App { name, .. } => name,
            SearchResult::Command { name, .. } => name,
            SearchResult::Alias { name, .. } => name,
            SearchResult::Quicklink { name, .. } => name,
            SearchResult::QuicklinkWithQuery { name, .. } => name,
            SearchResult::Script { name, .. } => name,
            SearchResult::ScriptWithArgument { name, .. } => name,
            SearchResult::ExtensionCommand { command } => &command.name,
            SearchResult::ExtensionCommandWithArg { command, .. } => &command.name,
            SearchResult::Calculation { result, .. } => result,
            SearchResult::ClipboardItem { preview, .. } => preview,
            SearchResult::FileResult { name, .. } => name,
            SearchResult::EmojiResult { name, .. } => name,
            SearchResult::UnitConversion { result, .. } => result,
        }
    }

    pub fn description(&self) -> Option<&str> {
        match self {
            SearchResult::App { description, .. } => description.as_deref(),
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
            SearchResult::ExtensionCommand { command } => {
                if command.description.is_empty() { None } else { Some(&command.description) }
            }
            SearchResult::ExtensionCommandWithArg { command, .. } => {
                if command.description.is_empty() { None } else { Some(&command.description) }
            }
            SearchResult::Calculation { expression, .. } => Some(expression),
            SearchResult::ClipboardItem { time_ago, .. } => Some(time_ago),
            SearchResult::FileResult { path, .. } => Some(path),
            SearchResult::EmojiResult { aliases, .. } => Some(aliases),
            SearchResult::UnitConversion { display, .. } => Some(display),
        }
    }

    /// Get the action to perform when this result is executed
    pub fn execution_action(&self) -> ExecutionAction {
        match self {
            SearchResult::App { exec, name, .. } => ExecutionAction::LaunchApp {
                exec: exec.clone(),
                name: name.clone(),
            },
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
                path, has_argument, output_mode, ..
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
                path, argument, output_mode, ..
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
            SearchResult::Calculation { result, expression } => {
                let value = result.trim_start_matches("= ");
                ExecutionAction::CopyToClipboard {
                    content: value.to_string(),
                    notification: format!("{} = {}", expression, value),
                }
            }
            SearchResult::ClipboardItem { content, preview, .. } => {
                ExecutionAction::CopyToClipboard {
                    content: content.clone(),
                    notification: preview.clone(),
                }
            }
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
            SearchResult::UnitConversion { result, display } => {
                ExecutionAction::CopyToClipboard {
                    content: result.clone(),
                    notification: display.clone(),
                }
            }
        }
    }
}

/// Built-in system commands
pub fn get_system_commands() -> Vec<SearchResult> {
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

/// Platform-agnostic app entry for search results
#[derive(Debug, Clone)]
pub struct PlatformAppEntry {
    pub id: String,
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
}

/// Search engine that aggregates results from all providers
pub struct SearchEngine {
    pub custom_commands: CustomCommandsIndex,
    pub extension_manager: ExtensionManager,
    pub extension_index: ExtensionIndex,
}

impl SearchEngine {
    pub fn new(config: &Config) -> Self {
        let custom_commands = CustomCommandsIndex::new(config);
        let extension_manager = ExtensionManager::load(&get_extensions_dir());
        let extension_index = ExtensionIndex::from_custom_commands(
            &custom_commands,
            &config.aliases,
            &config.quicklinks,
        );

        Self {
            custom_commands,
            extension_manager,
            extension_index,
        }
    }

    /// Perform a search with all available providers
    pub fn search(
        &self,
        apps: &[PlatformAppEntry],
        clipboard_history: &clipboard::ClipboardHistory,
        query: &str,
        max_results: usize,
    ) -> Vec<SearchResult> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        let query_parts: Vec<&str> = query.splitn(2, ' ').collect();
        let keyword = query_parts[0].to_lowercase();
        let remaining_query = query_parts.get(1).map(|s| s.to_string());

        // 1. Aliases
        for alias in &self.custom_commands.aliases {
            let alias_keyword = alias.keyword.to_lowercase();
            if alias_keyword == keyword
                || alias_keyword.contains(&query_lower)
                || alias.name.to_lowercase().contains(&query_lower)
            {
                results.push(SearchResult::Alias {
                    keyword: alias.keyword.clone(),
                    name: alias.name.clone(),
                    target: alias.target.clone(),
                });
            }
        }

        // 2. Calculator
        if let Some(calc_result) = calculator::evaluate(query) {
            let formatted = calculator::format_result(calc_result);
            results.push(SearchResult::Calculation {
                expression: query.to_string(),
                result: format!("= {}", formatted),
            });
        }

        // 3. Unit converter
        if query.contains(" to ") {
            if let Some(conversion) = units::convert(query) {
                results.push(SearchResult::UnitConversion {
                    display: conversion.display(),
                    result: conversion.result(),
                });
            }
        }

        // 4. Clipboard history
        let clipboard_keywords = ["clip", "clipboard", "paste", "history"];
        if clipboard_keywords
            .iter()
            .any(|kw| query_lower.starts_with(kw))
        {
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

        // 5. File search
        if query.starts_with('~') || query.starts_with('/') {
            for entry in file_search::search_files(query, 10) {
                let icon_prefix = if entry.is_dir { "[D] " } else { "" };
                results.push(SearchResult::FileResult {
                    name: format!("{}{}", icon_prefix, entry.display_name()),
                    path: entry.display_path(),
                    is_dir: entry.is_dir,
                });
            }
        }

        // 6. Emoji picker
        if query.starts_with(':') && query.len() > 1 {
            let emoji_query = &query[1..];
            for e in emoji::search(emoji_query, 10) {
                results.push(SearchResult::EmojiResult {
                    emoji: e.char.to_string(),
                    name: format!("{} {}", e.char, e.name()),
                    aliases: e.aliases(),
                });
            }
        }

        // 7. Quicklinks
        for quicklink in &self.custom_commands.quicklinks {
            let ql_keyword = quicklink.keyword.to_lowercase();

            if ql_keyword == keyword {
                if quicklink.has_query_placeholder() {
                    if let Some(ref q) = remaining_query {
                        results.push(SearchResult::QuicklinkWithQuery {
                            keyword: quicklink.keyword.clone(),
                            name: format!("{}: {}", quicklink.name, q),
                            url: quicklink.url.clone(),
                            query: q.clone(),
                            resolved_url: quicklink.resolve_url(q),
                        });
                    } else {
                        results.push(SearchResult::Quicklink {
                            keyword: quicklink.keyword.clone(),
                            name: format!("{} (type to search)", quicklink.name),
                            url: quicklink.url.clone(),
                            has_query: true,
                        });
                    }
                } else {
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
                results.push(SearchResult::Quicklink {
                    keyword: quicklink.keyword.clone(),
                    name: quicklink.name.clone(),
                    url: quicklink.url.clone(),
                    has_query: quicklink.has_query_placeholder(),
                });
            }
        }

        // 8. Scripts
        for script in &self.custom_commands.scripts {
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

        // 9. System commands
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

        // 10. Extension commands
        for cmd in self.extension_manager.search_commands(&query_lower) {
            let cmd_keyword = cmd.keyword.to_lowercase();

            if cmd_keyword == keyword {
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

        // 11. Apps (last since there are many)
        for app in apps {
            // Simple fuzzy match on name and keywords
            let app_matches = app.name.to_lowercase().contains(&query_lower)
                || app
                    .keywords
                    .iter()
                    .any(|k| k.to_lowercase().contains(&query_lower))
                || app
                    .description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&query_lower))
                    .unwrap_or(false);

            if query.is_empty() || app_matches {
                results.push(SearchResult::App {
                    id: app.id.clone(),
                    name: app.name.clone(),
                    exec: app.exec.clone(),
                    icon: app.icon.clone(),
                    description: app.description.clone(),
                });
            }
        }

        results.truncate(max_results);
        results
    }

    /// Search within a specific command mode context
    pub fn search_in_command_mode(
        &self,
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
                    vec![SearchResult::Quicklink {
                        keyword: ext.keyword.clone(),
                        name: format!("Type to search {}", ext.name),
                        url: url.clone(),
                        has_query: true,
                    }]
                } else {
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
                vec![SearchResult::Alias {
                    keyword: ext.keyword.clone(),
                    name: ext.name.clone(),
                    target: target.clone(),
                }]
            }
        }
    }
}
