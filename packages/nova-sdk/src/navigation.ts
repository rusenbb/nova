/**
 * Navigation Hook
 *
 * Provides a hook for multi-view navigation in Nova extensions.
 */

import { useMemo, useCallback, useState } from "react";
import type { ComponentData } from "./types/index.js";

/**
 * Navigation state and methods.
 */
export interface UseNavigationReturn {
  /**
   * Push a new view onto the navigation stack.
   * Accepts serialized component data.
   */
  push: (view: ComponentData) => void;

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
 * }
 * ```
 */
export function useNavigation(): UseNavigationReturn {
  // Track depth locally for canGoBack
  const [stackDepth, setStackDepth] = useState(() => {
    if (typeof Nova !== "undefined" && Nova.navigation) {
      return Nova.navigation.depth();
    }
    return 0;
  });

  const push = useCallback((view: ComponentData): void => {
    if (typeof Nova !== "undefined" && Nova.navigation) {
      Nova.navigation.push(view);
      setStackDepth((d) => d + 1);
    }
  }, []);

  const pop = useCallback((): boolean => {
    if (typeof Nova !== "undefined" && Nova.navigation) {
      const result = Nova.navigation.pop();
      if (result) {
        setStackDepth((d) => Math.max(0, d - 1));
      }
      return result;
    }
    return false;
  }, []);

  const getDepth = useCallback((): number => {
    if (typeof Nova !== "undefined" && Nova.navigation) {
      return Nova.navigation.depth();
    }
    return stackDepth;
  }, [stackDepth]);

  const canGoBack = stackDepth > 0;

  return useMemo(
    () => ({
      push,
      pop,
      depth: getDepth,
      canGoBack,
    }),
    [push, pop, getDepth, canGoBack]
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
 *   navigation.push({ type: "Detail", markdown: "..." });
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

  // Also register with Nova runtime for event dispatch
  if (typeof Nova !== "undefined" && Nova.registerEventHandler) {
    Nova.registerEventHandler(id, callback as (...args: unknown[]) => void);
  }

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

// Note: Nova global is declared in types/api.ts
