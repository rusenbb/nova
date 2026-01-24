/**
 * List component definitions.
 * These types mirror the Rust definitions in src/extensions/components/list.rs
 */

import type { AccessoryType, IconType } from "./common.js";
import type { ActionPanel } from "./action.js";

/**
 * Filtering behavior for the list.
 */
export type ListFiltering = "default" | "none" | "custom";

/**
 * List component props - displays a searchable list of items.
 */
export interface ListProps {
  /** Whether the list is loading data */
  isLoading?: boolean;
  /** Placeholder text for the search bar */
  searchBarPlaceholder?: string;
  /** Filtering behavior (default: "default") */
  filtering?: ListFiltering;
  /** Callback ID for search text changes */
  onSearchChange?: string;
  /** Callback ID for selection changes */
  onSelectionChange?: string;
  /** Child items and sections */
  children?: ListChildElement[];
}

/**
 * A single item in a list.
 */
export interface ListItemProps {
  /** Unique identifier (required) */
  id: string;
  /** Primary text (required) */
  title: string;
  /** Secondary text */
  subtitle?: string;
  /** Icon displayed on the left */
  icon?: IconType;
  /** Accessories displayed on the right */
  accessories?: AccessoryType[];
  /** Additional search keywords */
  keywords?: string[];
  /** Actions available for this item */
  actions?: ActionPanel;
}

/**
 * A section that groups list items.
 */
export interface ListSectionProps {
  /** Section title */
  title?: string;
  /** Section subtitle */
  subtitle?: string;
  /** Items in this section */
  children?: ListItemElement[];
}

/**
 * Serialized ListItem (with type discriminator).
 */
export interface ListItemData {
  type: "List.Item";
  id: string;
  title: string;
  subtitle?: string;
  icon?: IconType;
  accessories?: AccessoryType[];
  keywords?: string[];
  actions?: ActionPanel;
}

/**
 * Serialized ListSection (with type discriminator).
 */
export interface ListSectionData {
  type: "List.Section";
  title?: string;
  subtitle?: string;
  children: Omit<ListItemData, "type">[];
}

/**
 * Union type for list children (serialized form).
 */
export type ListChildData = ListItemData | ListSectionData;

/**
 * Serialized List component (with type discriminator).
 */
export interface ListData {
  type: "List";
  isLoading?: boolean;
  searchBarPlaceholder?: string;
  filtering?: ListFiltering;
  onSearchChange?: string;
  onSelectionChange?: string;
  children: ListChildData[];
}

// JSX Element types (used in component definitions)
export type ListItemElement = { $$type: "List.Item"; props: ListItemProps };
export type ListSectionElement = { $$type: "List.Section"; props: ListSectionProps; children: ListItemElement[] };
export type ListChildElement = ListItemElement | ListSectionElement;
export type ListElement = { $$type: "List"; props: ListProps; children: ListChildElement[] };
