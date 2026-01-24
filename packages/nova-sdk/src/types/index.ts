/**
 * Type definitions for Nova SDK.
 * Re-exports all component and common types.
 */

// Common types
export type {
  IconType,
  AccessoryType,
  DateFormat,
  Shortcut,
  KeyModifier,
} from "./common.js";

export { Icon, Accessory, shortcut } from "./common.js";

// Action types
export type { Action, ActionPanel, ActionStyle } from "./action.js";

// List types
export type {
  ListProps,
  ListItemProps,
  ListSectionProps,
  ListFiltering,
  ListData,
  ListItemData,
  ListSectionData,
  ListChildData,
  ListElement,
  ListItemElement,
  ListSectionElement,
  ListChildElement,
} from "./list.js";

// Detail types
export type {
  DetailProps,
  DetailMetadataProps,
  MetadataItemProps,
  MetadataLink,
  DetailData,
  DetailMetadataData,
  MetadataItemData,
  DetailElement,
  DetailMetadataElement,
  MetadataItemElement,
} from "./detail.js";

// Form types
export type {
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
  FormElement,
  FormTextFieldElement,
  FormDropdownElement,
  FormCheckboxElement,
  FormDatePickerElement,
  FormFieldElement,
} from "./form.js";

// Component types
export type { ComponentData, NovaElement, NovaNode } from "./component.js";

// API types
export type {
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
} from "./api.js";
