//! Deno ops for Nova extension API.
//!
//! This module defines all the operations that extensions can call from JavaScript.
//! Each op is exposed via deno_core's #[op2] macro and registered with the
//! nova_extension! macro.

use std::cell::RefCell;
use std::rc::Rc;

use deno_core::{error::AnyError, op2, OpState};

use crate::extensions::components::{Component, Validate};

use super::context::NovaContext;
use super::types::{FetchMethod, FetchRequest, FetchResponse};

// ─────────────────────────────────────────────────────────────────────────────
// Clipboard Ops
// ─────────────────────────────────────────────────────────────────────────────

/// Copy text to the system clipboard.
#[op2(fast)]
fn op_nova_clipboard_copy(state: &mut OpState, #[string] text: String) -> Result<(), AnyError> {
    let ctx = state.borrow::<NovaContext>();
    ctx.check_permission("clipboard")?;
    ctx.platform
        .clipboard_write(&text)
        .map_err(|e| anyhow::anyhow!("Clipboard write failed: {}", e))
}

/// Read text from the system clipboard.
#[op2]
#[string]
fn op_nova_clipboard_read(state: &mut OpState) -> Result<String, AnyError> {
    let ctx = state.borrow::<NovaContext>();
    ctx.check_permission("clipboard")?;
    ctx.platform
        .clipboard_read()
        .ok_or_else(|| anyhow::anyhow!("Clipboard is empty or unavailable"))
}

// ─────────────────────────────────────────────────────────────────────────────
// Storage Ops
// ─────────────────────────────────────────────────────────────────────────────

/// Get a value from extension storage.
#[op2]
#[serde]
fn op_nova_storage_get(
    state: &mut OpState,
    #[string] key: String,
) -> Result<Option<serde_json::Value>, AnyError> {
    let ctx = state.borrow::<NovaContext>();
    ctx.storage.get(&key)
}

/// Set a value in extension storage.
#[op2]
fn op_nova_storage_set(
    state: &mut OpState,
    #[string] key: String,
    #[serde] value: serde_json::Value,
) -> Result<(), AnyError> {
    let ctx = state.borrow_mut::<NovaContext>();
    ctx.storage.set(&key, value)
}

/// Remove a key from extension storage.
#[op2(fast)]
fn op_nova_storage_remove(state: &mut OpState, #[string] key: String) -> Result<(), AnyError> {
    let ctx = state.borrow_mut::<NovaContext>();
    ctx.storage.remove(&key)
}

/// Get all keys in extension storage.
#[op2]
#[serde]
fn op_nova_storage_keys(state: &mut OpState) -> Result<Vec<String>, AnyError> {
    let ctx = state.borrow::<NovaContext>();
    ctx.storage.keys()
}

// ─────────────────────────────────────────────────────────────────────────────
// Preferences Ops
// ─────────────────────────────────────────────────────────────────────────────

/// Get a preference value by key.
#[op2]
#[serde]
fn op_nova_preferences_get(
    state: &mut OpState,
    #[string] key: String,
) -> Result<Option<serde_json::Value>, AnyError> {
    let ctx = state.borrow::<NovaContext>();
    Ok(ctx.preferences.get(&key).cloned())
}

/// Get all preferences as a JSON object.
#[op2]
#[serde]
fn op_nova_preferences_all(state: &mut OpState) -> Result<serde_json::Value, AnyError> {
    let ctx = state.borrow::<NovaContext>();
    Ok(serde_json::to_value(&ctx.preferences)?)
}

// ─────────────────────────────────────────────────────────────────────────────
// Fetch Op
// ─────────────────────────────────────────────────────────────────────────────

