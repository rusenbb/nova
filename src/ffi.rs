//! C FFI layer for Nova core library.
//!
//! This module provides a C-compatible interface for native frontends (Swift, GTK4, etc.)
//! to interact with the Nova search engine and execution system.
//!
//! All complex data types are serialized as JSON strings for cross-language compatibility.

use std::collections::HashMap;
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
    /// Per-extension preferences (including background settings).
    extension_preferences: HashMap<String, HashMap<String, serde_json::Value>>,
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
        let extensions_dir = dirs::data_dir()
            .map(|d| d.join("nova").join("extensions"))
            .unwrap_or_else(|| std::path::PathBuf::from("~/.nova/extensions"));

        println!("[Nova] Deno extensions dir: {:?}", extensions_dir);
        println!(
            "[Nova] Deno extensions dir exists: {}",
            extensions_dir.exists()
        );

        if extensions_dir.exists() {
            let config = ExtensionHostConfig {
                extensions_dir: extensions_dir.clone(),
                ..Default::default()
            };
            match ExtensionHost::new(config) {
                Ok(host) => {
                    println!(
                        "[Nova] Deno host initialized: {} extensions, {} commands",
                        host.extension_count(),
                        host.command_count()
                    );
                    Some(host)
                }
                Err(e) => {
                    println!(
                        "[Nova] Warning: Failed to initialize Deno extension host: {}",
                        e
                    );
                    None
                }
            }
        } else {
            println!("[Nova] Deno extensions dir not found, skipping");
            None
        }
    };

    // Load background preferences
    let bg_prefs = load_background_preferences();
    let mut extension_preferences: HashMap<String, HashMap<String, serde_json::Value>> =
        HashMap::new();

    // Convert background preferences to extension preferences format
    for (ext_id, enabled) in bg_prefs.enabled {
        let mut prefs = HashMap::new();
        prefs.insert(
            "__background_enabled".to_string(),
            serde_json::json!(enabled),
        );
        extension_preferences.insert(ext_id, prefs);
    }

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
        extension_preferences,
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
        Some(r) => r.clone(),
        None => {
            let response = ExecuteResponse {
                result: ExecutionResult::Error("Invalid result index".to_string()),
            };
            let json = serde_json::to_string(&response).unwrap_or_default();
            return CString::new(json)
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut());
        }
    };

    // Handle DenoCommand specially - execute via extension host
    if let SearchResult::DenoCommand {
        extension_id,
        command_id,
        ..
    } = &result
    {
        if let Some(ref mut deno_host) = core.deno_host {
            match deno_host.execute_command(extension_id, command_id, None) {
                Ok(_) => {
                    let response = ExecuteResponse {
                        result: ExecutionResult::SuccessKeepOpen,
                    };
                    let json = serde_json::to_string(&response).unwrap_or_default();
                    return CString::new(json)
                        .map(|s| s.into_raw())
                        .unwrap_or(ptr::null_mut());
                }
                Err(e) => {
                    let response = ExecuteResponse {
                        result: ExecutionResult::Error(format!("Extension error: {}", e)),
                    };
                    let json = serde_json::to_string(&response).unwrap_or_default();
                    return CString::new(json)
                        .map(|s| s.into_raw())
                        .unwrap_or(ptr::null_mut());
                }
            }
        } else {
            let response = ExecuteResponse {
                result: ExecutionResult::Error("Extension host not available".to_string()),
            };
            let json = serde_json::to_string(&response).unwrap_or_default();
            return CString::new(json)
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut());
        }
    }

    // Convert SearchResult to ExecutionAction
    let action = result_to_action(&result);

    // Execute the action
    let exec_result = execute(
        &action,
        core.platform.as_ref(),
        Some(&core.extension_manager),
    );

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

    let core = &mut *handle;

    // Convert C strings to Rust strings
    let ext_id = match CStr::from_ptr(extension_id).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            return error_response("Invalid extension ID encoding");
        }
    };

    let cb_id = match CStr::from_ptr(callback_id).to_str() {
        Ok(s) => s,
        Err(_) => {
            return error_response("Invalid callback ID encoding");
        }
    };

    let event: Option<&str> = if event_data.is_null() {
        None
    } else {
        match CStr::from_ptr(event_data).to_str() {
            Ok(s) => Some(s),
            Err(_) => {
                return error_response("Invalid event data encoding");
            }
        }
    };

    // Get extension host
    let host = match &mut core.deno_host {
        Some(h) => h,
        None => {
            return error_response("Extension host not initialized");
        }
    };

    // Dispatch the event
    match host.dispatch_event(&ext_id, cb_id, event) {
        Ok(result_json) => {
            // Parse the result to extract component if present
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result_json) {
                // Try to parse component if present
                let component: Option<Component> = parsed
                    .get("component")
                    .and_then(|c| serde_json::from_value(c.clone()).ok());

                let response = ExtensionExecuteResponse {
                    success: parsed
                        .get("success")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    component,
                    error: parsed
                        .get("error")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    should_close: parsed
                        .get("should_close")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                };

                let json = serde_json::to_string(&response).unwrap_or_default();
                CString::new(json)
                    .map(|s| s.into_raw())
                    .unwrap_or(ptr::null_mut())
            } else {
                // Return raw result if parsing fails
                CString::new(result_json)
                    .map(|s| s.into_raw())
                    .unwrap_or(ptr::null_mut())
            }
        }
        Err(e) => error_response(&format!("Event dispatch failed: {}", e)),
    }
}

