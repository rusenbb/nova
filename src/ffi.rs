//! C FFI layer for Nova core library.
//!
//! This module provides a C-compatible interface for native frontends (Swift, GTK4, etc.)
//! to interact with the Nova search engine and execution system.
//!
//! All complex data types are serialized as JSON strings for cross-language compatibility.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use crate::config::Config;
use crate::core::{SearchEngine, SearchResult};
use crate::executor::{execute, ExecutionAction, ExecutionResult};
use crate::extensions::components::Component;
use crate::extensions::{ExtensionHost, ExtensionHostConfig};
use crate::platform::{self, AppEntry, Platform};
use crate::services::clipboard::ClipboardHistory;
use crate::services::custom_commands::CustomCommandsIndex;
use crate::services::extensions::{get_extensions_dir, ExtensionManager};

/// Opaque handle to the Nova core engine.
///
/// This struct holds all the state needed for search and execution.
/// It is created by `nova_core_new()` and must be freed with `nova_core_free()`.
pub struct NovaCore {
    platform: Box<dyn Platform>,
    config: Config,
    search_engine: SearchEngine,
    apps: Vec<AppEntry>,
    custom_commands: CustomCommandsIndex,
    extension_manager: ExtensionManager,
    clipboard_history: ClipboardHistory,
    /// Deno extension host (lazy-loaded).
    deno_host: Option<ExtensionHost>,
    /// Cached search results for `nova_core_execute()`
    last_results: Vec<SearchResult>,
}

/// JSON response wrapper for search results.
#[derive(serde::Serialize)]
struct SearchResponse {
    results: Vec<SearchResult>,
}

/// JSON response wrapper for execution results.
#[derive(serde::Serialize)]
struct ExecuteResponse {
    #[serde(flatten)]
    result: ExecutionResult,
}

// ============================================================================
// FFI Functions
// ============================================================================

/// Create a new Nova core instance.
///
/// Returns a pointer to the core instance, or null on failure.
/// The caller is responsible for calling `nova_core_free()` to release the memory.
#[no_mangle]
pub extern "C" fn nova_core_new() -> *mut NovaCore {
    let platform = platform::current();
    let config = Config::load();

    // Discover apps
    let apps = platform.discover_apps();

    // Load custom commands
    let custom_commands = CustomCommandsIndex::new(&config);

    // Load extensions
    let extension_manager = ExtensionManager::load(&get_extensions_dir());

    // Initialize search engine
    let search_engine = SearchEngine::new();

    // Initialize clipboard history
    let clipboard_history = ClipboardHistory::new(50);

    // Initialize Deno extension host (if extensions directory exists)
    let deno_host = {
        let deno_extensions_dir = dirs::data_dir()
            .map(|d| d.join("nova").join("deno-extensions"))
            .unwrap_or_else(|| std::path::PathBuf::from("~/.nova/deno-extensions"));

        if deno_extensions_dir.exists() {
            let config = ExtensionHostConfig {
                extensions_dir: deno_extensions_dir,
                ..Default::default()
            };
            match ExtensionHost::new(config) {
                Ok(host) => Some(host),
                Err(e) => {
                    eprintln!("Warning: Failed to initialize Deno extension host: {}", e);
                    None
                }
            }
        } else {
            None
        }
    };

    let core = Box::new(NovaCore {
        platform,
        config,
        search_engine,
        apps,
        custom_commands,
        extension_manager,
        clipboard_history,
        deno_host,
        last_results: Vec::new(),
    });

    Box::into_raw(core)
}

/// Free a Nova core instance.
///
/// # Safety
/// The handle must be a valid pointer returned by `nova_core_new()`.
/// After calling this function, the handle is no longer valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_free(handle: *mut NovaCore) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

