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
//!
//! BackgroundScheduler
//! ├── extensions: HashMap<ExtensionId, BackgroundTaskState>
//! ├── power_state: PowerState (AC/Battery)
//! └── callback: BackgroundCallback
//! ```
//!
//! Extensions are loaded on-demand when their commands are triggered.
//! Isolates are kept warm for a configurable timeout, then unloaded.
//!
//! Background tasks are scheduled by the BackgroundScheduler, which:
//! - Runs periodic tasks at configurable intervals
//! - Throttles execution when on battery power
//! - Respects user toggles for per-extension background execution

pub mod background;
pub mod components;
mod error;
mod host;
pub mod ipc;
mod isolate;
mod manifest;
pub mod permissions;
pub mod storage;

pub use background::{
    BackgroundCallback, BackgroundScheduler, BackgroundSchedulerConfig, BackgroundSchedulerHandle,
    PowerState,
};
pub use components::Component;
pub use error::{ExtensionError, ExtensionResult};
pub use host::{ExtensionHost, ExtensionHostConfig};
pub use ipc::{nova_extension, NovaContext};
pub use manifest::{
    BackgroundConfig, CommandConfig, ExtensionManifest, ExtensionMeta, PermissionsConfig,
    PreferenceConfig, PreferenceType,
};
pub use permissions::{PermissionError, PermissionSet, PermissionStore};
pub use storage::ExtensionStorage;

/// Unique identifier for an extension.
pub type ExtensionId = String;

/// Unique identifier for a command within an extension.
pub type CommandId = String;