// ============================================================================
// Background Scheduler FFI Functions
// ============================================================================

/// JSON response for background operations.
#[derive(serde::Serialize)]
struct BackgroundResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    power_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
}

/// Enable or disable background execution for an extension.
///
/// This is a user preference that persists across sessions.
/// Even if an extension has background enabled in its manifest,
/// the user can disable it.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
/// * `extension_id` - Extension identifier (C string)
/// * `enabled` - Whether to enable (1) or disable (0) background
///
/// # Returns
/// A JSON string with the result. The caller must free this
/// string using `nova_string_free()`.
///
/// # Safety
/// The handle and extension_id must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn nova_core_set_background_enabled(
    handle: *mut NovaCore,
    extension_id: *const c_char,
    enabled: i32,
) -> *mut c_char {
    if handle.is_null() || extension_id.is_null() {
        let response = BackgroundResponse {
            success: false,
            error: Some("Invalid handle or extension ID".to_string()),
            power_state: None,
            enabled: None,
        };
        let json = serde_json::to_string(&response).unwrap_or_default();
        return CString::new(json)
            .map(|s| s.into_raw())
            .unwrap_or(ptr::null_mut());
    }

    let core = &mut *handle;

    // Convert extension ID
    let ext_id = match CStr::from_ptr(extension_id).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            let response = BackgroundResponse {
                success: false,
                error: Some("Invalid extension ID encoding".to_string()),
                power_state: None,
                enabled: None,
            };
            let json = serde_json::to_string(&response).unwrap_or_default();
            return CString::new(json)
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut());
        }
    };

    let enabled_bool = enabled != 0;

    // For now, we store the setting in the extension host's preferences
    // In a full implementation, this would use the BackgroundScheduler
    // but that requires async runtime integration

    // Store the preference
    if !core.extension_preferences.contains_key(&ext_id) {
        core.extension_preferences
            .insert(ext_id.clone(), std::collections::HashMap::new());
    }

    if let Some(prefs) = core.extension_preferences.get_mut(&ext_id) {
        prefs.insert(
            "__background_enabled".to_string(),
            serde_json::json!(enabled_bool),
        );
    }

    // Persist to disk
    let _ = save_background_preference(&ext_id, enabled_bool);

    let response = BackgroundResponse {
        success: true,
        error: None,
        power_state: None,
        enabled: Some(enabled_bool),
    };

    let json = serde_json::to_string(&response).unwrap_or_default();
    CString::new(json)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// Check if background execution is enabled for an extension.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
