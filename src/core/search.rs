//! Search engine for Nova - platform-agnostic search logic.
//!
//! This module contains the SearchResult enum and all search-related logic
//! that doesn't depend on any specific platform or UI framework.

use std::path::PathBuf;

use crate::platform::{AppEntry, SystemCommand};
use crate::services::clipboard::ClipboardHistory;
use crate::services::custom_commands::{CustomCommandsIndex, ScriptOutputMode};
use crate::services::extension::{Extension, ExtensionKind};
use crate::services::extensions::{ExtensionManager, LoadedCommand};
use crate::services::{calculator, emoji, file_search, units};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// Search results that appear in the launcher.
#[derive(Debug, Clone)]
pub enum SearchResult {
    /// An installed application
    App(AppEntry),

    /// A built-in Nova command (settings, quit, system commands)
    Command {
        id: String,
        name: String,
        description: String,
    },

    /// An alias (keyword -> shell command)
    Alias {
        keyword: String,
        name: String,
        target: String,
    },

    /// A quicklink (keyword -> URL, optionally with query)
    Quicklink {
        keyword: String,
        name: String,
        url: String,
        has_query: bool,
    },

    /// A quicklink with a query filled in
    QuicklinkWithQuery {
        keyword: String,
        name: String,
        url: String,
        query: String,
        resolved_url: String,
    },

    /// A script that can be executed
    Script {
        id: String,
        name: String,
        description: String,
        path: PathBuf,
        has_argument: bool,
        output_mode: ScriptOutputMode,
    },

    /// A script with an argument provided
    ScriptWithArgument {
        id: String,
        name: String,
        description: String,
        path: PathBuf,
        argument: String,
        output_mode: ScriptOutputMode,
    },

    /// An extension command
    ExtensionCommand { command: LoadedCommand },

    /// An extension command with an argument
    ExtensionCommandWithArg {
        command: LoadedCommand,
        argument: String,
    },

    /// A calculator result
    Calculation { expression: String, result: String },

    /// A clipboard history item
    ClipboardItem {
        index: usize,
        content: String,
        preview: String,
        time_ago: String,
    },

    /// A file search result
    FileResult {
        name: String,
        path: String,
        is_dir: bool,
    },

    /// An emoji picker result
    EmojiResult {
        emoji: String,
        name: String,
        aliases: String,
    },

    /// A unit conversion result
    UnitConversion { display: String, result: String },
}