/// Perform an HTTP fetch request.
#[op2(async)]
#[serde]
async fn op_nova_fetch(
    state: Rc<RefCell<OpState>>,
    #[serde] request: FetchRequest,
) -> Result<FetchResponse, AnyError> {
    // Parse URL and check domain permissions
    let url = url::Url::parse(&request.url)
        .map_err(|e| anyhow::anyhow!("Invalid URL '{}': {}", request.url, e))?;

    let domain = url.host_str().unwrap_or("");

    {
        let state_ref = state.borrow();
        let ctx = state_ref.borrow::<NovaContext>();

        // Check if domain is in allowed list
        if !ctx.permissions.network.contains(&domain.to_string())
            && !ctx.permissions.network.iter().any(|d| d == "*")
        {
            return Err(anyhow::anyhow!(
                "Network access to '{}' not allowed. Add it to permissions.network in nova.toml",
                domain
            ));
        }
    }

    // Build the reqwest client and request
    let client = reqwest::Client::new();

    let mut req_builder = match request.method {
        FetchMethod::Get => client.get(&request.url),
        FetchMethod::Post => client.post(&request.url),
        FetchMethod::Put => client.put(&request.url),
        FetchMethod::Delete => client.delete(&request.url),
        FetchMethod::Patch => client.patch(&request.url),
        FetchMethod::Head => client.head(&request.url),
        FetchMethod::Options => client.request(reqwest::Method::OPTIONS, &request.url),
    };

    // Add headers
    for (key, value) in &request.headers {
        req_builder = req_builder.header(key.as_str(), value.as_str());
    }

    // Add body if present
    if let Some(body) = request.body {
        req_builder = req_builder.body(body);
    }

    // Execute request
    let response = req_builder
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

    let status = response.status().as_u16();

    // Collect response headers
    let headers: std::collections::HashMap<String, String> = response
        .headers()
        .iter()
        .filter_map(|(k, v)| {
            v.to_str()
                .ok()
                .map(|val| (k.as_str().to_string(), val.to_string()))
        })
        .collect();

    // Read body as text
    let body = response
        .text()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;

    Ok(FetchResponse {
        status,
        headers,
        body,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// System Ops
// ─────────────────────────────────────────────────────────────────────────────

/// Open a URL in the default browser.
#[op2(fast)]
fn op_nova_open_url(state: &mut OpState, #[string] url: String) -> Result<(), AnyError> {
    let ctx = state.borrow::<NovaContext>();
    ctx.platform
        .open_url(&url)
        .map_err(|e| anyhow::anyhow!("Failed to open URL: {}", e))
}

/// Open a file or directory in the default application.
#[op2(fast)]
fn op_nova_open_path(state: &mut OpState, #[string] path: String) -> Result<(), AnyError> {
    let ctx = state.borrow::<NovaContext>();
    ctx.platform
        .open_file(&path)
        .map_err(|e| anyhow::anyhow!("Failed to open path: {}", e))
}

/// Show a system notification.
#[op2(fast)]
fn op_nova_notify(
    state: &mut OpState,
    #[string] title: String,
    #[string] body: String,
) -> Result<(), AnyError> {
    let ctx = state.borrow::<NovaContext>();
    ctx.check_permission("notifications")?;
    ctx.platform
        .show_notification(&title, &body)
        .map_err(|e| anyhow::anyhow!("Failed to show notification: {}", e))
}

/// Request the Nova window to close.
#[op2(fast)]
fn op_nova_close_window(state: &mut OpState) -> Result<(), AnyError> {
    let ctx = state.borrow_mut::<NovaContext>();
    ctx.should_close = true;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Render Op
// ─────────────────────────────────────────────────────────────────────────────

/// Render a component tree to the Nova UI.
///
/// The component is deserialized to a strongly-typed Component enum and
/// validated before being stored.
#[op2]
fn op_nova_render(state: &mut OpState, #[serde] component: Component) -> Result<(), AnyError> {
    // Validate the component tree
    component
        .validate()
        .map_err(|e| anyhow::anyhow!("Component validation failed: {}", e))?;

    let ctx = state.borrow_mut::<NovaContext>();
    ctx.set_rendered_component(component);
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Navigation Ops
// ─────────────────────────────────────────────────────────────────────────────

/// Push a new view onto the navigation stack.
#[op2]
fn op_nova_navigation_push(
    state: &mut OpState,
    #[serde] component: Component,
) -> Result<(), AnyError> {
    // Validate the component tree
    component
        .validate()
        .map_err(|e| anyhow::anyhow!("Component validation failed: {}", e))?;

    let ctx = state.borrow_mut::<NovaContext>();
    ctx.navigation_stack.push(component);
    Ok(())
}

/// Pop the top view from the navigation stack.
#[op2(fast)]
fn op_nova_navigation_pop(state: &mut OpState) -> Result<bool, AnyError> {
    let ctx = state.borrow_mut::<NovaContext>();
    Ok(ctx.navigation_stack.pop().is_some())
}

/// Get the current navigation stack depth.
#[op2(fast)]
fn op_nova_navigation_depth(state: &mut OpState) -> Result<u32, AnyError> {
    let ctx = state.borrow::<NovaContext>();
    Ok(ctx.navigation_stack.len() as u32)
}

// ─────────────────────────────────────────────────────────────────────────────
// Extension Registration
// ─────────────────────────────────────────────────────────────────────────────

deno_core::extension!(
    nova_extension,
    ops = [
        // Clipboard
        op_nova_clipboard_copy,
        op_nova_clipboard_read,
        // Storage
        op_nova_storage_get,
        op_nova_storage_set,
        op_nova_storage_remove,
        op_nova_storage_keys,
        // Preferences
        op_nova_preferences_get,
        op_nova_preferences_all,
        // Fetch
        op_nova_fetch,
        // System
        op_nova_open_url,
        op_nova_open_path,
        op_nova_notify,
        op_nova_close_window,
        // Render
        op_nova_render,
        // Navigation
        op_nova_navigation_push,
        op_nova_navigation_pop,
        op_nova_navigation_depth,
    ],
    esm_entry_point = "ext:nova/runtime.js",
    esm = [dir "src/extensions/js", "runtime.js"],
);
