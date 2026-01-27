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
use crate::services::frecency::FrecencyData;

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
    /// Frecency data for ranking search results by usage.
    frecency: FrecencyData,
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

    // Load frecency data for smart ranking
    let frecency = FrecencyData::load();
    println!("[Nova] Frecency loaded: {} entries", frecency.len());

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
        frecency,
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
        let mut core = Box::from_raw(handle);
        // Flush frecency data to disk before dropping
        core.frecency.flush();
        drop(core);
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

    // Perform search with frecency-based ranking
    let mut results = core.search_engine.search(
        &core.apps,
        &core.custom_commands,
        &core.extension_manager,
        &core.clipboard_history,
        Some(&core.frecency),
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
        // Log frecency for Deno extension command
        let frecency_id = format!("{}:{}", extension_id, command_id);
        core.frecency.log_usage(
            &frecency_id,
            crate::services::frecency::ResultKind::Extension,
        );

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

    // Log frecency usage for this result
    if let (Some(id), Some(kind)) = (result.frecency_id(), result.frecency_kind()) {
        core.frecency.log_usage(id, kind);
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
            } else if let Some(position) = SearchEngine::parse_window_command(id) {
                ExecutionAction::SetWindowPosition { position }
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
        Ok(result_json) => {
            // Parse the result to get component
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result_json) {
                // Try to parse component if present
                let component: Option<Component> = parsed
                    .get("component")
                    .and_then(|c| serde_json::from_value(c.clone()).ok());

                let response = ExtensionExecuteResponse {
                    success: parsed
                        .get("success")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true),
                    error: parsed
                        .get("error")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    component,
                    should_close: parsed
                        .get("shouldClose")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                };

                let json = serde_json::to_string(&response).unwrap_or_default();
                CString::new(json)
                    .map(|s| s.into_raw())
                    .unwrap_or(ptr::null_mut())
            } else {
                // Fallback if parsing fails
                let response = ExtensionExecuteResponse {
                    success: true,
                    error: None,
                    component: None,
                    should_close: false,
                };

                let json = serde_json::to_string(&response).unwrap_or_default();
                CString::new(json)
                    .map(|s| s.into_raw())
                    .unwrap_or(ptr::null_mut())
            }
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
// Permission Management FFI Functions
// ============================================================================

use crate::extensions::permissions::{permission_description, permission_icon, PermissionStore};

/// JSON response for permission queries.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PermissionQueryResponse {
    /// Permissions that need user consent.
    needs_consent: Vec<PermissionInfo>,
}

/// Information about a single permission.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PermissionInfo {
    /// Permission name (e.g., "clipboard", "network").
    name: String,
    /// Human-readable description.
    description: String,
    /// Icon name (SF Symbols).
    icon: String,
    /// Additional details (e.g., allowed domains for network).
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

/// JSON response for extension permissions list.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ExtensionPermissionsResponse {
    /// List of extensions with their permissions.
    extensions: Vec<ExtensionPermissionEntry>,
}

/// Permission entry for an extension.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ExtensionPermissionEntry {
    /// Extension ID.
    extension_id: String,
    /// Extension title.
    #[serde(skip_serializing_if = "Option::is_none")]
    extension_title: Option<String>,
    /// Granted permissions.
    permissions: Vec<String>,
    /// When permissions were last updated.
    updated_at: u64,
}

/// Check if an extension needs permission consent before execution.
///
/// This should be called before executing an extension command.
/// If the response contains permissions that need consent, show a dialog
/// and call `nova_core_grant_permission` for each approved permission.
///
/// Get the title of an extension by ID.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
/// * `extension_id` - Extension identifier
///
/// # Returns
/// The extension title as a C string, or null if not found.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// The handle and extension_id must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_get_extension_title(
    handle: *mut NovaCore,
    extension_id: *const c_char,
) -> *mut c_char {
    if handle.is_null() || extension_id.is_null() {
        return std::ptr::null_mut();
    }

    let core = &*handle;

    let ext_id = match CStr::from_ptr(extension_id).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let deno_host = match &core.deno_host {
        Some(host) => host,
        None => return std::ptr::null_mut(),
    };

    match deno_host.get_extension_title(&ext_id.to_string()) {
        Some(title) => {
            CString::new(title)
                .map(|s| s.into_raw())
                .unwrap_or(std::ptr::null_mut())
        }
        None => std::ptr::null_mut(),
    }
}

