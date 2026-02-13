pub mod config;
pub mod error;
pub mod executor;
pub mod search;
pub mod services;
pub mod theme;

pub use config::Config;
pub use error::{NovaError, NovaResult};
pub use executor::{ExecutionAction, SystemCommand};
pub use search::{CommandModeState, PlatformAppEntry, SearchEngine, SearchResult};
pub use theme::{ThemePalette, available_themes, get_theme_colors, get_theme_palette, parse_hex_color};
