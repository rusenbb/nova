/**
 * @aspect/nova - TypeScript SDK for Nova Launcher Extensions
 *
 * This package provides type definitions, JSX runtime, and utilities
 * for building Nova extensions in TypeScript.
 *
 * @example
 * ```tsx
 * import { List, Icon, registerCommand, render, useState } from "@aspect/nova";
 *
 * registerCommand("my-command", () => {
 *   render(<MyComponent />);
 * });
 *
 * function MyComponent() {
 *   const [query, setQuery] = useState("");
 *
 *   return (
 *     <List searchBarPlaceholder="Search...">
 *       <List.Item
 *         id="1"
 *         title="Hello World"
 *         icon={Icon.emoji("👋")}
 *       />
 *     </List>
 *   );
 * }
 * ```
 *
 * @packageDocumentation
 */

// ─────────────────────────────────────────────────────────────────────────────
// Type Exports
// ─────────────────────────────────────────────────────────────────────────────

// Re-export all types
export type {
  // Common types
  IconType,
  AccessoryType,
  DateFormat,
  Shortcut,
  KeyModifier,

  // Action types
  Action,
  ActionPanel,
  ActionStyle,

  // List types
  ListProps,
  ListItemProps,
  ListSectionProps,
  ListFiltering,
  ListData,
  ListItemData,
  ListSectionData,
  ListChildData,

  // Detail types
  DetailProps,
  DetailMetadataProps,
  MetadataItemProps,
  MetadataLink,
  DetailData,
  DetailMetadataData,
  MetadataItemData,

  // Form types
  FormProps,
  FormTextFieldProps,
  FormDropdownProps,
  FormCheckboxProps,
  FormDatePickerProps,
  FieldValidation,
  DropdownOption,
  TextFieldType,
  FormData,
  FormTextFieldData,
  FormDropdownData,
  FormCheckboxData,
  FormDatePickerData,
  FormFieldData,

  // Component types
  ComponentData,
  NovaElement,
  NovaNode,

  // API types
  NovaAPI,
  ClipboardAPI,
  StorageAPI,
  PreferencesAPI,
  SystemAPI,
  NavigationAPI,
  FetchOptions,
  FetchResponse,
  FetchMethod,
  CommandHandler,
  CommandProps,
} from "./types/index.js";

// ─────────────────────────────────────────────────────────────────────────────
// Value Exports (Icon/Accessory factories)
// ─────────────────────────────────────────────────────────────────────────────

export { Icon, Accessory, shortcut } from "./types/index.js";

// ─────────────────────────────────────────────────────────────────────────────
// React Components (from reconciler)
// ─────────────────────────────────────────────────────────────────────────────

export { List, Detail, Form } from "./reconciler/index.js";

// ─────────────────────────────────────────────────────────────────────────────
// Factory Functions (for programmatic use without JSX)
// ─────────────────────────────────────────────────────────────────────────────

export {
  List as ListFactory,
  Detail as DetailFactory,
  Form as FormFactory,
  createAction,
  createActionPanel,
} from "./components.js";

// ─────────────────────────────────────────────────────────────────────────────
// React Hooks (from React)
// ─────────────────────────────────────────────────────────────────────────────

export {
  useState,
  useEffect,
  useMemo,
  useCallback,
  useRef,
  useReducer,
  useId,
  useContext,
  useLayoutEffect,
  useImperativeHandle,
  useDebugValue,
  useDeferredValue,
  useTransition,
  useSyncExternalStore,
  useInsertionEffect,
} from "react";

// Re-export common React types
export type {
  FC,
  ReactNode,
  ReactElement,
  Dispatch,
  SetStateAction,
  MutableRefObject,
  RefObject,
  Reducer,
  ReducerState,
  ReducerAction,
  DependencyList,
  EffectCallback,
} from "react";

// ─────────────────────────────────────────────────────────────────────────────
// Render System (from reconciler)
// ─────────────────────────────────────────────────────────────────────────────

export { render, unmount } from "./reconciler/index.js";

// ─────────────────────────────────────────────────────────────────────────────
// Navigation
// ─────────────────────────────────────────────────────────────────────────────

export {
  useNavigation,
  registerCallback,
  getCallback,
  clearCallback,
} from "./navigation.js";

export type { UseNavigationReturn } from "./navigation.js";

// ─────────────────────────────────────────────────────────────────────────────
// IPC Bridge (Direct API Access)
// ─────────────────────────────────────────────────────────────────────────────

export {
  // Clipboard
  clipboardCopy,
  clipboardRead,

  // Storage
  storageGet,
  storageSet,
  storageRemove,
  storageKeys,

  // Preferences
  getPreference,
  getAllPreferences,

  // Fetch
  fetch,
  fetchJson,
  postJson,

  // System
  openUrl,
  openPath,
  showNotification,
  closeWindow,

  // Render (legacy - prefer using render() from reconciler)
  renderComponent,

  // Navigation
  navigationPush,
  navigationPop,
  navigationDepth,

  // Commands
  registerCommand,
} from "./ipc.js";

// ─────────────────────────────────────────────────────────────────────────────
// JSX Runtime (re-exported for convenience)
// ─────────────────────────────────────────────────────────────────────────────

export { jsx, jsxs, Fragment, jsxDEV } from "./jsx-runtime.js";