/// # Arguments
/// * `handle` - A valid NovaCore handle
/// * `extension_id` - Extension identifier
///
/// # Returns
/// A JSON string containing permissions that need consent.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// The handle and extension_id must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_check_permissions(
    handle: *mut NovaCore,
    extension_id: *const c_char,
) -> *mut c_char {
    if handle.is_null() || extension_id.is_null() {
        return permission_error_response("Invalid handle or extension ID");
    }

    let core = &*handle;

    // Convert C string to Rust string
    let ext_id = match CStr::from_ptr(extension_id).to_str() {
        Ok(s) => s,
        Err(_) => return permission_error_response("Invalid extension ID encoding"),
    };

    // Get the extension manifest to see what permissions it requests
    let deno_host = match &core.deno_host {
        Some(host) => host,
        None => return permission_error_response("Extension host not initialized"),
    };

    let manifest = match deno_host.get_manifest(&ext_id.to_string()) {
        Some(m) => m,
        None => return permission_error_response(&format!("Extension '{}' not found", ext_id)),
    };

    // Load permission store
    let store = PermissionStore::new();

    // Convert manifest permissions to PermissionSet
    let requested =
        crate::extensions::permissions::PermissionSet::from_manifest(&manifest.permissions);

    // Check which permissions need consent
    let needs = store.needs_consent(ext_id, &requested);

    // Build response
    let mut needs_consent = Vec::new();

    for perm_name in needs {
        let mut details = None;

        // Add details for certain permissions
        if perm_name == "network" && !manifest.permissions.network.is_empty() {
            details = Some(format!(
                "Domains: {}",
                manifest.permissions.network.join(", ")
            ));
        }

        needs_consent.push(PermissionInfo {
            name: perm_name.clone(),
            description: permission_description(&perm_name).to_string(),
            icon: permission_icon(&perm_name).to_string(),
            details,
        });
    }

    let response = PermissionQueryResponse { needs_consent };
    let json = serde_json::to_string(&response).unwrap_or_default();

    CString::new(json)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// Grant a permission to an extension.
///
/// Call this after the user approves a permission in the consent dialog.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
/// * `extension_id` - Extension identifier
/// * `permission` - Permission name to grant (e.g., "clipboard", "network")
///
/// # Returns
/// 1 on success, 0 on failure.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_grant_permission(
    handle: *mut NovaCore,
    extension_id: *const c_char,
    permission: *const c_char,
) -> i32 {
    if handle.is_null() || extension_id.is_null() || permission.is_null() {
        return 0;
    }

    let core = &*handle;

    // Convert C strings to Rust strings
    let ext_id = match CStr::from_ptr(extension_id).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let perm_name = match CStr::from_ptr(permission).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    // Get the extension manifest
    let deno_host = match &core.deno_host {
        Some(host) => host,
        None => return 0,
    };

    let manifest = match deno_host.get_manifest(&ext_id.to_string()) {
        Some(m) => m,
        None => return 0,
    };

    // Load existing grants
    let mut store = PermissionStore::new();
    let mut current = store.get_permissions(ext_id);

    // Grant the specific permission
    match perm_name {
        "clipboard" => current.clipboard = true,
        "network" => {
            current.network.enabled = true;
            current.network.allowed_domains = manifest.permissions.network.clone();
        }
        "filesystem" => current.filesystem.enabled = true,
        "system" | "notifications" => current.system = true,
        "storage" => current.storage = true,
        "background" => current.background = true,
        _ => return 0,
    }

    // Save
    store.grant(ext_id, current, Some(manifest.extension.version.clone()));

    match store.save() {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

/// Grant all requested permissions to an extension at once.
///
/// This is a convenience function for when the user approves all permissions.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
/// * `extension_id` - Extension identifier
///
/// # Returns
/// 1 on success, 0 on failure.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_grant_all_permissions(
    handle: *mut NovaCore,
    extension_id: *const c_char,
) -> i32 {
    if handle.is_null() || extension_id.is_null() {
        return 0;
    }

    let core = &*handle;

    // Convert C string to Rust string
    let ext_id = match CStr::from_ptr(extension_id).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    // Get the extension manifest
    let deno_host = match &core.deno_host {
        Some(host) => host,
        None => return 0,
    };

    let manifest = match deno_host.get_manifest(&ext_id.to_string()) {
        Some(m) => m,
        None => return 0,
    };

    // Convert manifest permissions to PermissionSet
    let permissions =
        crate::extensions::permissions::PermissionSet::from_manifest(&manifest.permissions);

    // Save all permissions
    let mut store = PermissionStore::new();
    store.grant(
        ext_id,
        permissions,
        Some(manifest.extension.version.clone()),
    );

    match store.save() {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

/// Revoke a permission from an extension.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
/// * `extension_id` - Extension identifier
/// * `permission` - Permission name to revoke (e.g., "clipboard", "network")
///
/// # Returns
/// 1 on success, 0 on failure.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_revoke_permission(
    _handle: *mut NovaCore,
    extension_id: *const c_char,
    permission: *const c_char,
) -> i32 {
    if extension_id.is_null() || permission.is_null() {
        return 0;
    }

    // Convert C strings to Rust strings
    let ext_id = match CStr::from_ptr(extension_id).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let perm_name = match CStr::from_ptr(permission).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let mut store = PermissionStore::new();
    store.revoke_permission(ext_id, perm_name);

    match store.save() {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

/// Revoke all permissions from an extension.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
/// * `extension_id` - Extension identifier
///
/// # Returns
/// 1 on success, 0 on failure.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_revoke_all_permissions(
    _handle: *mut NovaCore,
    extension_id: *const c_char,
) -> i32 {
    if extension_id.is_null() {
        return 0;
    }

    // Convert C string to Rust string
    let ext_id = match CStr::from_ptr(extension_id).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let mut store = PermissionStore::new();
    store.revoke(ext_id);

    match store.save() {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

/// Get all extensions with their granted permissions.
///
/// Use this for the permissions management settings page.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
///
/// # Returns
/// A JSON string containing all extensions and their permissions.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// The handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_list_permissions(handle: *mut NovaCore) -> *mut c_char {
    if handle.is_null() {
        return permission_error_response("Invalid handle");
    }

    let core = &*handle;

    let store = PermissionStore::new();
    let deno_host = &core.deno_host;

    let mut extensions = Vec::new();

    for (ext_id, grants) in store.all_grants() {
        // Get extension title if available
        let title = deno_host
            .as_ref()
            .and_then(|h| h.get_manifest(ext_id))
            .map(|m| m.extension.title.clone());

        // Get list of granted permissions
        let permissions = grants
            .permissions
            .enabled_permissions()
            .into_iter()
            .map(String::from)
            .collect();

        extensions.push(ExtensionPermissionEntry {
            extension_id: ext_id.clone(),
            extension_title: title,
            permissions,
            updated_at: grants.updated_at,
        });
    }

    let response = ExtensionPermissionsResponse { extensions };
    let json = serde_json::to_string(&response).unwrap_or_default();

    CString::new(json)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// Helper function to create a permission error response.
fn permission_error_response(message: &str) -> *mut c_char {
    let response = serde_json::json!({
        "error": message,
        "needsConsent": []
    });
    let json = serde_json::to_string(&response).unwrap_or_default();
    CString::new(json)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
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

// ============================================================================
// Extension Browser FFI Functions
// ============================================================================

use crate::services::browser::{BrowserClient, BrowserTab, ExtensionBrowserData};

/// Get extension browser data for the "Browse Extensions" view.
///
/// This fetches popular extensions from the registry and combines them
/// with installed extension status.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
/// * `tab` - Tab name: "discover", "installed", or "updates"
/// * `search_query` - Optional search query (NULL for empty)
///
/// # Returns
/// A JSON string containing ExtensionBrowserData.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// The handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_get_browser_data(
    handle: *mut NovaCore,
    tab: *const c_char,
    search_query: *const c_char,
) -> *mut c_char {
    if handle.is_null() {
        return browser_error_response("Invalid handle");
    }

    let _core = &*handle;

    // Parse tab
    let tab_str = if tab.is_null() {
        "discover"
    } else {
        CStr::from_ptr(tab).to_str().unwrap_or("discover")
    };

    let browser_tab = match tab_str {
        "installed" => BrowserTab::Installed,
        "updates" => BrowserTab::Updates,
        _ => BrowserTab::Discover,
    };

    // Parse search query
    let query = if search_query.is_null() {
        String::new()
    } else {
        CStr::from_ptr(search_query)
            .to_str()
            .unwrap_or("")
            .to_string()
    };

    // Get installed extensions
    let installed = get_installed_extensions_list();

    // Create async runtime and fetch from registry
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return browser_error_response(&format!("Failed to create runtime: {}", e)),
    };

    let client = BrowserClient::new();

    // Fetch based on tab and query
    let registry_result = rt.block_on(async {
        if !query.is_empty() {
            client.search(&query, 20).await
        } else {
            client.get_popular(20).await
        }
    });

    let registry_extensions = match registry_result {
        Ok(exts) => exts,
        Err(e) => {
            // Return partial data with error
            let data = ExtensionBrowserData {
                extensions: Vec::new(),
                search_query: query,
                loading: false,
                tab: browser_tab,
                error: Some(format!("Registry error: {}", e)),
            };
            let json = serde_json::to_string(&data).unwrap_or_default();
            return CString::new(json)
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut());
        }
    };

    // Check for updates if on updates tab
    let updates = if browser_tab == BrowserTab::Updates {
        rt.block_on(async { client.check_updates(&installed).await })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    // Build browser data
    let data = crate::services::browser::build_browser_data(
        &registry_extensions,
        &installed,
        &updates,
        browser_tab,
        &query,
    );

    let json = serde_json::to_string(&data).unwrap_or_default();
    CString::new(json)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// Install an extension from the registry.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
/// * `publisher` - Publisher name
/// * `name` - Extension name
/// * `version` - Optional version (NULL for latest)
///
/// # Returns
/// A JSON string with the result.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_install_extension(
    handle: *mut NovaCore,
    publisher: *const c_char,
    name: *const c_char,
    version: *const c_char,
) -> *mut c_char {
    if handle.is_null() || publisher.is_null() || name.is_null() {
        return browser_error_response("Invalid handle or parameters");
    }

    let pub_str = match CStr::from_ptr(publisher).to_str() {
        Ok(s) => s,
        Err(_) => return browser_error_response("Invalid publisher encoding"),
    };

    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return browser_error_response("Invalid name encoding"),
    };

    let ver_str = if version.is_null() {
        None
    } else {
        CStr::from_ptr(version).to_str().ok()
    };

    // Create async runtime
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return browser_error_response(&format!("Failed to create runtime: {}", e)),
    };

    // Download from registry
    let download_result =
        rt.block_on(async { crate::cli::registry::download(pub_str, name_str, ver_str).await });

    let data = match download_result {
        Ok(d) => d,
        Err(e) => return browser_error_response(&format!("Download failed: {}", e)),
    };

    // Install from tarball
    let full_name = format!("{}/{}", pub_str, name_str);
    match crate::cli::install::install_from_tarball(&data, &full_name) {
        Ok(()) => {
            // Reload extensions in the core
            let core = &mut *handle;
            core.extension_manager = ExtensionManager::load(&get_extensions_dir());

            // Re-initialize Deno host to pick up new extension
            if let Some(ref mut host) = core.deno_host {
                let _ = host.scan_extensions();
            }

            let response = serde_json::json!({
                "success": true,
                "message": format!("Installed {}", full_name)
            });
            let json = serde_json::to_string(&response).unwrap_or_default();
            CString::new(json)
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut())
        }
        Err(e) => browser_error_response(&format!("Install failed: {}", e)),
    }
}

/// Uninstall an extension.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
/// * `name` - Extension name (publisher/name format)
///
/// # Returns
/// A JSON string with the result.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_uninstall_extension(
    handle: *mut NovaCore,
    name: *const c_char,
) -> *mut c_char {
    if handle.is_null() || name.is_null() {
        return browser_error_response("Invalid handle or name");
    }

    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return browser_error_response("Invalid name encoding"),
    };

    // Get extension directory
    let extensions_dir = get_extensions_dir();

    // Find the extension directory (could be just the name part after /)
    let ext_name = name_str.split('/').next_back().unwrap_or(name_str);
    let ext_dir = extensions_dir.join(ext_name);

    if !ext_dir.exists() {
        return browser_error_response(&format!("Extension '{}' not found", name_str));
    }

    // Remove the directory
    match std::fs::remove_dir_all(&ext_dir) {
        Ok(()) => {
            // Reload extensions in the core
            let core = &mut *handle;
            core.extension_manager = ExtensionManager::load(&get_extensions_dir());

            let response = serde_json::json!({
                "success": true,
                "message": format!("Uninstalled {}", name_str)
            });
            let json = serde_json::to_string(&response).unwrap_or_default();
            CString::new(json)
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut())
        }
        Err(e) => browser_error_response(&format!("Uninstall failed: {}", e)),
    }
}

