//! Nova - Keyboard-driven productivity launcher.
//!
//! This library contains all the core functionality for Nova,
//! which can be used by both the GTK and iced UI binaries.

pub mod config;
pub mod core;
pub mod executor;
pub mod platform;
pub mod services;

#[cfg(feature = "iced-ui")]
pub mod ui;

// Re-export commonly used types
pub use config::Config;
pub use platform::{AppEntry, Platform, SystemCommand};
