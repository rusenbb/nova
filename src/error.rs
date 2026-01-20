//! Error types for Nova
//!
//! Provides standardized error handling across the application.

use std::fmt;

/// Errors that can occur in Nova
#[derive(Debug)]
pub enum NovaError {
    /// Configuration-related errors
    Config(String),

    /// Extension loading or execution errors
    Extension(String),

    /// Script execution errors
    Script(String),

    /// Clipboard operation errors
    Clipboard(String),

    /// File search errors
    FileSearch(String),

    /// IPC communication errors
    Ipc(String),

    /// Launch errors (failed to start app)
    Launch(String),
}

impl fmt::Display for NovaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NovaError::Config(msg) => write!(f, "Configuration error: {}", msg),
            NovaError::Extension(msg) => write!(f, "Extension error: {}", msg),
            NovaError::Script(msg) => write!(f, "Script error: {}", msg),
            NovaError::Clipboard(msg) => write!(f, "Clipboard error: {}", msg),
            NovaError::FileSearch(msg) => write!(f, "File search error: {}", msg),
            NovaError::Ipc(msg) => write!(f, "IPC error: {}", msg),
            NovaError::Launch(msg) => write!(f, "Launch error: {}", msg),
        }
    }
}

impl std::error::Error for NovaError {}

/// Result type alias for Nova operations
pub type NovaResult<T> = Result<T, NovaError>;

impl From<std::io::Error> for NovaError {
    fn from(err: std::io::Error) -> Self {
        NovaError::Ipc(err.to_string())
    }
}

impl From<toml::de::Error> for NovaError {
    fn from(err: toml::de::Error) -> Self {
        NovaError::Config(err.to_string())
    }
}