/// Perform a search and return JSON results.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle from `nova_core_new()`
/// * `query` - The search query as a C string (UTF-8)
/// * `max_results` - Maximum number of results to return
///
/// # Returns
/// A JSON string containing the search results. The caller must free this
/// string using `nova_string_free()`.
///
/// # Safety
/// The handle must be valid and the query must be a valid UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn nova_core_search(
    handle: *mut NovaCore,
    query: *const c_char,
    max_results: u32,
) -> *mut c_char {
    if handle.is_null() || query.is_null() {
        return ptr::null_mut();
    }

    let core = &mut *handle;

    // Convert C string to Rust string
    let query_str = match CStr::from_ptr(query).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    // Perform search
    let mut results = core.search_engine.search(
        &core.apps,
        &core.custom_commands,
        &core.extension_manager,
        &core.clipboard_history,
        query_str,
        max_results as usize,
    );

    // Add Deno extension commands to results
    if let Some(ref deno_host) = core.deno_host {
        for cmd in deno_host.search_commands(query_str) {
            results.push(SearchResult::DenoCommand {
                extension_id: cmd.extension_id,
                command_id: cmd.command_id,
                title: cmd.title,
                subtitle: cmd.subtitle,
                icon: cmd.icon,
                keywords: cmd.keywords,
            });
        }
    }

    // Limit total results
    results.truncate(max_results as usize);

    // Cache results for execute
    core.last_results = results.clone();

    // Serialize to JSON
    let response = SearchResponse { results };
    let json = match serde_json::to_string(&response) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    // Convert to C string
    match CString::new(json) {
        Ok(s) => s.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Execute a search result by index.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
/// * `index` - Index of the result in the last search results (0-based)
///
/// # Returns
/// A JSON string containing the execution result. The caller must free this
/// string using `nova_string_free()`.
///
/// # Safety
/// The handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_execute(handle: *mut NovaCore, index: u32) -> *mut c_char {
    if handle.is_null() {
        return ptr::null_mut();
    }

    let core = &mut *handle;

    // Get the result at the index
    let result = match core.last_results.get(index as usize) {
        Some(r) => r,
        None => {
            let response = ExecuteResponse {
                result: ExecutionResult::Error("Invalid result index".to_string()),
            };
            let json = serde_json::to_string(&response).unwrap_or_default();
            return CString::new(json).map(|s| s.into_raw()).unwrap_or(ptr::null_mut());
        }
    };

    // Convert SearchResult to ExecutionAction
    let action = result_to_action(result);

    // Execute the action
    let exec_result = execute(&action, core.platform.as_ref(), Some(&core.extension_manager));

    // Serialize result
    let response = ExecuteResponse {
        result: exec_result,
    };
    let json = match serde_json::to_string(&response) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    match CString::new(json) {
        Ok(s) => s.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Poll the clipboard for new content.
///
/// Call this periodically to update the clipboard history.
///
/// # Safety
/// The handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_poll_clipboard(handle: *mut NovaCore) {
    if handle.is_null() {
        return;
    }

    let core = &mut *handle;

    if let Some(content) = core.platform.clipboard_read() {
        core.clipboard_history.poll_with_content(&content);
    }
}

/// Reload configuration and refresh app list.
///
/// # Safety
/// The handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_reload(handle: *mut NovaCore) {
    if handle.is_null() {
        return;
    }

    let core = &mut *handle;

    // Reload config
    core.config = Config::load();

    // Refresh custom commands
    core.custom_commands = CustomCommandsIndex::new(&core.config);

    // Refresh extensions
    core.extension_manager = ExtensionManager::load(&get_extensions_dir());

    // Refresh apps
    core.apps = core.platform.discover_apps();
}

/// Get the number of results from the last search.
///
/// # Safety
/// The handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_result_count(handle: *mut NovaCore) -> u32 {
    if handle.is_null() {
        return 0;
    }

    let core = &*handle;
    core.last_results.len() as u32
}

/// Free a string allocated by the FFI functions.
///
/// # Safety
/// The pointer must be a valid string returned by one of the FFI functions,
/// or null (which is safely ignored).
#[no_mangle]
pub unsafe extern "C" fn nova_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

// ============================================================================
// Internal Helper Functions
// ============================================================================