/// Check for extension updates.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
///
/// # Returns
/// A JSON string with available updates.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// The handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_check_extension_updates(handle: *mut NovaCore) -> *mut c_char {
    if handle.is_null() {
        return browser_error_response("Invalid handle");
    }

    let installed = get_installed_extensions_list();

    if installed.is_empty() {
        let response = serde_json::json!({
            "updates": [],
            "count": 0
        });
        let json = serde_json::to_string(&response).unwrap_or_default();
        return CString::new(json)
            .map(|s| s.into_raw())
            .unwrap_or(ptr::null_mut());
    }

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return browser_error_response(&format!("Failed to create runtime: {}", e)),
    };

    let client = BrowserClient::new();
    let updates_result = rt.block_on(async { client.check_updates(&installed).await });

    match updates_result {
        Ok(updates) => {
            let response = serde_json::json!({
                "updates": updates,
                "count": updates.len()
            });
            let json = serde_json::to_string(&response).unwrap_or_default();
            CString::new(json)
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut())
        }
        Err(e) => browser_error_response(&format!("Update check failed: {}", e)),
    }
}

/// Helper function to get list of installed extensions.
fn get_installed_extensions_list() -> Vec<(String, String)> {
    let extensions_dir = get_extensions_dir();
    if !extensions_dir.exists() {
        return Vec::new();
    }

    let mut installed = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&extensions_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("nova.toml");
            if !manifest_path.exists() {
                continue;
            }

            // Parse manifest to get name and version
            if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                if let Ok(manifest) = toml::from_str::<toml::Value>(&content) {
                    let default_name = entry.file_name().to_str().unwrap_or("unknown").to_string();

                    let name = manifest
                        .get("extension")
                        .and_then(|e| e.get("name"))
                        .and_then(|n| n.as_str())
                        .map(String::from)
                        .unwrap_or(default_name);

                    let version = manifest
                        .get("extension")
                        .and_then(|e| e.get("version"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("0.0.0");

                    let author = manifest
                        .get("extension")
                        .and_then(|e| e.get("author"))
                        .and_then(|a| a.as_str())
                        .unwrap_or("local");

                    installed.push((format!("{}/{}", author, name), version.to_string()));
                }
            }
        }
    }

    installed
}

