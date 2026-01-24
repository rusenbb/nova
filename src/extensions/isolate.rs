//! Deno isolate wrapper for extension execution.
//!
//! Each extension runs in its own V8 isolate for memory isolation
//! and crash containment.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use deno_core::{JsRuntime, RuntimeOptions};

use super::error::{ExtensionError, ExtensionResult};
use super::ipc::{nova_extension, NovaContext};
use super::manifest::ExtensionManifest;
use super::{CommandId, ExtensionId};

/// State of an extension isolate.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Error variant will be used in Phase 2
pub enum IsolateState {
    /// Not yet loaded into memory.
    Unloaded,
    /// Currently loading (initializing runtime).
    Loading,
    /// Ready to execute commands.
    Ready,
    /// Currently executing a command.
    Executing { command: CommandId },
    /// Error state (requires reload).
    Error { message: String },
}

/// Wrapper around a Deno JsRuntime for a single extension.
#[allow(dead_code)] // Some fields used in Phase 2
pub struct ExtensionIsolate {
    /// Extension identifier.
    pub id: ExtensionId,

    /// Parsed manifest (used for permission checks in Phase 2).
    pub manifest: ExtensionManifest,

    /// Path to extension directory.
    pub extension_dir: PathBuf,

    /// Current state.
    pub state: IsolateState,

    /// Last time this isolate was used.
    pub last_active: Instant,

    /// The V8 runtime (None when unloaded).
    runtime: Option<JsRuntime>,
}

impl ExtensionIsolate {
    /// Create a new isolate for an extension (starts unloaded).
    pub fn new(id: ExtensionId, manifest: ExtensionManifest, extension_dir: PathBuf) -> Self {
        Self {
            id,
            manifest,
            extension_dir,
            state: IsolateState::Unloaded,
            last_active: Instant::now(),
            runtime: None,
        }
    }

    /// Load the extension into memory with the given context.
    ///
    /// The context provides access to platform APIs, storage, and permissions.
    pub fn load(&mut self, ctx: NovaContext) -> ExtensionResult<()> {
        if self.state == IsolateState::Ready {
            return Ok(());
        }

        self.state = IsolateState::Loading;

        // Create runtime with Nova extension ops
        let options = RuntimeOptions {
            extensions: vec![nova_extension::init_ops_and_esm()],
            ..Default::default()
        };

        // Create the runtime
        let mut runtime = JsRuntime::new(options);

        // Add NovaContext to OpState so ops can access it
        runtime.op_state().borrow_mut().put(ctx);

        // Load the extension's entry point
        let entry_path = self.extension_dir.join("dist").join("index.js");
        if !entry_path.exists() {
            // Fall back to src/index.js for development
            let dev_entry = self.extension_dir.join("src").join("index.js");
            if dev_entry.exists() {
                self.load_module(&mut runtime, &dev_entry)?;
            } else {
                return Err(ExtensionError::LoadFailed {
                    extension: self.id.clone(),
                    message: format!(
                        "Entry point not found at {:?} or {:?}",
                        entry_path, dev_entry
                    ),
                });
            }
        } else {
            self.load_module(&mut runtime, &entry_path)?;
        }

        self.runtime = Some(runtime);
        self.state = IsolateState::Ready;
        self.last_active = Instant::now();

        Ok(())
    }

    /// Load a JavaScript module into the runtime.
    fn load_module(&mut self, runtime: &mut JsRuntime, path: &PathBuf) -> ExtensionResult<()> {
        let code = std::fs::read_to_string(path).map_err(|e| ExtensionError::LoadFailed {
            extension: self.id.clone(),
            message: format!("Failed to read {}: {}", path.display(), e),
        })?;

        // Execute the module code
        runtime
            .execute_script("<extension>", code)
            .map_err(|e| ExtensionError::LoadFailed {
                extension: self.id.clone(),
                message: format!("JavaScript error: {}", e),
            })?;

        Ok(())
    }

    /// Unload the extension from memory.
    pub fn unload(&mut self) {
        self.runtime = None;
        self.state = IsolateState::Unloaded;
    }

