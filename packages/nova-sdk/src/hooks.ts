/**
 * Nova Hooks System
 *
 * Implements React-like hooks for Nova components.
 * Uses a simple render context that tracks hook state.
 */

import type { NovaElement } from "./jsx-runtime.js";
import { serializeElement } from "./jsx-runtime.js";

// ─────────────────────────────────────────────────────────────────────────────
// Render Context
// ─────────────────────────────────────────────────────────────────────────────

interface HookState {
  value: unknown;
  deps?: unknown[];
}

interface RenderContext {
  /** Current hook index during render */
  hookIndex: number;
  /** Hook states for the current component */
  hookStates: HookState[];
  /** Whether this is the first render */
  isFirstRender: boolean;
  /** Pending effects to run after render */
  pendingEffects: Array<() => void | (() => void)>;
  /** Cleanup functions from previous effects */
  effectCleanups: Map<number, () => void>;
  /** Request a re-render */
  requestRender: () => void;
}

let currentContext: RenderContext | null = null;

/**
 * Get the current render context or throw.
 */
function getContext(): RenderContext {
  if (!currentContext) {
    throw new Error(
      "Hooks can only be called inside a component during render. " +
      "Make sure you are not calling hooks conditionally or in event handlers."
    );
  }
  return currentContext;
}

/**
 * Get the next hook state slot.
 */
function getHookState<T>(initialValue: () => T): { state: T; index: number; isNew: boolean } {
  const ctx = getContext();
  const index = ctx.hookIndex++;

  if (ctx.isFirstRender || index >= ctx.hookStates.length) {
    // First render or new hook - initialize state
    ctx.hookStates[index] = { value: initialValue() };
    return { state: ctx.hookStates[index].value as T, index, isNew: true };
  }

  return { state: ctx.hookStates[index].value as T, index, isNew: false };
}

// ─────────────────────────────────────────────────────────────────────────────
// useState Hook
// ─────────────────────────────────────────────────────────────────────────────

export type SetStateAction<S> = S | ((prevState: S) => S);
export type Dispatch<A> = (action: A) => void;

/**
 * Returns a stateful value and a function to update it.
 *
 * @param initialState - Initial state value or function that returns it
 * @returns Tuple of [state, setState]
 */
export function useState<S>(
  initialState: S | (() => S)
): [S, Dispatch<SetStateAction<S>>] {
  const ctx = getContext();
  const { state, index } = getHookState(() =>
    typeof initialState === "function"
      ? (initialState as () => S)()
      : initialState
  );

  const setState: Dispatch<SetStateAction<S>> = (action) => {
    const prevState = ctx.hookStates[index].value as S;
    const nextState =
      typeof action === "function"
        ? (action as (prev: S) => S)(prevState)
        : action;

    if (!Object.is(prevState, nextState)) {
      ctx.hookStates[index].value = nextState;
      ctx.requestRender();
    }
  };

  return [state as S, setState];
}

// ─────────────────────────────────────────────────────────────────────────────
// useEffect Hook
// ─────────────────────────────────────────────────────────────────────────────

export type EffectCallback = () => void | (() => void);
export type DependencyList = readonly unknown[];

/**
 * Accepts a function that contains imperative, possibly effectful code.
 * Effects run after render is complete.
 *
 * @param effect - Effect function (can return a cleanup function)
 * @param deps - Dependencies array (effect re-runs when these change)
 */
