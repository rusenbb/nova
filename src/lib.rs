//! Nova - Keyboard-driven productivity launcher.
//!
//! Nova provides a fast, keyboard-driven interface for launching apps, running
//! scripts, searching files, and executing custom commands on Linux and macOS.
//!
//! # Architecture
//!
//! The library is organized into these main modules:
//!
//! - [`config`] - Configuration loading and management
//! - [`core`] - Core search engine and result types
//! - [`executor`] - Action execution (launching apps, running scripts)
//! - [`platform`] - Platform abstraction layer (Linux, macOS, Windows)
//! - [`extensions`] - JavaScript/TypeScript extension runtime
//! - [`services`] - Search providers (apps, clipboard, emoji, files, etc.)
//!
//! # FFI Layer
//!
//! Native frontends (Swift, GTK4, WinUI) interact via the C FFI layer in
//! [`ffi`]. This provides a stable ABI for cross-language interop.
//!
//! # Example
//!
//! ```ignore
//! use nova::{Config, Platform, AppEntry};
//!
//! // Load configuration
//! let config = Config::load().expect("Failed to load config");
//!
//! // Get platform-specific operations
//! let platform = nova::platform::current();
//!
//! // Discover installed applications
//! let apps = platform.discover_apps();
//! ```

// Public modules
pub mod cli;
pub mod config;
pub mod core;
pub mod executor;
pub mod extensions;
pub mod platform;
pub mod services;
pub mod theme;

// FFI module - internal implementation details
#[doc(hidden)]
pub mod ffi;

// Internal modules
mod error;
mod search;

// Re-export commonly used types for convenience
pub use config::Config;
pub use core::search::{SearchEngine, SearchResult};
pub use error::{NovaError, NovaResult};
pub use executor::ExecutionAction;
pub use platform::{AppEntry, Platform, SystemCommand};
