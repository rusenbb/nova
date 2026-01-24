/**
 * Common types used across Nova components.
 * These types mirror the Rust definitions in src/extensions/components/common.rs
 */

/**
 * Icon reference - can be a system icon, URL, asset, emoji, or text.
 * Discriminated union with "type" tag.
 */
export type IconType =
  | { type: "system"; name: string }
  | { type: "url"; url: string }
  | { type: "asset"; name: string }
  | { type: "emoji"; emoji: string }
  | { type: "text"; text: string; color?: string };

/**
 * Helper object to create Icon values.
 * Use Icon.system("star.fill") instead of { type: "system", name: "star.fill" }
 */
export const Icon = {
  system: (name: string): IconType => ({ type: "system", name }),
  url: (url: string): IconType => ({ type: "url", url }),
  asset: (name: string): IconType => ({ type: "asset", name }),
  emoji: (emoji: string): IconType => ({ type: "emoji", emoji }),
  text: (text: string, color?: string): IconType => ({ type: "text", text, color }),
} as const;

/**
 * Accessory displayed on the right side of list items.
 * Discriminated union with "type" tag.
 */
export type AccessoryType =
  | { type: "text"; text: string }
  | { type: "icon"; icon: IconType; text?: string }
  | { type: "tag"; value: string; color?: string }
  | { type: "date"; date: string; format?: DateFormat };

/**
 * Helper object to create Accessory values.
 * Use Accessory.tag("TypeScript", "#3178c6") instead of { type: "tag", value: "TypeScript", color: "#3178c6" }
 */
export const Accessory = {
  text: (text: string): AccessoryType => ({ type: "text", text }),
  icon: (icon: IconType, text?: string): AccessoryType => ({ type: "icon", icon, text }),
  tag: (value: string, color?: string): AccessoryType => ({ type: "tag", value, color }),
  date: (date: string | Date, format?: DateFormat): AccessoryType => ({
    type: "date",
    date: typeof date === "string" ? date : date.toISOString(),
    format,
  }),
} as const;

/**
 * Date display format.
 */
export type DateFormat = "relative" | "absolute" | "time";

/**
 * Keyboard shortcut definition.
 */
export interface Shortcut {
  /** Modifier keys */
  modifiers: KeyModifier[];
  /** The key to press (e.g., "o", "enter", "backspace") */
  key: string;
}

/**
 * Keyboard modifier keys.
 */
export type KeyModifier = "cmd" | "ctrl" | "alt" | "shift";

/**
 * Helper to create keyboard shortcuts.
 */
export function shortcut(modifiers: KeyModifier[], key: string): Shortcut {
  return { modifiers, key };
}