/// * `extension_id` - Extension identifier (C string)
///
/// # Returns
/// A JSON string with the result including the enabled status.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// The handle and extension_id must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn nova_core_is_background_enabled(
    handle: *mut NovaCore,
    extension_id: *const c_char,
) -> *mut c_char {
    if handle.is_null() || extension_id.is_null() {
        let response = BackgroundResponse {
            success: false,
            error: Some("Invalid handle or extension ID".to_string()),
            power_state: None,
            enabled: None,
        };
        let json = serde_json::to_string(&response).unwrap_or_default();
        return CString::new(json)
            .map(|s| s.into_raw())
            .unwrap_or(ptr::null_mut());
    }

    let core = &*handle;

    // Convert extension ID
    let ext_id = match CStr::from_ptr(extension_id).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            let response = BackgroundResponse {
                success: false,
                error: Some("Invalid extension ID encoding".to_string()),
                power_state: None,
                enabled: None,
            };
            let json = serde_json::to_string(&response).unwrap_or_default();
            return CString::new(json)
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut());
        }
    };

    // Check manifest first - if no background config, it's not available
    let has_background = core
        .deno_host
        .as_ref()
        .and_then(|h| h.get_manifest(&ext_id))
        .map(|m| m.background.is_some() && m.permissions.background)
        .unwrap_or(false);

    if !has_background {
        let response = BackgroundResponse {
            success: true,
            error: None,
            power_state: None,
            enabled: Some(false),
        };
        let json = serde_json::to_string(&response).unwrap_or_default();
        return CString::new(json)
            .map(|s| s.into_raw())
            .unwrap_or(ptr::null_mut());
    }

    // Check user preference (default to true if manifest has background)
    let enabled = core
        .extension_preferences
        .get(&ext_id)
        .and_then(|prefs| prefs.get("__background_enabled"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true); // Default to enabled if manifest has background

    let response = BackgroundResponse {
        success: true,
        error: None,
        power_state: None,
        enabled: Some(enabled),
    };

    let json = serde_json::to_string(&response).unwrap_or_default();
    CString::new(json)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// Get the current power state (AC, Battery, or Unknown).
///
/// # Returns
/// A JSON string with the power state.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// No safety requirements.
#[no_mangle]
pub extern "C" fn nova_core_get_power_state() -> *mut c_char {
    use crate::extensions::background::detect_power_state;
    use crate::extensions::PowerState;

    let power_state = detect_power_state();
    let state_str = match power_state {
        PowerState::AcPower => "ac",
        PowerState::Battery => "battery",
        PowerState::Unknown => "unknown",
    };

    let response = BackgroundResponse {
        success: true,
        error: None,
        power_state: Some(state_str.to_string()),
        enabled: None,
    };

    let json = serde_json::to_string(&response).unwrap_or_default();
    CString::new(json)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// List all extensions with background execution configured.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
///
/// # Returns
/// A JSON string with an array of extension IDs that have background configured.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// The handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_list_background_extensions(
    handle: *mut NovaCore,
) -> *mut c_char {
    if handle.is_null() {
        return ptr::null_mut();
    }

    let core = &*handle;

    let extensions: Vec<serde_json::Value> = if let Some(ref host) = core.deno_host {
        host.extensions_with_background()
            .into_iter()
            .map(|(id, config)| {
                // Check user preference
                let user_enabled = core
                    .extension_preferences
                    .get(&id)
                    .and_then(|prefs| prefs.get("__background_enabled"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                serde_json::json!({
                    "extensionId": id,
                    "interval": config.interval,
                    "runOnLoad": config.run_on_load,
                    "userEnabled": user_enabled
                })
            })
            .collect()
    } else {
        Vec::new()
    };

    let json = serde_json::to_string(&extensions).unwrap_or_else(|_| "[]".to_string());
    CString::new(json)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

// ============================================================================
// Background Settings Persistence
// ============================================================================

/// Preferences for extension background execution.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct ExtensionBackgroundPrefs {
    enabled: std::collections::HashMap<String, bool>,
}

/// Load background preferences from disk.
fn load_background_preferences() -> ExtensionBackgroundPrefs {
    let prefs_path = dirs::data_dir()
        .map(|d| d.join("nova").join("background").join("preferences.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("~/.nova/background/preferences.json"));

    if !prefs_path.exists() {
        return ExtensionBackgroundPrefs::default();
    }

    match std::fs::read_to_string(&prefs_path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => ExtensionBackgroundPrefs::default(),
    }
}

/// Save a single background preference.
fn save_background_preference(extension_id: &str, enabled: bool) -> Result<(), std::io::Error> {
    let prefs_dir = dirs::data_dir()
        .map(|d| d.join("nova").join("background"))
        .unwrap_or_else(|| std::path::PathBuf::from("~/.nova/background"));

    std::fs::create_dir_all(&prefs_dir)?;

    let prefs_path = prefs_dir.join("preferences.json");

    // Load existing preferences
    let mut prefs = load_background_preferences();

    // Update
    prefs.enabled.insert(extension_id.to_string(), enabled);

    // Save
    let content = serde_json::to_string_pretty(&prefs)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    std::fs::write(&prefs_path, content)
}
