//! iced-based UI for Nova launcher.
//!
//! This module provides the cross-platform UI implementation using the iced framework.

#[cfg(feature = "iced-ui")]
pub mod app;
#[cfg(feature = "iced-ui")]
pub mod hotkey;
#[cfg(feature = "iced-ui")]
pub mod settings;
#[cfg(feature = "iced-ui")]
pub mod single_instance;
#[cfg(feature = "iced-ui")]
pub mod style;
#[cfg(feature = "iced-ui")]
pub mod theme;

#[cfg(feature = "iced-ui")]
pub use app::NovaApp;
#[cfg(feature = "iced-ui")]
pub use hotkey::{HotkeyConfig, HotkeyKey, HotkeyMessage, HotkeyModifier};
#[cfg(feature = "iced-ui")]
pub use single_instance::{acquire_or_signal, InstanceMessage, InstanceResult};
