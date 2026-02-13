use std::collections::HashMap;
use std::path::PathBuf;

use super::custom_commands::{CustomCommandsIndex, ScriptOutputMode};
use crate::config::{AliasConfig, QuicklinkConfig};

/// Unified abstraction for keyword-triggered commands (Aliases, Quicklinks, Scripts)
#[derive(Debug, Clone)]
pub struct Extension {
    pub keyword: String,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub kind: ExtensionKind,
}

#[derive(Debug, Clone)]
pub enum ExtensionKind {
    Alias {
        target: String,
    },
    Quicklink {
        url: String,
        has_query: bool,
    },
    Script {
        path: PathBuf,
        has_argument: bool,
        output_mode: ScriptOutputMode,
        description: String,
    },
}

impl Extension {
    /// Check if this extension accepts a query/argument
    pub fn accepts_query(&self) -> bool {
        match &self.kind {
            ExtensionKind::Alias { .. } => false,
            ExtensionKind::Quicklink { has_query, .. } => *has_query,
            ExtensionKind::Script { has_argument, .. } => *has_argument,
        }
    }

    /// Get display name for command mode pill
    pub fn pill_text(&self) -> &str {
        &self.name
    }

    /// Get pill color (default to accent color if not specified)
    pub fn pill_color(&self) -> &str {
        self.color.as_deref().unwrap_or("#cba6f7")
    }
}

/// Index of all extensions for fast keyword lookup
pub struct ExtensionIndex {
    extensions: Vec<Extension>,
    by_keyword: HashMap<String, Extension>,
}

impl ExtensionIndex {
    /// Build an ExtensionIndex from CustomCommandsIndex
    pub fn from_custom_commands(
        index: &CustomCommandsIndex,
        aliases: &[AliasConfig],
        quicklinks: &[QuicklinkConfig],
    ) -> Self {
        let mut extensions = Vec::new();
        let mut by_keyword = HashMap::new();

        for alias in aliases {
            let ext = Extension {
                keyword: alias.keyword.clone(),
                name: alias.name.clone(),
                icon: alias.icon.clone(),
                color: None,
                kind: ExtensionKind::Alias {
                    target: alias.target.clone(),
                },
            };
            by_keyword.insert(alias.keyword.to_lowercase(), ext.clone());
            extensions.push(ext);
        }

        for ql in quicklinks {
            let ext = Extension {
                keyword: ql.keyword.clone(),
                name: ql.name.clone(),
                icon: ql.icon.clone(),
                color: None,
                kind: ExtensionKind::Quicklink {
                    url: ql.url.clone(),
                    has_query: ql.has_query_placeholder(),
                },
            };
            by_keyword.insert(ql.keyword.to_lowercase(), ext.clone());
            extensions.push(ext);
        }

        for script in &index.scripts {
            let ext = Extension {
                keyword: script.id.clone(),
                name: script.name.clone(),
                icon: script.icon.clone(),
                color: None,
                kind: ExtensionKind::Script {
                    path: script.path.clone(),
                    has_argument: script.has_argument,
                    output_mode: script.output_mode.clone(),
                    description: script.description.clone(),
                },
            };
            by_keyword.insert(script.id.to_lowercase(), ext.clone());
            extensions.push(ext);
        }

        Self {
            extensions,
            by_keyword,
        }
    }

    /// Get an extension by exact keyword match (case-insensitive)
    pub fn get_by_keyword(&self, keyword: &str) -> Option<&Extension> {
        self.by_keyword.get(&keyword.to_lowercase())
    }

    /// Search extensions by partial keyword or name match
    pub fn search(&self, query: &str) -> Vec<&Extension> {
        let query_lower = query.to_lowercase();
        self.extensions
            .iter()
            .filter(|ext| {
                ext.keyword.to_lowercase().contains(&query_lower)
                    || ext.name.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
}