/// Convert a SearchResult to an ExecutionAction.
fn result_to_action(result: &SearchResult) -> ExecutionAction {
    match result {
        SearchResult::App(app) => ExecutionAction::LaunchApp { app: app.clone() },

        SearchResult::Command { id, .. } => {
            if id == "nova:settings" {
                ExecutionAction::OpenSettings
            } else if id == "nova:quit" {
                ExecutionAction::Quit
            } else if let Some(cmd) = SearchEngine::parse_system_command(id) {
                ExecutionAction::SystemCommand { command: cmd }
            } else {
                ExecutionAction::NeedsInput
            }
        }

        SearchResult::Alias { target, .. } => ExecutionAction::RunShellCommand {
            command: target.clone(),
        },

        SearchResult::Quicklink { url, has_query, .. } => {
            if *has_query {
                ExecutionAction::NeedsInput
            } else {
                ExecutionAction::OpenUrl { url: url.clone() }
            }
        }

        SearchResult::QuicklinkWithQuery { resolved_url, .. } => ExecutionAction::OpenUrl {
            url: resolved_url.clone(),
        },

        SearchResult::Script {
            path,
            has_argument,
            output_mode,
            ..
        } => {
            if *has_argument {
                ExecutionAction::NeedsInput
            } else {
                ExecutionAction::RunScript {
                    path: path.clone(),
                    argument: None,
                    output_mode: output_mode.clone(),
                }
            }
        }

        SearchResult::ScriptWithArgument {
            path,
            argument,
            output_mode,
            ..
        } => ExecutionAction::RunScript {
            path: path.clone(),
            argument: Some(argument.clone()),
            output_mode: output_mode.clone(),
        },

        SearchResult::ExtensionCommand { command } => {
            if command.has_argument {
                ExecutionAction::NeedsInput
            } else {
                ExecutionAction::RunExtensionCommand {
                    command: command.clone(),
                    argument: None,
                }
            }
        }

        SearchResult::ExtensionCommandWithArg { command, argument } => {
            ExecutionAction::RunExtensionCommand {
                command: command.clone(),
                argument: Some(argument.clone()),
            }
        }

        SearchResult::DenoCommand {
            extension_id,
            command_id,
            ..
        } => ExecutionAction::RunDenoCommand {
            extension_id: extension_id.clone(),
            command_id: command_id.clone(),
            argument: None,
        },

        SearchResult::DenoCommandWithArg {
            extension_id,
            command_id,
            argument,
            ..
        } => ExecutionAction::RunDenoCommand {
            extension_id: extension_id.clone(),
            command_id: command_id.clone(),
            argument: Some(argument.clone()),
        },

        SearchResult::Calculation { result, .. } => ExecutionAction::CopyToClipboard {
            content: result.trim_start_matches("= ").to_string(),
            notification: "Calculation result copied".to_string(),
        },

        SearchResult::ClipboardItem { content, .. } => ExecutionAction::CopyToClipboard {
            content: content.clone(),
            notification: "Clipboard item copied".to_string(),
        },

        SearchResult::FileResult { path, .. } => ExecutionAction::OpenFile { path: path.clone() },

        SearchResult::EmojiResult { emoji, .. } => ExecutionAction::CopyToClipboard {
            content: emoji.clone(),
            notification: format!("{} copied", emoji),
        },

        SearchResult::UnitConversion { result, .. } => ExecutionAction::CopyToClipboard {
            content: result.clone(),
            notification: "Conversion result copied".to_string(),
        },
    }
}

// ============================================================================
// Extension Execution FFI Functions
// ============================================================================

/// JSON response for extension command execution.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ExtensionExecuteResponse {
    /// Whether the command succeeded.
    success: bool,
    /// Error message if failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    /// Rendered component tree (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    component: Option<Component>,
    /// Whether the window should close.
    should_close: bool,
}

