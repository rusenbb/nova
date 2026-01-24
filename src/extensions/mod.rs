//! Extension system for Nova.
//!
//! This module provides the infrastructure for running TypeScript/JavaScript
//! extensions using an embedded Deno runtime.
//!
//! # Architecture
//!
//! ```text
//! ExtensionHost
//! ├── manifests: HashMap<ExtensionId, ExtensionManifest>
//! ├── isolates: HashMap<ExtensionId, ExtensionIsolate>
//! ├── command_index: HashMap<keyword, (ExtensionId, CommandId)>
//! └── tokio_runtime: Runtime
//! ```
//!
//! Extensions are loaded on-demand when their commands are triggered.
//! Isolates are kept warm for a configurable timeout, then unloaded.

pub mod components;
mod error;
mod host;
pub mod ipc;
mod isolate;
mod manifest;
pub mod storage;

pub use components::Component;
pub use error::{ExtensionError, ExtensionResult};
pub use host::{ExtensionHost, ExtensionHostConfig};
pub use ipc::{nova_extension, NovaContext};
pub use manifest::{
    CommandConfig, ExtensionManifest, ExtensionMeta, PermissionsConfig, PreferenceConfig,
    PreferenceType,
};
pub use storage::ExtensionStorage;

/// Unique identifier for an extension.
pub type ExtensionId = String;

/// Unique identifier for a command within an extension.
pub type CommandId = String;
