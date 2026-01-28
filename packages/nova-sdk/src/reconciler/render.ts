/**
 * Nova Render System
 *
 * Creates a React reconciler instance and provides the render/unmount functions.
 */

import Reconciler from "react-reconciler";
import type { ReactElement } from "react";
import {
  createHostConfig,
  setRenderCallback,
  type NovaContainer,
} from "./host-config.js";
import type { ComponentData } from "../types/index.js";

// ─────────────────────────────────────────────────────────────────────────────
// Reconciler Setup
// ─────────────────────────────────────────────────────────────────────────────

// Create the reconciler with our host config
const hostConfig = createHostConfig();
const reconciler = Reconciler(hostConfig);

// Store container instances per command ID
const containers = new Map<
  string,
  {
    container: ReturnType<typeof reconciler.createContainer>;
    containerInfo: NovaContainer;
  }
>();

// ─────────────────────────────────────────────────────────────────────────────
// Render Callback Setup
// ─────────────────────────────────────────────────────────────────────────────

// Set up the render callback to send data to Nova
setRenderCallback((data: ComponentData) => {
  // Nova global is injected by the Deno host
  if (typeof Nova !== "undefined" && Nova.render) {
    Nova.render(data);
  } else {
    // Development fallback - log the data
    console.log("[Nova SDK] Rendered:", JSON.stringify(data, null, 2));
  }
});

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Render a React element to Nova's UI.
 *
 * This should be called from a command handler.
 *
 * @param element - The React element to render (e.g., <List>...</List>)
 *
 * @example
 * ```tsx
 * registerCommand("my-command", () => {
 *   render(<MyComponent />);
 * });
 *
 * function MyComponent() {
 *   return (
 *     <List>
 *       <List.Item id="1" title="Hello" />
 *     </List>
 *   );
 * }
 * ```
 */
export function render(element: ReactElement): void {
  const commandId = globalThis.__nova_current_command ?? "default";

  let entry = containers.get(commandId);

  if (!entry) {
    // Create a new container for this command
    const containerInfo: NovaContainer = { root: null };
    const container = reconciler.createContainer(
      containerInfo,
      0, // ConcurrentRoot would be 1, but we use legacy mode for simplicity
      null, // hydrationCallbacks
      false, // isStrictMode
      null, // concurrentUpdatesByDefaultOverride
      commandId, // identifierPrefix
      (error: Error) => {
        console.error("[Nova SDK] Render error:", error);
      },
      null // transitionCallbacks
    );
    entry = { container, containerInfo };
    containers.set(commandId, entry);
  }

  // Update the container with the new element
  reconciler.updateContainer(element, entry.container, null, () => {
    // Callback after render completes (optional)
  });
}

/**
 * Unmount the component for a command.
 *
 * This cleans up the React tree and frees resources.
 *
 * @param commandId - The command ID to unmount (defaults to current command)
 */
export function unmount(commandId?: string): void {
  const id = commandId ?? globalThis.__nova_current_command ?? "default";
  const entry = containers.get(id);

  if (entry) {
    // Unmount by rendering null
    reconciler.updateContainer(null, entry.container, null, () => {
      containers.delete(id);
    });
  }
}

/**
 * Get the number of active command containers.
 * Useful for debugging.
 */
export function getActiveContainerCount(): number {
  return containers.size;
}

// ─────────────────────────────────────────────────────────────────────────────
// Global Types
// ─────────────────────────────────────────────────────────────────────────────

// Note: Nova global is declared in types/api.ts
declare global {
  var __nova_current_command: string | undefined;
}
