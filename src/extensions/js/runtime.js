// Nova Runtime - provides global Nova API to extensions
// This file is loaded before any extension code runs.

// -----------------------------------------------------------------------------
// Timer Polyfills (for React reconciler compatibility)
// -----------------------------------------------------------------------------

// Simple synchronous "setTimeout" polyfill - executes immediately
// This is sufficient for React reconciler in synchronous mode
if (typeof globalThis.setTimeout === 'undefined') {

  let timerId = 0;
  const timers = new Map();

  globalThis.setTimeout = (callback, delay, ...args) => {
    const id = ++timerId;
    // For our use case, we execute callbacks immediately (synchronous mode)
    // A proper async implementation would need Deno.core scheduling
    try {
      callback(...args);
    } catch (e) {
      console.error("[Nova Runtime] setTimeout callback error:", e);
    }
    return id;
  };

  globalThis.clearTimeout = (id) => {
    // No-op for immediate execution
  };

  globalThis.setInterval = (callback, delay, ...args) => {
    // For Nova extensions, we don't need real intervals
    // Return a dummy ID
    return ++timerId;
  };

  globalThis.clearInterval = (id) => {
    // No-op
  };

  // queueMicrotask should be available, but just in case
  if (typeof globalThis.queueMicrotask === 'undefined') {
    globalThis.queueMicrotask = (callback) => {
      Promise.resolve().then(callback);
    };
  }
}

const {
  op_nova_clipboard_copy,
  op_nova_clipboard_read,
  op_nova_storage_get,
  op_nova_storage_set,
  op_nova_storage_remove,
  op_nova_storage_keys,
  op_nova_preferences_get,
  op_nova_preferences_all,
  op_nova_fetch,
  op_nova_open_url,
  op_nova_open_path,
  op_nova_notify,
  op_nova_close_window,
  op_nova_render,
  op_nova_navigation_push,
  op_nova_navigation_pop,
  op_nova_navigation_depth,
} = Deno.core.ops;

/**
 * Nova API - the main interface for extensions to interact with Nova.
 * @namespace Nova
 */
globalThis.Nova = {
  /**
   * Clipboard operations.
   * Requires "clipboard" permission in nova.toml.
   */
  clipboard: {
    /**
     * Copy text to the system clipboard.
     * @param {string} text - The text to copy
     */
    copy: (text) => op_nova_clipboard_copy(text),

    /**
     * Read text from the system clipboard.
     * @returns {string} The clipboard contents
     */
    read: () => op_nova_clipboard_read(),
  },

  /**
   * Persistent key-value storage for the extension.
   * Data is persisted across Nova sessions.
   */
  storage: {
    /**
     * Get a value from storage.
     * @param {string} key - The key to retrieve
     * @returns {any} The stored value, or undefined if not found
     */
    get: (key) => op_nova_storage_get(key),

    /**
     * Set a value in storage.
     * @param {string} key - The key to set
     * @param {any} value - The value to store (must be JSON-serializable)
     */
    set: (key, value) => op_nova_storage_set(key, value),

    /**
     * Remove a key from storage.
     * @param {string} key - The key to remove
     */
    remove: (key) => op_nova_storage_remove(key),

    /**
     * Get all keys in storage.
     * @returns {string[]} Array of all storage keys
     */
    keys: () => op_nova_storage_keys(),
  },

  /**
   * User-configured preferences for this extension.
   * Preferences are defined in nova.toml and configured by users.
   */
  preferences: {
    /**
     * Get a preference value by key.
     * @param {string} key - The preference key
     * @returns {any} The preference value, or undefined if not set
     */
    get: (key) => op_nova_preferences_get(key),

    /**
     * Get all preferences as an object.
     * @returns {Object} All preferences
     */
    all: () => op_nova_preferences_all(),
  },

  /**
   * Perform HTTP requests.
   * Requires the target domain to be listed in permissions.network.
   *
   * @param {string} url - The URL to fetch
   * @param {Object} [options] - Fetch options
   * @param {string} [options.method="GET"] - HTTP method
   * @param {Object} [options.headers] - Request headers
   * @param {string} [options.body] - Request body
   * @returns {Promise<{status: number, headers: Object, body: string}>}
   */
  fetch: async (url, options = {}) => {
    return await op_nova_fetch({
      url,
      method: options.method || "GET",
      headers: options.headers || {},
      body: options.body || null,
    });
  },

  /**
   * System operations.
   */
  system: {
    /**
     * Open a URL in the default browser.
     * @param {string} url - The URL to open
     */
    openUrl: (url) => op_nova_open_url(url),

    /**
     * Open a file or directory in the default application.
     * @param {string} path - The file path to open
     */
    openPath: (path) => op_nova_open_path(path),

    /**
     * Show a system notification.
     * Requires "notifications" permission.
     * @param {string} title - Notification title
     * @param {string} [body=""] - Notification body
     */
    notify: (title, body = "") => op_nova_notify(title, body),

    /**
     * Close the Nova window.
     */
    closeWindow: () => op_nova_close_window(),
  },

  /**
   * Render a component tree to the Nova UI.
   *
   * @param {Object} component - The component tree to render
   *
   * @example
   * Nova.render({
   *   type: "List",
   *   props: { searchBarPlaceholder: "Search items..." },
   *   children: [
   *     {
   *       type: "List.Item",
   *       props: { title: "Item 1", subtitle: "Description" }
   *     }
   *   ]
   * });
   */
  render: (component) => op_nova_render(component),

  /**
   * Navigation stack for multi-view extensions.
   */
  navigation: {
    /**
     * Push a new view onto the navigation stack.
     * @param {Object} component - The component to push
     */
    push: (component) => op_nova_navigation_push(component),

    /**
     * Pop the top view from the navigation stack.
     * @returns {boolean} True if a view was popped, false if stack was empty
     */
    pop: () => op_nova_navigation_pop(),

    /**
     * Get the current navigation stack depth.
     * @returns {number} The number of views in the stack
     */
    depth: () => op_nova_navigation_depth(),
  },
};

