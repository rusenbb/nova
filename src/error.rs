//! Error types for Nova
//!
//! Provides standardized error handling across the application.

// Allow dead code - used by FFI layer but not by GTK binary.
#![allow(dead_code)]

use thiserror::Error;

/// Errors that can occur in Nova
#[derive(Debug, Error)]
pub enum NovaError {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Extension loading or execution errors
    #[error("Extension error: {0}")]
    Extension(String),

    /// Script execution errors
    #[error("Script error: {0}")]
    Script(String),

    /// Clipboard operation errors
    #[error("Clipboard error: {0}")]
    Clipboard(String),

    /// File search errors
    #[error("File search error: {0}")]
    FileSearch(String),

    /// IPC communication errors
    #[error("IPC error: {0}")]
    Ipc(String),

    /// Launch errors (failed to start app)
    #[error("Launch error: {0}")]
    Launch(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parsing errors
    #[error("Config parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
}

/// Result type alias for Nova operations
pub type NovaResult<T> = Result<T, NovaError>;
