/**
 * Root component type definitions.
 * These types mirror the Rust definitions in src/extensions/components/mod.rs
 */

import type { ListData, ListElement } from "./list.js";
import type { DetailData, DetailElement } from "./detail.js";
import type { FormData, FormElement } from "./form.js";

/**
 * Serialized component data (matches Rust serde JSON format).
 * This is what gets sent to Nova.render().
 */
export type ComponentData = ListData | DetailData | FormData;

/**
 * Internal JSX element representation.
 */
export type NovaElement = ListElement | DetailElement | FormElement;

/**
 * Any renderable Nova element or fragment.
 */
export type NovaNode =
  | NovaElement
  | string
  | number
  | boolean
  | null
  | undefined
  | NovaNode[];