/// Helper function to create a browser error response.
fn browser_error_response(message: &str) -> *mut c_char {
    let response = serde_json::json!({
        "success": false,
        "error": message,
        "extensions": [],
        "loading": false,
        "tab": "discover"
    });
    let json = serde_json::to_string(&response).unwrap_or_default();
    CString::new(json)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

// ============================================================================
// Window Management FFI Functions
// ============================================================================

use crate::platform::{ScreenInfo, WindowFrame, WindowInfo, WindowPosition};

/// JSON response for window operations.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct WindowResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    window: Option<WindowInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    windows: Option<Vec<WindowInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    screens: Option<Vec<ScreenInfo>>,
}

/// Check if window management is supported.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
///
/// # Returns
/// 1 if supported, 0 if not.
///
/// # Safety
/// The handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_window_management_supported(handle: *mut NovaCore) -> i32 {
    if handle.is_null() {
        return 0;
    }

    let core = &*handle;
    if core.platform.window_management_supported() {
        1
    } else {
        0
    }
}

/// Get the currently focused window.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
///
/// # Returns
/// A JSON string containing WindowInfo.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// The handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_get_focused_window(handle: *mut NovaCore) -> *mut c_char {
    if handle.is_null() {
        return window_error_response("Invalid handle");
    }

    let core = &*handle;

    match core.platform.get_focused_window() {
        Ok(window) => {
            let response = WindowResponse {
                success: true,
                error: None,
                window: Some(window),
                windows: None,
                screens: None,
            };
            let json = serde_json::to_string(&response).unwrap_or_default();
            CString::new(json)
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut())
        }
        Err(e) => window_error_response(&e),
    }
}

