/**
 * IPC Bridge
 *
 * Type-safe wrappers around the Nova global API.
 * These functions provide better TypeScript integration and
 * can be used directly instead of calling Nova.* methods.
 */

import type {
  FetchOptions,
  FetchResponse,
  ComponentData,
} from "./types/index.js";

// ─────────────────────────────────────────────────────────────────────────────
// Clipboard API
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Copy text to the system clipboard.
 * Requires "clipboard" permission in nova.toml.
 *
 * @param text - The text to copy
 */
export function clipboardCopy(text: string): void {
  Nova.clipboard.copy(text);
}

/**
 * Read text from the system clipboard.
 * Requires "clipboard" permission in nova.toml.
 *
 * @returns The clipboard contents
 */
export function clipboardRead(): string {
  return Nova.clipboard.read();
}

// ─────────────────────────────────────────────────────────────────────────────
// Storage API
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Get a value from persistent storage.
 *
 * @param key - The key to retrieve
 * @returns The stored value, or undefined if not found
 */
export function storageGet<T = unknown>(key: string): T | undefined {
  return Nova.storage.get(key) as T | undefined;
}

/**
 * Set a value in persistent storage.
 *
 * @param key - The key to set
 * @param value - The value to store (must be JSON-serializable)
 */
export function storageSet<T = unknown>(key: string, value: T): void {
  Nova.storage.set(key, value);
}

/**
 * Remove a key from persistent storage.
 *
 * @param key - The key to remove
 */
export function storageRemove(key: string): void {
  Nova.storage.remove(key);
}

/**
 * Get all keys in persistent storage.
 *
 * @returns Array of all storage keys
 */
export function storageKeys(): string[] {
  return Nova.storage.keys();
}

// ─────────────────────────────────────────────────────────────────────────────
// Preferences API
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Get a user-configured preference value.
 *
 * @param key - The preference key (as defined in nova.toml)
 * @returns The preference value, or undefined if not set
 */
export function getPreference<T = unknown>(key: string): T | undefined {
  return Nova.preferences.get(key) as T | undefined;
}

/**
 * Get all user-configured preferences.
 *
 * @returns Object containing all preferences
 */
export function getAllPreferences(): Record<string, unknown> {
  return Nova.preferences.all();
}

// ─────────────────────────────────────────────────────────────────────────────
// Fetch API
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Perform an HTTP fetch request.
 * Requires the target domain to be listed in permissions.network.
 *
 * @param url - The URL to fetch
 * @param options - Fetch options (method, headers, body)
 * @returns Response with status, headers, and body
 */
export async function fetch(
  url: string,
  options?: FetchOptions
): Promise<FetchResponse> {
  return Nova.fetch(url, options);
}

/**
 * Convenience method for JSON GET requests.
 *
 * @param url - The URL to fetch
 * @param headers - Optional additional headers
 * @returns Parsed JSON response
 */
export async function fetchJson<T = unknown>(
  url: string,
  headers?: Record<string, string>
): Promise<T> {
  const response = await Nova.fetch(url, {
    method: "GET",
    headers: {
      Accept: "application/json",
      ...headers,
    },
  });

  if (response.status >= 400) {
    throw new Error(`HTTP ${response.status}: ${response.body}`);
  }

  return JSON.parse(response.body) as T;
}

/**
 * Convenience method for JSON POST requests.
 *
 * @param url - The URL to post to
 * @param body - JSON body to send
 * @param headers - Optional additional headers
 * @returns Parsed JSON response
 */
export async function postJson<T = unknown, R = unknown>(
  url: string,
  body: T,
  headers?: Record<string, string>
): Promise<R> {
  const response = await Nova.fetch(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
      ...headers,
    },
    body: JSON.stringify(body),
  });

  if (response.status >= 400) {
    throw new Error(`HTTP ${response.status}: ${response.body}`);
  }

  return JSON.parse(response.body) as R;
}

// ─────────────────────────────────────────────────────────────────────────────
// System API
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Open a URL in the default browser.
 *
 * @param url - The URL to open
 */
export function openUrl(url: string): void {
  Nova.system.openUrl(url);
}

/**
 * Open a file or directory in the default application.
 *
 * @param path - The file path to open
 */
export function openPath(path: string): void {
  Nova.system.openPath(path);
}

/**
 * Show a system notification.
 * Requires "notifications" permission in nova.toml.
 *
 * @param title - Notification title
 * @param body - Optional notification body
 */
export function showNotification(title: string, body?: string): void {
  Nova.system.notify(title, body ?? "");
}

/**
 * Close the Nova window.
 */
export function closeWindow(): void {
  Nova.system.closeWindow();
}

// ─────────────────────────────────────────────────────────────────────────────
// Render API
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Render a component tree to the Nova UI.
 * Typically called via the render() function from hooks.ts instead.
 *
 * @param component - Serialized component data
 */
export function renderComponent(component: ComponentData): void {
  Nova.render(component);
}

// ─────────────────────────────────────────────────────────────────────────────
// Navigation API
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Push a new view onto the navigation stack.
 *
 * @param component - The component to push
 */
export function navigationPush(component: ComponentData): void {
  Nova.navigation.push(component);
}

/**
 * Pop the top view from the navigation stack.
 *
 * @returns True if a view was popped, false if stack was empty
 */
export function navigationPop(): boolean {
  return Nova.navigation.pop();
}

/**
 * Get the current navigation stack depth.
 *
 * @returns The number of views in the stack
 */
export function navigationDepth(): number {
  return Nova.navigation.depth();
}

// ─────────────────────────────────────────────────────────────────────────────
// Command Registration
// ─────────────────────────────────────────────────────────────────────────────

import type { CommandHandler } from "./types/index.js";

/**
 * Register a command handler.
 *
 * @param name - Command name (must match command id in nova.toml)
 * @param handler - Handler function
 */
export function registerCommand(name: string, handler: CommandHandler): void {
  Nova.registerCommand(name, handler);
}