// Command registration system
globalThis.__nova_commands = {};
globalThis.__nova_current_command = null;

/**
 * Register a command handler.
 * This is called by extensions to register their command implementations.
 *
 * @param {string} name - The command name (must match a command id in nova.toml)
 * @param {Function} handler - The command handler function
 *
 * @example
 * Nova.registerCommand("search", async (props) => {
 *   const query = props.argument || "";
 *   const results = await searchDatabase(query);
 *   Nova.render({ type: "List", children: results.map(toListItem) });
 * });
 */
Nova.registerCommand = (name, handler) => {
  globalThis.__nova_commands[name] = handler;
};

/**
 * Internal: Execute a registered command.
 * Called by the Nova runtime when a command is triggered.
 *
 * @param {string} commandId - The command to execute
 * @param {Object} props - Properties passed to the command
 * @returns {Promise<void>}
 */
globalThis.__nova_execute_command = async (commandId, props = {}) => {
  const handler = globalThis.__nova_commands[commandId];
  if (!handler) {
    throw new Error(`Command not found: ${commandId}`);
  }

  globalThis.__nova_current_command = commandId;

  try {
    await handler(props);
  } finally {
    globalThis.__nova_current_command = null;
  }
};

// -----------------------------------------------------------------------------
// Event Dispatch System
// -----------------------------------------------------------------------------

// Event handlers registered by extensions
globalThis.__nova_event_handlers = {};

/**
 * Register an event handler.
 * Called by extensions to handle actions, form submissions, etc.
 *
 * @param {string} eventId - Unique event identifier
 * @param {Function} handler - Event handler function
 */
Nova.registerEventHandler = (eventId, handler) => {
  globalThis.__nova_event_handlers[eventId] = handler;
};

/**
 * Internal: Dispatch an event to a registered handler.
 * Called by the Nova runtime when an action is triggered.
 *
 * @param {string} eventId - The event ID to dispatch (e.g., "copy:123", "submit")
 * @param {Object} eventData - Data associated with the event
 * @returns {Object} Result with success status and optional error
 */
globalThis.__nova_dispatch_event = (eventId, eventData = {}) => {
  try {
    // First, check if it's a registered callback (cb_xxx format)
    const handler = globalThis.__nova_event_handlers[eventId];

    if (handler) {
      const result = handler(eventData);
      // Handle async handlers
      if (result instanceof Promise) {
        // For now, we execute synchronously. Async support will be added later.
        return { success: true };
      }
      return { success: true, result };
    }

    // Parse action format: "action:arg" or just "action"
    const colonIndex = eventId.indexOf(':');
    let action, arg;

    if (colonIndex !== -1) {
      action = eventId.substring(0, colonIndex);
      arg = eventId.substring(colonIndex + 1);
    } else {
      action = eventId;
      arg = null;
    }

    // Check for built-in actions
    switch (action) {
      case 'submit':
        // Form submission - eventData contains form values
        return { success: true, action: 'submit', data: eventData };

      case 'search':
        // Search text changed
        return { success: true, action: 'search', query: eventData.query };

      case 'select':
        // Item selected
        return { success: true, action: 'select', itemId: arg || eventData.itemId };

      default:
        // Custom action - pass to current command's context
        return {
          success: true,
          action: action,
          arg: arg,
          data: eventData
        };
    }
  } catch (e) {
    return { success: false, error: e.message || String(e) };
  }
};

/**
 * Internal: Get the current rendered component.
 * Called by the runtime to retrieve the latest component tree.
 *
 * @returns {Object|null} The rendered component or null
 */
globalThis.__nova_get_rendered_component = () => {
  // This is set by op_nova_render when extension calls Nova.render()
  return globalThis.__nova_rendered_component || null;
};

// Store rendered component for retrieval
const originalRender = Nova.render;
Nova.render = (component) => {
  globalThis.__nova_rendered_component = component;
  return originalRender(component);
};

// Freeze the Nova API to prevent modifications
Object.freeze(Nova.clipboard);
Object.freeze(Nova.storage);
Object.freeze(Nova.preferences);
Object.freeze(Nova.system);
Object.freeze(Nova.navigation);
Object.freeze(Nova);