/// List all visible windows.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
///
/// # Returns
/// A JSON string containing array of WindowInfo.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// The handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_list_windows(handle: *mut NovaCore) -> *mut c_char {
    if handle.is_null() {
        return window_error_response("Invalid handle");
    }

    let core = &*handle;

    match core.platform.list_windows() {
        Ok(windows) => {
            let response = WindowResponse {
                success: true,
                error: None,
                window: None,
                windows: Some(windows),
                screens: None,
            };
            let json = serde_json::to_string(&response).unwrap_or_default();
            CString::new(json)
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut())
        }
        Err(e) => window_error_response(&e),
    }
}

/// List all screens/displays.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
///
/// # Returns
/// A JSON string containing array of ScreenInfo.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// The handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_list_screens(handle: *mut NovaCore) -> *mut c_char {
    if handle.is_null() {
        return window_error_response("Invalid handle");
    }

    let core = &*handle;

    match core.platform.list_screens() {
        Ok(screens) => {
            let response = WindowResponse {
                success: true,
                error: None,
                window: None,
                windows: None,
                screens: Some(screens),
            };
            let json = serde_json::to_string(&response).unwrap_or_default();
            CString::new(json)
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut())
        }
        Err(e) => window_error_response(&e),
    }
}

/// Move and resize a window.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
/// * `window_id` - Window identifier
/// * `x` - New X position
/// * `y` - New Y position
/// * `width` - New width
/// * `height` - New height
///
/// # Returns
/// A JSON string with the result.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// The handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_set_window_frame(
    handle: *mut NovaCore,
    window_id: u64,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> *mut c_char {
    if handle.is_null() {
        return window_error_response("Invalid handle");
    }

    let core = &*handle;

    let frame = WindowFrame {
        x,
        y,
        width,
        height,
    };

    match core.platform.set_window_frame(window_id, frame) {
        Ok(()) => {
            let response = serde_json::json!({
                "success": true
            });
            let json = serde_json::to_string(&response).unwrap_or_default();
            CString::new(json)
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut())
        }
        Err(e) => window_error_response(&e),
    }
}

