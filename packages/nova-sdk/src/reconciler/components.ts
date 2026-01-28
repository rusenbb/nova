/**
 * React Components for Nova
 *
 * These components render to Nova's native UI through the custom reconciler.
 * They create React elements with string types that the reconciler handles.
 */

import { createElement, type ReactNode, type FC } from "react";
import type {
  ListProps,
  ListItemProps,
  ListSectionProps,
  DetailProps,
  DetailMetadataProps,
  MetadataItemProps,
  FormProps,
  FormTextFieldProps,
  FormDropdownProps,
  FormCheckboxProps,
  FormDatePickerProps,
} from "../types/index.js";

// ─────────────────────────────────────────────────────────────────────────────
// List Components
// ─────────────────────────────────────────────────────────────────────────────

/**
 * List component props with React children.
 */
export type ListComponentProps = Omit<ListProps, "children"> & {
  children?: ReactNode;
};

/**
 * List.Item component props (no children needed).
 */
export type ListItemComponentProps = ListItemProps;

/**
 * List.Section component props with React children.
 */
export type ListSectionComponentProps = Omit<ListSectionProps, "children"> & {
  children?: ReactNode;
};

/**
 * A searchable list of items.
 */
const ListRoot: FC<ListComponentProps> = (props) => {
  return createElement("List", props);
};

/**
 * A single item in a list.
 */
const ListItem: FC<ListItemComponentProps> = (props) => {
  return createElement("List.Item", props);
};

/**
 * A section that groups list items.
 */
const ListSection: FC<ListSectionComponentProps> = (props) => {
  return createElement("List.Section", props);
};

/**
 * List component with Item and Section sub-components.
 */
export const List = Object.assign(ListRoot, {
  Item: ListItem,
  Section: ListSection,
});

// ─────────────────────────────────────────────────────────────────────────────
// Detail Components
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Detail component props (no React children for metadata).
 */
export type DetailComponentProps = DetailProps;

/**
 * Detail.Metadata component props with React children.
 */
export type DetailMetadataComponentProps = Omit<DetailMetadataProps, "children"> & {
  children?: ReactNode;
};

/**
 * Detail.Metadata.Item component props.
 */
export type MetadataItemComponentProps = MetadataItemProps;

/**
 * Displays markdown content with optional metadata sidebar.
 */
const DetailRoot: FC<DetailComponentProps> = (props) => {
  return createElement("Detail", props);
};

/**
 * Metadata sidebar for Detail component.
 */
const DetailMetadataComponent: FC<DetailMetadataComponentProps> = (props) => {
  return createElement("Detail.Metadata", props);
};

/**
 * A single metadata item (key-value pair).
 */
const MetadataItem: FC<MetadataItemComponentProps> = (props) => {
  return createElement("Detail.Metadata.Item", props);
};

/**
 * Detail.Metadata with Item sub-component.
 */
const DetailMetadata = Object.assign(DetailMetadataComponent, {
  Item: MetadataItem,
});

/**
 * Detail component with Metadata sub-component.
 */
export const Detail = Object.assign(DetailRoot, {
  Metadata: DetailMetadata,
});

// ─────────────────────────────────────────────────────────────────────────────
// Form Components
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Form component props with React children.
 */
export type FormComponentProps = Omit<FormProps, "children"> & {
  children?: ReactNode;
};

/**
 * Form field component props.
 */
export type FormTextFieldComponentProps = FormTextFieldProps;
export type FormDropdownComponentProps = FormDropdownProps;
export type FormCheckboxComponentProps = FormCheckboxProps;
export type FormDatePickerComponentProps = FormDatePickerProps;

/**
 * A form for collecting user input.
 */
const FormRoot: FC<FormComponentProps> = (props) => {
  return createElement("Form", props);
};

/**
 * Text input field.
 */
const FormTextField: FC<FormTextFieldComponentProps> = (props) => {
  return createElement("Form.TextField", props);
};

/**
 * Dropdown/select field.
 */
const FormDropdown: FC<FormDropdownComponentProps> = (props) => {
  return createElement("Form.Dropdown", props);
};

/**
 * Checkbox field.
 */
const FormCheckbox: FC<FormCheckboxComponentProps> = (props) => {
  return createElement("Form.Checkbox", props);
};

/**
 * Date picker field.
 */
const FormDatePicker: FC<FormDatePickerComponentProps> = (props) => {
  return createElement("Form.DatePicker", props);
};

/**
 * Form component with field sub-components.
 */
export const Form = Object.assign(FormRoot, {
  TextField: FormTextField,
  Dropdown: FormDropdown,
  Checkbox: FormCheckbox,
  DatePicker: FormDatePicker,
});