    /// Check if this isolate has been idle longer than the given duration.
    pub fn is_idle(&self, timeout: Duration) -> bool {
        self.last_active.elapsed() > timeout
    }

    /// Execute a command in this isolate.
    ///
    /// The isolate must be loaded before calling this method. Use `load()` first.
    ///
    /// Returns a JSON string with the command result (component tree or error).
    pub fn execute_command(
        &mut self,
        command: &str,
        argument: Option<&str>,
    ) -> ExtensionResult<String> {
        // Check state
        if self.state == IsolateState::Unloaded {
            return Err(ExtensionError::ExecutionError(
                "Isolate not loaded. Call load() first.".to_string(),
            ));
        }

        if let IsolateState::Error { message } = &self.state {
            return Err(ExtensionError::ExecutionError(message.clone()));
        }

        let runtime = self.runtime.as_mut().ok_or_else(|| {
            ExtensionError::ExecutionError("Runtime not initialized".to_string())
        })?;

        self.state = IsolateState::Executing {
            command: command.to_string(),
        };
        self.last_active = Instant::now();

        // Build the command invocation script
        let arg_json = match argument {
            Some(arg) => serde_json::to_string(arg).unwrap_or_else(|_| "null".to_string()),
            None => "null".to_string(),
        };

        let invoke_script = format!(
            r#"
            (function() {{
                const command = "{}";
                const argument = {};

                // Look for the command handler
                if (typeof globalThis.__nova_commands === 'undefined') {{
                    return JSON.stringify({{ error: "No commands registered" }});
                }}

                const handler = globalThis.__nova_commands[command];
                if (!handler) {{
                    return JSON.stringify({{ error: "Command not found: " + command }});
                }}

                try {{
                    const result = handler(argument);
                    return JSON.stringify({{ result: result }});
                }} catch (e) {{
                    return JSON.stringify({{ error: e.message || String(e) }});
                }}
            }})()
            "#,
            command, arg_json
        );

        let result = runtime
            .execute_script("<invoke>", invoke_script)
            .map_err(|e| ExtensionError::ExecutionError(format!("Execution failed: {}", e)))?;

        // Convert result to string
        let scope = &mut runtime.handle_scope();
        let local = deno_core::v8::Local::new(scope, result);
        let result_str = local
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_else(|| "{}".to_string());

        self.state = IsolateState::Ready;
        self.last_active = Instant::now();

        Ok(result_str)
    }

    /// Get the runtime for direct operations (used by host for async ops in Phase 2).
    #[allow(dead_code)]
    pub fn runtime_mut(&mut self) -> Option<&mut JsRuntime> {
        self.runtime.as_mut()
    }
}

// Nova extension ops are defined in the ipc module (ops.rs).
// The NovaContext is added to OpState during load() and accessed by ops.

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manifest() -> ExtensionManifest {
        ExtensionManifest {
            extension: super::super::manifest::ExtensionMeta {
                name: "test".to_string(),
                title: "Test".to_string(),
                description: String::new(),
                version: "1.0.0".to_string(),
                author: None,
                repo: None,
                homepage: None,
                license: None,
                icon: None,
                keywords: vec![],
                nova_version: None,
            },
            permissions: Default::default(),
            background: None,
            commands: vec![],
            preferences: vec![],
        }
    }

    #[test]
    fn test_isolate_lifecycle() {
        let manifest = create_test_manifest();
        let mut isolate =
            ExtensionIsolate::new("test".to_string(), manifest, PathBuf::from("/tmp/test"));

        assert_eq!(isolate.state, IsolateState::Unloaded);
        assert!(isolate.runtime.is_none());

        isolate.unload();
        assert_eq!(isolate.state, IsolateState::Unloaded);
    }

    #[test]
    fn test_idle_check() {
        let manifest = create_test_manifest();
        let isolate =
            ExtensionIsolate::new("test".to_string(), manifest, PathBuf::from("/tmp/test"));

        // Should not be idle immediately
        assert!(!isolate.is_idle(Duration::from_secs(30)));

        // Would be idle after 0 seconds
        assert!(isolate.is_idle(Duration::from_secs(0)));
    }
}