/// Apply a preset window position.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
/// * `position` - Position name (e.g., "left-half", "maximize", "center")
///
/// # Returns
/// A JSON string with the result.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// The handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_set_window_position(
    handle: *mut NovaCore,
    position: *const c_char,
) -> *mut c_char {
    if handle.is_null() || position.is_null() {
        return window_error_response("Invalid handle or position");
    }

    let core = &*handle;

    let pos_str = match CStr::from_ptr(position).to_str() {
        Ok(s) => s,
        Err(_) => return window_error_response("Invalid position encoding"),
    };

    let window_pos: WindowPosition = match pos_str.parse() {
        Ok(p) => p,
        Err(e) => return window_error_response(&e),
    };

    // Get focused window first
    let window = match core.platform.get_focused_window() {
        Ok(w) => w,
        Err(e) => return window_error_response(&format!("Failed to get focused window: {}", e)),
    };

    match core.platform.set_window_position(window.id, window_pos) {
        Ok(()) => {
            let response = serde_json::json!({
                "success": true,
                "position": pos_str
            });
            let json = serde_json::to_string(&response).unwrap_or_default();
            CString::new(json)
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut())
        }
        Err(e) => window_error_response(&e),
    }
}

/// Helper function to create a window error response.
fn window_error_response(message: &str) -> *mut c_char {
    let response = WindowResponse {
        success: false,
        error: Some(message.to_string()),
        window: None,
        windows: None,
        screens: None,
    };
    let json = serde_json::to_string(&response).unwrap_or_default();
    CString::new(json)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

// ============================================================================
// Frecency Management FFI Functions
// ============================================================================

use crate::services::frecency::FrecencyStats;

/// JSON response for frecency operations.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct FrecencyResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stats: Option<FrecencyStatsJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_items: Option<Vec<FrecencyItem>>,
}

