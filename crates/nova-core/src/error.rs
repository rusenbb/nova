use thiserror::Error;

/// Errors that can occur in Nova
#[derive(Debug, Error)]
pub enum NovaError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Extension error: {0}")]
    Extension(String),

    #[error("Script error: {0}")]
    Script(String),

    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("File search error: {0}")]
    FileSearch(String),

    #[error("IPC error: {0}")]
    Ipc(String),

    #[error("Launch error: {0}")]
    Launch(String),

    #[error("Platform error: {0}")]
    Platform(String),
}

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
