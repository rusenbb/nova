/**
 * Nova API type definitions.
 * These types describe the global Nova API available in extensions.
 */

import type { ComponentData } from "./component.js";

/**
 * HTTP fetch method.
 */
export type FetchMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS";

/**
 * Options for Nova.fetch().
 */
export interface FetchOptions {
  /** HTTP method (default: "GET") */
  method?: FetchMethod;
  /** Request headers */
  headers?: Record<string, string>;
  /** Request body */
  body?: string;
}

/**
 * Response from Nova.fetch().
 */
export interface FetchResponse {
  /** HTTP status code */
  status: number;
  /** Response headers */
  headers: Record<string, string>;
  /** Response body */
  body: string;
}

/**
 * Clipboard API.
 */
export interface ClipboardAPI {
  /** Copy text to the system clipboard */
  copy(text: string): void;
  /** Read text from the system clipboard */
  read(): string;
}

/**
 * Storage API for persistent key-value storage.
 */
export interface StorageAPI {
  /** Get a value from storage */
  get<T = unknown>(key: string): T | undefined;
  /** Set a value in storage */
  set<T = unknown>(key: string, value: T): void;
  /** Remove a key from storage */
  remove(key: string): void;
  /** Get all keys in storage */
  keys(): string[];
}

/**
 * Preferences API for user-configured settings.
 */
export interface PreferencesAPI {
  /** Get a preference value by key */
  get<T = unknown>(key: string): T | undefined;
  /** Get all preferences as an object */
  all(): Record<string, unknown>;
}

/**
 * System API for OS interactions.
 */
export interface SystemAPI {
  /** Open a URL in the default browser */
  openUrl(url: string): void;
  /** Open a file or directory in the default application */
  openPath(path: string): void;
  /** Show a system notification */
  notify(title: string, body?: string): void;
  /** Close the Nova window */
  closeWindow(): void;
}

/**
 * Navigation API for multi-view extensions.
 */
export interface NavigationAPI {
  /** Push a new view onto the navigation stack */
  push(component: ComponentData): void;
  /** Pop the top view from the navigation stack */
  pop(): boolean;
  /** Get the current navigation stack depth */
  depth(): number;
}

/**
 * Command handler function type.
 */
export type CommandHandler = (props: CommandProps) => void | Promise<void>;

/**
 * Props passed to a command handler.
 */
export interface CommandProps {
  /** Optional argument passed to the command */
  argument?: string;
  /** Additional context */
  [key: string]: unknown;
}

/**
 * The global Nova API interface.
 */
export interface NovaAPI {
  /** Clipboard operations (requires "clipboard" permission) */
  clipboard: ClipboardAPI;
  /** Persistent key-value storage */
  storage: StorageAPI;
  /** User-configured preferences */
  preferences: PreferencesAPI;
  /** System operations */
  system: SystemAPI;
  /** Navigation stack for multi-view extensions */
  navigation: NavigationAPI;

  /**
   * Perform an HTTP fetch request.
   * Requires the target domain to be listed in permissions.network.
   */
  fetch(url: string, options?: FetchOptions): Promise<FetchResponse>;

  /**
   * Render a component tree to the Nova UI.
   */
  render(component: ComponentData): void;

  /**
   * Register a command handler.
   */
  registerCommand(name: string, handler: CommandHandler): void;

  /**
   * Register an event handler for action callbacks.
   * This is called internally by registerCallback().
   */
  registerEventHandler(eventId: string, handler: (...args: unknown[]) => void): void;
}

/**
 * Declare the global Nova API.
 */
declare global {
  const Nova: NovaAPI;
}

export {};