/// JSON-friendly version of FrecencyStats.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct FrecencyStatsJson {
    total_entries: usize,
    total_usage_count: u64,
    max_usage_count: u32,
    entries_by_kind: HashMap<String, usize>,
}

/// A single frecency item with score.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct FrecencyItem {
    id: String,
    score: f64,
}

impl From<FrecencyStats> for FrecencyStatsJson {
    fn from(stats: FrecencyStats) -> Self {
        Self {
            total_entries: stats.total_entries,
            total_usage_count: stats.total_usage_count,
            max_usage_count: stats.max_usage_count,
            entries_by_kind: stats
                .entries_by_kind
                .into_iter()
                .map(|(k, v)| (format!("{:?}", k).to_lowercase(), v))
                .collect(),
        }
    }
}

/// Get frecency usage statistics.
///
/// Returns statistics about tracked items including total entries,
/// total usage count, and breakdown by result kind.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
///
/// # Returns
/// A JSON string containing FrecencyStats.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// The handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_get_frecency_stats(handle: *mut NovaCore) -> *mut c_char {
    if handle.is_null() {
        return frecency_error_response("Invalid handle");
    }

    let core = &*handle;
    let stats = core.frecency.stats();

    let response = FrecencyResponse {
        success: true,
        error: None,
        stats: Some(stats.into()),
        top_items: None,
    };

    let json = serde_json::to_string(&response).unwrap_or_default();
    CString::new(json)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// Get the top N items by frecency score.
