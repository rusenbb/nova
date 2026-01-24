//! IPC type definitions for extension communication.

use serde::{Deserialize, Serialize};

use crate::extensions::components::Component;

/// Request to fetch a URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchRequest {
    pub url: String,
    #[serde(default)]
    pub method: FetchMethod,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub body: Option<String>,
}

/// HTTP method for fetch requests.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FetchMethod {
    #[default]
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

/// Response from a fetch request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchResponse {
    pub status: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
}

/// A rendered component from an extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedComponent {
    #[serde(rename = "type")]
    pub component_type: String,
    #[serde(default)]
    pub props: serde_json::Value,
    #[serde(default)]
    pub children: Vec<RenderedComponent>,
}

/// Result of executing an extension command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandExecutionResult {
    /// Whether the command succeeded.
    pub success: bool,
    /// Error message if failed.
    #[serde(default)]
    pub error: Option<String>,
    /// Rendered component tree (strongly typed).
    #[serde(default)]
    pub component: Option<Component>,
    /// Whether the window should close.
    #[serde(default)]
    pub should_close: bool,
}
