//! Core engine module - platform-agnostic business logic.
//!
//! This module contains all the core functionality that doesn't depend on
//! any specific platform or UI framework:
//! - Search result types and search logic
//! - Calculator (math expression evaluation)
//! - Emoji picker
//! - Unit converter
//! - File search
//! - Custom commands (aliases, quicklinks, scripts)
//! - Extensions system
//! - Clipboard history management

pub mod search;

// Re-export search types (used by FFI layer)
#[allow(unused_imports)]
pub use search::{SearchEngine, SearchResult};

// Re-export from services (these are already platform-agnostic)
#[allow(unused_imports)]
pub use crate::services::calculator;
pub use crate::services::clipboard::{ClipboardEntry, ClipboardHistory};
pub use crate::services::custom_commands::{CustomCommandsIndex, ScriptEntry, ScriptOutputMode};
pub use crate::services::emoji::{self, Emoji};
pub use crate::services::extension::{Extension, ExtensionIndex, ExtensionKind};
pub use crate::services::extensions::{
    get_extensions_dir, ExtensionManager, LoadedCommand, OutputMode,
};
pub use crate::services::file_search::{self, FileEntry};
pub use crate::services::format;
pub use crate::services::units::{self, Conversion};
