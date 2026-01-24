/**
 * Form component definitions.
 * These types mirror the Rust definitions in src/extensions/components/form.rs
 */

import type { IconType } from "./common.js";

/**
 * Text field input type.
 */
export type TextFieldType = "text" | "password" | "number";

/**
 * Validation rules for form fields.
 */
export interface FieldValidation {
  /** Whether the field is required */
  required?: boolean;
  /** Regex pattern to match */
  pattern?: string;
  /** Minimum length */
  minLength?: number;
  /** Maximum length */
  maxLength?: number;
}

/**
 * An option in a dropdown.
 */
export interface DropdownOption {
  /** Value to submit */
  value: string;
  /** Display title */
  title: string;
  /** Optional icon */
  icon?: IconType;
}

/**
 * Text input field props.
 */
export interface FormTextFieldProps {
  /** Unique identifier */
  id: string;
  /** Field label */
  title: string;
  /** Placeholder text */
  placeholder?: string;
  /** Default value */
  defaultValue?: string;
  /** Input type (default: "text") */
  fieldType?: TextFieldType;
  /** Validation rules */
  validation?: FieldValidation;
}

/**
 * Dropdown/select field props.
 */
export interface FormDropdownProps {
  /** Unique identifier */
  id: string;
  /** Field label */
  title: string;
  /** Default selected value */
  defaultValue?: string;
  /** Available options */
  options: DropdownOption[];
}

/**
 * Checkbox field props.
 */
export interface FormCheckboxProps {
  /** Unique identifier */
  id: string;
  /** Field label */
  title: string;
  /** Additional label text */
  label?: string;
  /** Default checked state */
  defaultValue?: boolean;
}

/**
 * Date picker field props.
 */
export interface FormDatePickerProps {
  /** Unique identifier */
  id: string;
  /** Field label */
  title: string;
  /** Default date (ISO 8601) */
  defaultValue?: string;
  /** Whether to include time selection */
  includeTime?: boolean;
}

/**
 * Form component props - collects user input.
 */
export interface FormProps {
  /** Whether the form is loading/submitting */
  isLoading?: boolean;
  /** Callback ID for form submission */
  onSubmit?: string;
  /** Callback ID for value changes */
  onChange?: string;
  /** Form fields */
  children?: FormFieldElement[];
}

/**
 * Serialized FormTextField (with type discriminator).
 */
export interface FormTextFieldData {
  type: "Form.TextField";
  id: string;
  title: string;
  placeholder?: string;
  defaultValue?: string;
  fieldType?: TextFieldType;
  validation?: FieldValidation;
}

/**
 * Serialized FormDropdown (with type discriminator).
 */
export interface FormDropdownData {
  type: "Form.Dropdown";
  id: string;
  title: string;
  defaultValue?: string;
  options: DropdownOption[];
}

/**
 * Serialized FormCheckbox (with type discriminator).
 */
export interface FormCheckboxData {
  type: "Form.Checkbox";
  id: string;
  title: string;
  label?: string;
  defaultValue?: boolean;
}

/**
 * Serialized FormDatePicker (with type discriminator).
 */
export interface FormDatePickerData {
  type: "Form.DatePicker";
  id: string;
  title: string;
  defaultValue?: string;
  includeTime?: boolean;
}

/**
 * Union type for form fields (serialized form).
 */
export type FormFieldData =
  | FormTextFieldData
  | FormDropdownData
  | FormCheckboxData
  | FormDatePickerData;

/**
 * Serialized Form component (with type discriminator).
 */
export interface FormData {
  type: "Form";
  isLoading?: boolean;
  onSubmit?: string;
  onChange?: string;
  children: FormFieldData[];
}

// JSX Element types
export type FormTextFieldElement = { $$type: "Form.TextField"; props: FormTextFieldProps };
export type FormDropdownElement = { $$type: "Form.Dropdown"; props: FormDropdownProps };
export type FormCheckboxElement = { $$type: "Form.Checkbox"; props: FormCheckboxProps };
export type FormDatePickerElement = { $$type: "Form.DatePicker"; props: FormDatePickerProps };
export type FormFieldElement = FormTextFieldElement | FormDropdownElement | FormCheckboxElement | FormDatePickerElement;
export type FormElement = { $$type: "Form"; props: FormProps; children: FormFieldElement[] };
