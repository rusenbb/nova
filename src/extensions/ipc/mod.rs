//! IPC module for extension-to-core communication.
//!
//! This module provides the bridge between JavaScript extensions running in Deno
//! isolates and the Rust Nova core. It uses deno_core's op2 system to expose
//! Nova APIs to extensions.

mod context;
mod ops;
mod types;

pub use context::NovaContext;
pub use ops::nova_extension;
pub use types::{CommandExecutionResult, FetchMethod, FetchRequest, FetchResponse, RenderedComponent};
