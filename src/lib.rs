//! Nova - Keyboard-driven productivity launcher.
//!
//! This library provides the core search engine and execution system for Nova.
//! Native frontends (Swift, GTK4, WinUI) interact via the C FFI layer in `ffi.rs`.

pub mod cli;
pub mod config;
pub mod core;
pub mod executor;
pub mod extensions;
pub mod ffi;
pub mod platform;
pub mod services;
pub mod theme;

// Re-export commonly used types
pub use config::Config;
pub use platform::{AppEntry, Platform, SystemCommand};