export function useEffect(effect: EffectCallback, deps?: DependencyList): void {
  const ctx = getContext();
  const { index, isNew } = getHookState(() => undefined);

  // Check if dependencies changed
  const prevDeps = ctx.hookStates[index].deps;
  const depsChanged = isNew || !deps || !prevDeps || !depsEqual(prevDeps, deps);

  if (depsChanged) {
    ctx.hookStates[index].deps = deps ? [...deps] : undefined;

    // Schedule the effect
    ctx.pendingEffects.push(() => {
      // Run cleanup from previous effect
      const cleanup = ctx.effectCleanups.get(index);
      if (cleanup) {
        cleanup();
        ctx.effectCleanups.delete(index);
      }

      // Run the effect
      const newCleanup = effect();
      if (typeof newCleanup === "function") {
        ctx.effectCleanups.set(index, newCleanup);
      }
    });
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// useMemo Hook
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Returns a memoized value. Re-computes only when dependencies change.
 *
 * @param factory - Function that computes the value
 * @param deps - Dependencies array
 * @returns The memoized value
 */
export function useMemo<T>(factory: () => T, deps: DependencyList): T {
  const ctx = getContext();
  const { state, index, isNew } = getHookState<{ value: T; deps: unknown[] }>(() => ({
    value: factory(),
    deps: [...deps],
  }));

  // Check if dependencies changed
  if (!isNew && depsEqual(state.deps, deps)) {
    return state.value;
  }

  // Recompute
  const newValue = factory();
  ctx.hookStates[index].value = { value: newValue, deps: [...deps] };
  return newValue;
}

// ─────────────────────────────────────────────────────────────────────────────
// useCallback Hook
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Returns a memoized callback. Only changes when dependencies change.
 *
 * @param callback - The callback function
 * @param deps - Dependencies array
 * @returns The memoized callback
 */
export function useCallback<T extends (...args: unknown[]) => unknown>(
  callback: T,
  deps: DependencyList
): T {
  return useMemo(() => callback, deps);
}

// ─────────────────────────────────────────────────────────────────────────────
// useRef Hook
// ─────────────────────────────────────────────────────────────────────────────

export interface MutableRefObject<T> {
  current: T;
}

/**
 * Returns a mutable ref object.
 *
 * @param initialValue - Initial value for ref.current
 * @returns A ref object
 */
export function useRef<T>(initialValue: T): MutableRefObject<T> {
  const { state } = getHookState<MutableRefObject<T>>(() => ({
    current: initialValue,
  }));
  return state;
}

// ─────────────────────────────────────────────────────────────────────────────
// useReducer Hook
// ─────────────────────────────────────────────────────────────────────────────

export type Reducer<S, A> = (prevState: S, action: A) => S;

/**
 * Alternative to useState for complex state logic.
 *
 * @param reducer - Reducer function
 * @param initialState - Initial state
 * @returns Tuple of [state, dispatch]
 */
export function useReducer<S, A>(
  reducer: Reducer<S, A>,
  initialState: S
): [S, Dispatch<A>] {
  const ctx = getContext();
  const { state, index } = getHookState(() => initialState);

  const dispatch: Dispatch<A> = (action) => {
    const prevState = ctx.hookStates[index].value as S;
    const nextState = reducer(prevState, action);

    if (!Object.is(prevState, nextState)) {
      ctx.hookStates[index].value = nextState;
      ctx.requestRender();
    }
  };

  return [state as S, dispatch];
}

// ─────────────────────────────────────────────────────────────────────────────
// useId Hook
// ─────────────────────────────────────────────────────────────────────────────

let idCounter = 0;

/**
 * Generate a unique ID for accessibility attributes.
 *
 * @returns A unique ID string
 */
export function useId(): string {
  const { state } = getHookState(() => `nova-id-${++idCounter}`);
  return state as string;
}

// ─────────────────────────────────────────────────────────────────────────────
// Utility Functions
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Compare two dependency arrays for equality.
 */
function depsEqual(a: readonly unknown[], b: readonly unknown[]): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (!Object.is(a[i], b[i])) return false;
  }
  return true;
}

// ─────────────────────────────────────────────────────────────────────────────
// Render System
// ─────────────────────────────────────────────────────────────────────────────

interface ComponentInstance {
  component: () => NovaElement | null;
  context: RenderContext;
}

const componentInstances = new Map<string, ComponentInstance>();

/**
 * Create a render context for a component.
 */
function createRenderContext(componentId: string): RenderContext {
  const existing = componentInstances.get(componentId);

  return {
    hookIndex: 0,
    hookStates: existing?.context.hookStates ?? [],
    isFirstRender: !existing,
    pendingEffects: [],
    effectCleanups: existing?.context.effectCleanups ?? new Map(),
    requestRender: () => {
      // Schedule a re-render on next tick
      queueMicrotask(() => {
        const instance = componentInstances.get(componentId);
        if (instance) {
          renderComponent(componentId, instance.component);
        }
      });
    },
  };
}

/**
 * Render a component and return serialized data.
 */
function renderComponent(
  componentId: string,
  component: () => NovaElement | null
): void {
  const context = createRenderContext(componentId);

  // Set current context
  currentContext = context;

  try {
    // Call the component
    const element = component();

    if (element === null) {
      throw new Error("Component returned null");
    }

    // Store the instance
    componentInstances.set(componentId, { component, context });

    // Serialize and render
    const data = serializeElement(element);
    Nova.render(data);

    // Run effects
    for (const effect of context.pendingEffects) {
      effect();
    }
  } finally {
    currentContext = null;
  }
}

/**
 * Mount a component as the root of a command.
 * This should be called from Nova.registerCommand handlers.
 *
 * @param component - The component function to render
 */
export function render(component: () => NovaElement | null): void {
  const commandId = globalThis.__nova_current_command ?? "default";
  renderComponent(commandId, component);
}

/**
 * Unmount a component and run cleanup effects.
 */
export function unmount(componentId: string): void {
  const instance = componentInstances.get(componentId);
  if (instance) {
    // Run all cleanup functions
    for (const cleanup of instance.context.effectCleanups.values()) {
      cleanup();
    }
    componentInstances.delete(componentId);
  }
}

// Extend global types
declare global {
  var __nova_current_command: string | null;
}
