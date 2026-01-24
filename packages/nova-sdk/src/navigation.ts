/**
 * Navigation Hook
 *
 * Provides a hook for multi-view navigation in Nova extensions.
 */

import type { NovaElement } from "./jsx-runtime.js";
import { serializeElement } from "./jsx-runtime.js";
import { useMemo, useCallback, useState } from "./hooks.js";
import type { ComponentData } from "./types/index.js";

/**
 * Navigation state and methods.
 */
export interface UseNavigationReturn {
  /**
   * Push a new view onto the navigation stack.
   * Accepts either a serialized component or a JSX element.
   */
  push: (view: ComponentData | NovaElement) => void;

  /**
   * Pop the current view from the navigation stack.
   * Returns true if a view was popped, false if at root.
   */
  pop: () => boolean;

  /**
   * Get the current stack depth.
   */
  depth: () => number;

  /**
   * Whether we can go back (depth > 0).
   */
  canGoBack: boolean;
}

/**
 * Hook for managing multi-view navigation.
 *
 * @example
 * ```tsx
 * function MyCommand() {
 *   const { push, pop, canGoBack } = useNavigation();
 *
 *   return (
 *     <List>
 *       <List.Item
 *         id="item-1"
 *         title="View Details"
 *         actions={{
 *           children: [{
 *             id: "open",
 *             title: "Open",
 *             onAction: "open-detail"
 *           }]
 *         }}
 *       />
 *     </List>
 *   );
 *
 *   // In action handler:
 *   // push(<DetailView item={selectedItem} />);
 * }
 * ```
 */
export function useNavigation(): UseNavigationReturn {
  // Track depth locally for canGoBack
  const [depth, setDepth] = useState(() => Nova.navigation.depth());

  const push = (view: ComponentData | NovaElement): void => {
    // Serialize if it's a JSX element
    const data = isNovaElement(view) ? serializeElement(view) : view;
    Nova.navigation.push(data);
    setDepth((d) => d + 1);
  };

  const pop = (): boolean => {
    const result = Nova.navigation.pop();
    if (result) {
      setDepth((d) => Math.max(0, d - 1));
    }
    return result;
  };

  const getDepth = (): number => {
    return Nova.navigation.depth();
  };

  const canGoBack = depth > 0;

  return {
    push,
    pop,
    depth: getDepth,
    canGoBack,
  };
}

/**
 * Type guard for NovaElement.
 */
function isNovaElement(value: unknown): value is NovaElement {
  return (
    typeof value === "object" &&
    value !== null &&
    "$$typeof" in value &&
    (value as { $$typeof: symbol }).$$typeof === Symbol.for("nova.element")
  );
}

/**
 * Action callback registration helper.
 *
 * This provides a way to register callbacks that can be triggered by actions.
 * The callback ID is returned for use in action definitions.
 */
const callbacks = new Map<string, (...args: unknown[]) => void>();
let callbackCounter = 0;

/**
 * Register a callback for use in action handlers.
 * Returns a callback ID that can be used in onAction.
 *
 * @example
 * ```tsx
 * const handleOpen = registerCallback((itemId) => {
 *   navigation.push(<DetailView itemId={itemId} />);
 * });
 *
 * <List.Item
 *   id="1"
 *   title="Item"
 *   actions={{
 *     children: [{
 *       id: "open",
 *       title: "Open",
 *       onAction: handleOpen
 *     }]
 *   }}
 * />
 * ```
 */
export function registerCallback<T extends (...args: unknown[]) => void>(
  callback: T
): string {
  const id = `cb_${++callbackCounter}_${Date.now()}`;
  callbacks.set(id, callback);
  return id;
}

/**
 * Get a registered callback by ID.
 * Used internally by the runtime.
 */
export function getCallback(id: string): ((...args: unknown[]) => void) | undefined {
  return callbacks.get(id);
}

/**
 * Clear a registered callback.
 */
export function clearCallback(id: string): void {
  callbacks.delete(id);
}

// Export the callback map for the runtime
export { callbacks };
