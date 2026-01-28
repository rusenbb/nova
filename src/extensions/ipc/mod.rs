//! IPC module for extension-to-core communication.
//!
//! This module provides the bridge between JavaScript extensions running in Deno
//! isolates and the Rust Nova core. It uses deno_core's op2 system to expose
//! Nova APIs to extensions.
//!
//! # Helper Macros
//!
//! The [`macros`] module provides convenience macros for implementing IPC operations:
//!
//! - [`nova_ctx!`] - Get immutable context reference
//! - [`nova_ctx_mut!`] - Get mutable context reference
//! - [`nova_with_permission!`] - Get context with permission check
//! - [`nova_with_permission_mut!`] - Get mutable context with permission check
//!
//! # Example
//!
//! ```ignore
//! use nova::nova_with_permission;
//!
//! #[op2(fast)]
//! fn op_my_feature(state: &mut OpState, #[string] arg: String) -> Result<(), AnyError> {
//!     let ctx = nova_with_permission!(state, "system");
//!     ctx.platform.do_something(&arg)?;
//!     Ok(())
//! }
//! ```

mod context;
#[macro_use]
pub mod macros;
mod ops;
mod types;

pub use context::NovaContext;
pub use macros::*;
pub use ops::nova_extension;
pub use types::{
    CommandExecutionResult, FetchMethod, FetchRequest, FetchResponse, RenderedComponent,
};
