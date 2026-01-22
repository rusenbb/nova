//! iced-based UI for Nova launcher.
//!
//! This module provides the cross-platform UI implementation using the iced framework.

#[cfg(feature = "iced-ui")]
pub mod app;
#[cfg(feature = "iced-ui")]
pub mod style;
#[cfg(feature = "iced-ui")]
pub mod theme;

#[cfg(feature = "iced-ui")]
pub use app::NovaApp;