impl SearchResult {
    /// Get the display name for this result.
    pub fn name(&self) -> &str {
        match self {
            SearchResult::App(app) => &app.name,
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

    /// Get the description for this result.
    pub fn description(&self) -> Option<&str> {
        match self {
            SearchResult::App(app) => app.description.as_deref(),
            SearchResult::Command { description, .. } => Some(description),
            SearchResult::Alias { target, .. } => Some(target),
            SearchResult::Quicklink { url, .. } => Some(url),
            SearchResult::QuicklinkWithQuery { resolved_url, .. } => Some(resolved_url),
            SearchResult::Script { description, .. } => {
                if description.is_empty() {
                    None
                } else {
                    Some(description)
                }
            }
            SearchResult::ScriptWithArgument { description, .. } => {
                if description.is_empty() {
                    None
                } else {
                    Some(description)
                }
            }
            SearchResult::ExtensionCommand { command } => {
                if command.description.is_empty() {
                    None
                } else {
                    Some(&command.description)
                }
            }
            SearchResult::ExtensionCommandWithArg { command, .. } => {
                if command.description.is_empty() {
                    None
                } else {
                    Some(&command.description)
                }
            }
            SearchResult::Calculation { expression, .. } => Some(expression),
            SearchResult::ClipboardItem { time_ago, .. } => Some(time_ago),
            SearchResult::FileResult { path, .. } => Some(path),
            SearchResult::EmojiResult { aliases, .. } => Some(aliases),
            SearchResult::UnitConversion { display, .. } => Some(display),
        }
    }

    /// Check if this result needs additional input before execution.
    pub fn needs_input(&self) -> bool {
        match self {
            SearchResult::Quicklink { has_query, .. } => *has_query,
            SearchResult::Script { has_argument, .. } => *has_argument,
            SearchResult::ExtensionCommand { command } => command.has_argument,
            _ => false,
        }
    }
}

/// The search engine that powers Nova's search functionality.
pub struct SearchEngine {
    matcher: SkimMatcherV2,
}

impl SearchEngine {
    /// Create a new search engine instance.
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Get built-in system commands.
    pub fn get_system_commands() -> Vec<SearchResult> {
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

    /// Parse a command ID into a SystemCommand.
    pub fn parse_system_command(id: &str) -> Option<SystemCommand> {
        match id {
            "system:lock" => Some(SystemCommand::Lock),
            "system:sleep" => Some(SystemCommand::Sleep),
            "system:logout" => Some(SystemCommand::Logout),
            "system:restart" => Some(SystemCommand::Restart),
            "system:shutdown" => Some(SystemCommand::Shutdown),
            _ => None,
        }
    }

    /// Search across all sources with the given query.
    pub fn search(
        &self,
        apps: &[AppEntry],
        custom_commands: &CustomCommandsIndex,
        extension_manager: &ExtensionManager,
        clipboard_history: &ClipboardHistory,
        query: &str,
        max_results: usize,
    ) -> Vec<SearchResult> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        // Split query into keyword and remaining text
        let query_parts: Vec<&str> = query.splitn(2, ' ').collect();
        let keyword = query_parts[0].to_lowercase();
        let remaining_query = query_parts.get(1).map(|s| s.to_string());

        // 1. Check for alias matches (exact keyword match or partial match in keyword/name)
        for alias in &custom_commands.aliases {
            let alias_keyword_lower = alias.keyword.to_lowercase();
            if alias_keyword_lower == keyword
                || alias_keyword_lower.contains(&query_lower)
                || alias.name.to_lowercase().contains(&query_lower)
            {
                results.push(SearchResult::Alias {
                    keyword: alias.keyword.clone(),
                    name: alias.name.clone(),
                    target: alias.target.clone(),
                });
            }
        }

        // 2. Calculator - try to evaluate as math expression
        if let Some(calc_result) = calculator::evaluate(query) {
            let formatted = calculator::format_result(calc_result);
            results.push(SearchResult::Calculation {
                expression: query.to_string(),
                result: format!("= {}", formatted),
            });
        }

        // 3. Unit converter - try to parse as unit conversion
        if query.contains(" to ") {
            if let Some(conversion) = units::convert(query) {
                results.push(SearchResult::UnitConversion {
                    display: conversion.display(),
                    result: conversion.result(),
                });
            }
        }

        // 4. Clipboard history - trigger on keywords
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

        // 5. File search - trigger on ~ or / prefix
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

        // 6. Emoji picker - trigger on : prefix
        if query.starts_with(':') && query.len() > 1 {
            let emoji_query = &query[1..];
            for emoji_entry in emoji::search(emoji_query, 10) {
                results.push(SearchResult::EmojiResult {
                    emoji: emoji_entry.char.to_string(),
                    name: format!("{} {}", emoji_entry.char, emoji_entry.name()),
                    aliases: emoji_entry.aliases(),
                });
            }
        }

        // 7. Check for quicklink matches
        for quicklink in &custom_commands.quicklinks {
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

        // 8. Search scripts
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

        // 9. System commands
        for cmd in Self::get_system_commands() {
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
        for cmd in extension_manager.search_commands(&query_lower) {
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

        // 11. App results (last since there are many)
        let app_results = self.search_apps(apps, query);
        for app in app_results {
            results.push(SearchResult::App(app.clone()));
        }

        // Limit total results
        results.truncate(max_results);
        results
    }

    /// Search within a specific command mode context.
    pub fn search_in_command_mode(
        &self,
        extension: &Extension,
        query: &str,
        _max_results: usize,
    ) -> Vec<SearchResult> {
        match &extension.kind {
            ExtensionKind::Quicklink { url, .. } => {
                if query.is_empty() {
                    vec![SearchResult::Quicklink {
                        keyword: extension.keyword.clone(),
                        name: format!("Type to search {}", extension.name),
                        url: url.clone(),
                        has_query: true,
                    }]
                } else {
                    let resolved = url.replace("{query}", &urlencoding::encode(query));
                    vec![SearchResult::QuicklinkWithQuery {
                        keyword: extension.keyword.clone(),
                        name: format!("{}: {}", extension.name, query),
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
                        id: extension.keyword.clone(),
                        name: format!("{} (type argument)", extension.name),
                        description: description.clone(),
                        path: path.clone(),
                        has_argument: true,
                        output_mode: output_mode.clone(),
                    }]
                } else {
                    vec![SearchResult::ScriptWithArgument {
                        id: extension.keyword.clone(),
                        name: format!("{}: {}", extension.name, query),
                        description: description.clone(),
                        path: path.clone(),
                        argument: query.to_string(),
                        output_mode: output_mode.clone(),
                    }]
                }
            }
            ExtensionKind::Alias { target } => {
                vec![SearchResult::Alias {
                    keyword: extension.keyword.clone(),
                    name: extension.name.clone(),
                    target: target.clone(),
                }]
            }
        }
    }

    /// Search apps using fuzzy matching.
    fn search_apps<'a>(&self, apps: &'a [AppEntry], query: &str) -> Vec<&'a AppEntry> {
        if query.is_empty() {
            return apps.iter().take(8).collect();
        }

        let query_lower = query.to_lowercase();
        let mut scored: Vec<(&AppEntry, i64)> = apps
            .iter()
            .filter_map(|entry| {
                // Match against name
                let name_score = self
                    .matcher
                    .fuzzy_match(&entry.name.to_lowercase(), &query_lower);

                // Match against keywords
                let keyword_score = entry
                    .keywords
                    .iter()
                    .filter_map(|kw| self.matcher.fuzzy_match(&kw.to_lowercase(), &query_lower))
                    .max();

                // Match against description
                let desc_score = entry
                    .description
                    .as_ref()
                    .and_then(|d| self.matcher.fuzzy_match(&d.to_lowercase(), &query_lower))
                    .map(|s| s / 2);

                // Get best score
                let best_score = [name_score, keyword_score, desc_score]
                    .into_iter()
                    .flatten()
                    .max()?;

                // Boost exact prefix matches
                let prefix_boost = if entry.name.to_lowercase().starts_with(&query_lower) {
                    100
                } else {
                    0
                };

                Some((entry, best_score + prefix_boost))
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().take(8).map(|(entry, _)| entry).collect()
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}
