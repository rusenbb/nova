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

// Re-exports used by FFI layer and native frontends.
// Some may appear unused when compiling with gtk-ui feature since
// GTK main.rs defines its own local types.
#[allow(unused_imports)]
pub use crate::services::calculator;
#[allow(unused_imports)]
pub use crate::services::clipboard::{ClipboardEntry, ClipboardHistory};
#[allow(unused_imports)]
pub use crate::services::custom_commands::{CustomCommandsIndex, ScriptEntry, ScriptOutputMode};
#[allow(unused_imports)]
pub use crate::services::emoji::{self, Emoji};
#[allow(unused_imports)]
pub use crate::services::extension::{Extension, ExtensionIndex, ExtensionKind};
#[allow(unused_imports)]
pub use crate::services::extensions::{
    get_extensions_dir, ExtensionManager, LoadedCommand, OutputMode,
};
#[allow(unused_imports)]
pub use crate::services::file_search::{self, FileEntry};
#[allow(unused_imports)]
pub use crate::services::format;
#[allow(unused_imports)]
pub use crate::services::units::{self, Conversion};
#[allow(unused_imports)]
pub use search::{SearchEngine, SearchResult};
