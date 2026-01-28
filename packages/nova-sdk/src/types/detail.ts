/**
 * Detail component definitions.
 * These types mirror the Rust definitions in src/extensions/components/detail.rs
 */

import type { IconType } from "./common.js";
import type { ActionPanel } from "./action.js";

/**
 * A clickable link in metadata.
 */
export interface MetadataLink {
  /** Display text */
  text: string;
  /** URL to open */
  url: string;
}

/**
 * A single metadata item (key-value pair).
 */
export interface MetadataItemProps {
  /** Label for this metadata */
  title: string;
  /** Text value */
  text?: string;
  /** Icon to display */
  icon?: IconType;
  /** Link to open */
  link?: MetadataLink;
}

/**
 * Metadata sidebar props for Detail component.
 */
export interface DetailMetadataProps {
  /** Metadata items */
  children?: MetadataItemElement[];
}

/**
 * Detail component props - displays markdown content with metadata.
 */
export interface DetailProps {
  /** Markdown content to render */
  markdown?: string;
  /** Whether the detail is loading */
  isLoading?: boolean;
  /** Actions available for this view */
  actions?: ActionPanel;
  /** Metadata sidebar */
  metadata?: DetailMetadataData;
}

/**
 * Serialized MetadataItem.
 */
export interface MetadataItemData {
  title: string;
  text?: string;
  icon?: IconType;
  link?: MetadataLink;
}

/**
 * Serialized DetailMetadata.
 */
export interface DetailMetadataData {
  children: MetadataItemData[];
}

/**
 * Serialized Detail component (with type discriminator).
 */
export interface DetailData {
  type: "Detail";
  markdown?: string;
  isLoading?: boolean;
  actions?: ActionPanel;
  metadata?: DetailMetadataData;
}

// JSX Element types
export type MetadataItemElement = { $$type: "Detail.Metadata.Item"; props: MetadataItemProps };
export type DetailMetadataElement = { $$type: "Detail.Metadata"; props: DetailMetadataProps; children: MetadataItemElement[] };
export type DetailElement = { $$type: "Detail"; props: DetailProps };
