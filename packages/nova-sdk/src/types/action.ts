/**
 * Action and ActionPanel component definitions.
 * These types mirror the Rust definitions in src/extensions/components/action.rs
 */

import type { IconType, Shortcut } from "./common.js";

/**
 * Visual style for an action.
 */
export type ActionStyle = "default" | "destructive";

/**
 * A single action that can be triggered by the user.
 */
export interface Action {
  /** Unique identifier for the action */
  id: string;
  /** Display title */
  title: string;
  /** Optional icon */
  icon?: IconType;
  /** Optional keyboard shortcut */
  shortcut?: Shortcut;
  /** Visual style (default: "default") */
  style?: ActionStyle;
  /** Callback ID to invoke when action is triggered */
  onAction?: string;
}

/**
 * Container for actions associated with a component.
 */
export interface ActionPanel {
  /** Optional title for the action panel section */
  title?: string;
  /** List of actions */
  children: Action[];
}