///
/// Returns items sorted by their frecency score (highest first).
/// Useful for showing "most used" items in settings.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
/// * `limit` - Maximum number of items to return
///
/// # Returns
/// A JSON string containing array of items with scores.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// The handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_get_top_frecency(
    handle: *mut NovaCore,
    limit: u32,
) -> *mut c_char {
    if handle.is_null() {
        return frecency_error_response("Invalid handle");
    }

    let core = &*handle;
    let top = core.frecency.top_by_score(limit as usize);

    let items: Vec<FrecencyItem> = top
        .into_iter()
        .map(|(id, score)| FrecencyItem { id, score })
        .collect();

    let response = FrecencyResponse {
        success: true,
        error: None,
        stats: None,
        top_items: Some(items),
    };

    let json = serde_json::to_string(&response).unwrap_or_default();
    CString::new(json)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// Clear all frecency data.
///
/// Resets the usage history. This cannot be undone.
/// Useful for privacy or when the user wants to start fresh.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
///
/// # Returns
/// A JSON string with the result.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// The handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_clear_frecency(handle: *mut NovaCore) -> *mut c_char {
    if handle.is_null() {
        return frecency_error_response("Invalid handle");
    }

    let core = &mut *handle;
    core.frecency.clear();

    let response = serde_json::json!({
        "success": true,
        "message": "Frecency data cleared"
    });

    let json = serde_json::to_string(&response).unwrap_or_default();
    CString::new(json)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// Boost an item's frecency score.
///
/// Increases the frequency count for an item, effectively making it
/// appear higher in search results. Use this when the user "pins"
/// or favorites an item.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
/// * `id` - Item identifier (C string)
/// * `multiplier` - Boost multiplier (e.g., 2.0 doubles the score)
///
/// # Returns
/// A JSON string with the result.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// The handle and id must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_boost_frecency(
    handle: *mut NovaCore,
    id: *const c_char,
    multiplier: f64,
) -> *mut c_char {
    if handle.is_null() || id.is_null() {
        return frecency_error_response("Invalid handle or id");
    }

    let core = &mut *handle;

    let id_str = match CStr::from_ptr(id).to_str() {
        Ok(s) => s,
        Err(_) => return frecency_error_response("Invalid id encoding"),
    };

    // Check if item exists
    if core.frecency.get_entry(id_str).is_none() {
        return frecency_error_response(&format!("Item '{}' not found in frecency data", id_str));
    }

    core.frecency.boost(id_str, multiplier);
    core.frecency.flush(); // Persist immediately

    let new_score = core.frecency.calculate(id_str);

    let response = serde_json::json!({
        "success": true,
        "id": id_str,
        "newScore": new_score
    });

    let json = serde_json::to_string(&response).unwrap_or_default();
    CString::new(json)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// Penalize an item's frecency score.
///
/// Decreases the frequency count for an item, effectively making it
/// appear lower in search results. Use this when the user wants to
/// "hide" or deprioritize an item.
///
/// # Arguments
/// * `handle` - A valid NovaCore handle
/// * `id` - Item identifier (C string)
/// * `divisor` - Penalty divisor (e.g., 2.0 halves the score)
///
/// # Returns
/// A JSON string with the result.
/// The caller must free this string using `nova_string_free()`.
///
/// # Safety
/// The handle and id must be valid.
#[no_mangle]
pub unsafe extern "C" fn nova_core_penalize_frecency(
    handle: *mut NovaCore,
    id: *const c_char,
    divisor: f64,
) -> *mut c_char {
    if handle.is_null() || id.is_null() {
        return frecency_error_response("Invalid handle or id");
    }

    let core = &mut *handle;

    let id_str = match CStr::from_ptr(id).to_str() {
        Ok(s) => s,
        Err(_) => return frecency_error_response("Invalid id encoding"),
    };

    // Check if item exists
    if core.frecency.get_entry(id_str).is_none() {
        return frecency_error_response(&format!("Item '{}' not found in frecency data", id_str));
    }

    core.frecency.penalize(id_str, divisor);
    core.frecency.flush(); // Persist immediately

    let new_score = core.frecency.calculate(id_str);

    let response = serde_json::json!({
        "success": true,
        "id": id_str,
        "newScore": new_score
    });

    let json = serde_json::to_string(&response).unwrap_or_default();
    CString::new(json)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// Helper function to create a frecency error response.
fn frecency_error_response(message: &str) -> *mut c_char {
    let response = FrecencyResponse {
        success: false,
        error: Some(message.to_string()),
        stats: None,
        top_items: None,
    };
    let json = serde_json::to_string(&response).unwrap_or_default();
    CString::new(json)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}