/// Execute a Deno extension command and return the rendered component.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
/// * `extension_id` - Extension identifier (C string)
/// * `command_id` - Command identifier (C string)
/// * `argument` - Optional argument (C string, can be null)
///
/// # Returns
/// A JSON string containing the execution result with rendered component.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// All pointers must be valid or null (for argument).
#[no_mangle]
pub unsafe extern "C" fn nova_core_execute_extension(
    handle: *mut NovaCore,
    extension_id: *const c_char,
    command_id: *const c_char,
    argument: *const c_char,
) -> *mut c_char {
    if handle.is_null() || extension_id.is_null() || command_id.is_null() {
        let response = ExtensionExecuteResponse {
            success: false,
            error: Some("Invalid handle or extension/command ID".to_string()),
            component: None,
            should_close: false,
        };
        let json = serde_json::to_string(&response).unwrap_or_default();
        return CString::new(json)
            .map(|s| s.into_raw())
            .unwrap_or(ptr::null_mut());
    }

    let core = &mut *handle;

    // Convert C strings to Rust strings
    let ext_id = match CStr::from_ptr(extension_id).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            return error_response("Invalid extension ID encoding");
        }
    };

    let cmd_id = match CStr::from_ptr(command_id).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            return error_response("Invalid command ID encoding");
        }
    };

    let arg = if argument.is_null() {
        None
    } else {
        match CStr::from_ptr(argument).to_str() {
            Ok(s) => Some(s.to_string()),
            Err(_) => {
                return error_response("Invalid argument encoding");
            }
        }
    };

    // Check if we have a Deno host
    let deno_host = match core.deno_host.as_mut() {
        Some(host) => host,
        None => {
            return error_response("Deno extension host not initialized");
        }
    };

    // Execute the command
    match deno_host.execute_command(&ext_id, &cmd_id, arg.as_deref()) {
        Ok(_result_json) => {
            // Parse the result to get component
            // The isolate returns a JSON string with the execution result
            let response = ExtensionExecuteResponse {
                success: true,
                error: None,
                component: None, // TODO: Extract from isolate context
                should_close: false,
            };

            let json = serde_json::to_string(&response).unwrap_or_default();
            CString::new(json)
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut())
        }
        Err(e) => error_response(&format!("Extension execution failed: {}", e)),
    }
}

/// Helper function to create an error response.
fn error_response(message: &str) -> *mut c_char {
    let response = ExtensionExecuteResponse {
        success: false,
        error: Some(message.to_string()),
        component: None,
        should_close: false,
    };
    let json = serde_json::to_string(&response).unwrap_or_default();
    CString::new(json)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// Send an event to an extension callback.
///
/// This is used for interactive components that need to respond to user actions,
/// like search text changes or action triggers.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
/// * `extension_id` - Extension identifier
/// * `callback_id` - The callback ID to invoke
/// * `event_data` - JSON-encoded event data
///
/// # Returns
/// A JSON string containing the updated component tree.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_send_event(
    handle: *mut NovaCore,
    extension_id: *const c_char,
    callback_id: *const c_char,
    event_data: *const c_char,
) -> *mut c_char {
    if handle.is_null() || extension_id.is_null() || callback_id.is_null() {
        return error_response("Invalid handle or parameters");
    }

    let _core = &mut *handle;

    // Convert C strings to Rust strings
    let _ext_id = match CStr::from_ptr(extension_id).to_str() {
        Ok(s) => s,
        Err(_) => {
            return error_response("Invalid extension ID encoding");
        }
    };

    let _cb_id = match CStr::from_ptr(callback_id).to_str() {
        Ok(s) => s,
        Err(_) => {
            return error_response("Invalid callback ID encoding");
        }
    };

    let _event = if event_data.is_null() {
        None
    } else {
        match CStr::from_ptr(event_data).to_str() {
            Ok(s) => Some(s),
            Err(_) => {
                return error_response("Invalid event data encoding");
            }
        }
    };

    // TODO: Implement callback invocation
    // This requires:
    // 1. Looking up the callback in the extension's context
    // 2. Invoking the JavaScript callback function
    // 3. Returning the updated component tree

    error_response("Event handling not yet implemented")
}
