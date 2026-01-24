//! Error types for the extension system.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur in the extension system.
#[derive(Debug, Error)]
pub enum ExtensionError {
    #[error("Extension directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("Manifest not found in extension: {0}")]
    ManifestNotFound(PathBuf),

    #[error("Invalid manifest in {path}: {message}")]
    ManifestInvalid { path: PathBuf, message: String },

    #[error("Extension '{0}' not found")]
    ExtensionNotFound(String),

    #[error("Command '{command}' not found in extension '{extension}'")]
    CommandNotFound { extension: String, command: String },

    #[error("Failed to load extension '{extension}': {message}")]
    LoadFailed { extension: String, message: String },

    #[error("Extension execution timed out after {0} seconds")]
    ExecutionTimeout(u64),

    #[error("Extension execution failed: {0}")]
    ExecutionError(String),

    #[error("Permission denied: {permission}")]
    PermissionDenied { permission: String },

    #[error("Network access to '{domain}' not allowed")]
    NetworkDomainNotAllowed { domain: String },

    #[error("Storage limit exceeded for extension '{0}'")]
    StorageLimitExceeded(String),

    #[error("Too many isolates running (max: {0})")]
    TooManyIsolates(usize),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("JavaScript error: {0}")]
    JavaScript(String),
}

/// Result type for extension operations.
pub type ExtensionResult<T> = Result<T, ExtensionError>;
