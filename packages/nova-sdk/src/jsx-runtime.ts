/**
 * Nova JSX Runtime
 *
 * Implements a React-compatible JSX runtime for Nova components.
 * This enables the automatic JSX transform (jsx/jsxs/Fragment).
 *
 * Usage in tsconfig.json:
 *   "jsx": "react-jsx",
 *   "jsxImportSource": "@aspect/nova"
 */

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
  ComponentData,
  ListChildData,
  FormFieldData,
  MetadataItemData,
} from "./types/index.js";

// ─────────────────────────────────────────────────────────────────────────────
// Internal element representation
// ─────────────────────────────────────────────────────────────────────────────

const NOVA_ELEMENT_TYPE = Symbol.for("nova.element");

/**
 * Internal Nova element structure.
 */
export interface NovaElement<P = unknown> {
  $$typeof: typeof NOVA_ELEMENT_TYPE;
  type: string | NovaComponent<P>;
  props: P;
  key: string | null;
}

/**
 * A Nova component function.
 */
export type NovaComponent<P = unknown> = (props: P) => NovaElement | null;

/**
 * Check if a value is a Nova element.
 */
export function isNovaElement(value: unknown): value is NovaElement {
  return (
    typeof value === "object" &&
    value !== null &&
    (value as NovaElement).$$typeof === NOVA_ELEMENT_TYPE
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// JSX Runtime Functions
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Creates a Nova element (single child case).
 * Used by the automatic JSX transform.
 */
export function jsx<P extends Record<string, unknown>>(
  type: string | NovaComponent<P>,
  props: P & { children?: unknown },
  key?: string
): NovaElement<P> {
  return {
    $$typeof: NOVA_ELEMENT_TYPE,
    type,
    props: props as P,
    key: key ?? null,
  };
}

/**
 * Creates a Nova element (multiple children case).
 * Used by the automatic JSX transform.
 */
export function jsxs<P extends Record<string, unknown>>(
  type: string | NovaComponent<P>,
  props: P & { children?: unknown[] },
  key?: string
): NovaElement<P> {
  return jsx(type, props, key);
}

/**
 * Fragment - groups children without a wrapper element.
 */
export const Fragment = Symbol.for("nova.fragment");

/**
 * Creates a Nova element (development mode).
 * Same as jsx but could include additional debug info.
 */
export const jsxDEV = jsx;

// ─────────────────────────────────────────────────────────────────────────────
// Element Serialization
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Flatten children, removing nulls/undefined/booleans and flattening arrays.
 */
function flattenChildren(children: unknown): NovaElement[] {
  if (children == null || typeof children === "boolean") {
    return [];
  }

  if (Array.isArray(children)) {
    return children.flatMap(flattenChildren);
  }

  if (isNovaElement(children)) {
    return [children];
  }

  // Strings and numbers are not supported as direct children in Nova
  return [];
}

/**
 * Serialize a Nova element tree to the JSON format expected by Rust.
 */
export function serializeElement(element: NovaElement): ComponentData {
  const { type, props } = element;

  // If type is a function component, render it first
  if (typeof type === "function") {
    const rendered = type(props);
    if (rendered === null) {
      throw new Error("Component returned null - Nova requires a component tree");
    }
    return serializeElement(rendered);
  }

  // Handle built-in component types
  switch (type) {
    case "List":
      return serializeList(props as ListProps & { children?: unknown });

    case "Detail":
      return serializeDetail(props as DetailProps);

    case "Form":
      return serializeForm(props as FormProps & { children?: unknown });

    default:
      throw new Error(`Unknown component type: ${type}`);
  }
}

/**
 * Serialize a List component.
 */
function serializeList(props: ListProps & { children?: unknown }): ComponentData {
  const children = flattenChildren(props.children);
  const serializedChildren: ListChildData[] = [];

  for (const child of children) {
    if (typeof child.type !== "string") {
      throw new Error("List children must be List.Item or List.Section");
    }

    switch (child.type) {
      case "List.Item": {
        const itemProps = child.props as ListItemProps;
        serializedChildren.push({
          type: "List.Item",
          id: itemProps.id,
          title: itemProps.title,
          subtitle: itemProps.subtitle,
          icon: itemProps.icon,
          accessories: itemProps.accessories,
          keywords: itemProps.keywords,
          actions: itemProps.actions,
        });
        break;
      }

      case "List.Section": {
        const sectionProps = child.props as ListSectionProps & { children?: unknown };
        const sectionChildren = flattenChildren(sectionProps.children);

        serializedChildren.push({
          type: "List.Section",
          title: sectionProps.title,
          subtitle: sectionProps.subtitle,
          children: sectionChildren.map((item) => {
            const itemProps = item.props as ListItemProps;
            return {
              id: itemProps.id,
              title: itemProps.title,
              subtitle: itemProps.subtitle,
              icon: itemProps.icon,
              accessories: itemProps.accessories,
              keywords: itemProps.keywords,
              actions: itemProps.actions,
            };
          }),
        });
        break;
      }

      default:
        throw new Error(`Invalid List child type: ${child.type}`);
    }
  }

  return {
    type: "List",
    isLoading: props.isLoading,
    searchBarPlaceholder: props.searchBarPlaceholder,
    filtering: props.filtering,
    onSearchChange: props.onSearchChange,
    onSelectionChange: props.onSelectionChange,
    children: serializedChildren,
  };
}

/**
 * Serialize a Detail component.
 */
function serializeDetail(props: DetailProps): ComponentData {
  return {
    type: "Detail",
    markdown: props.markdown,
    isLoading: props.isLoading,
    actions: props.actions,
    metadata: props.metadata,
  };
}

/**
 * Serialize a Form component.
 */
function serializeForm(props: FormProps & { children?: unknown }): ComponentData {
  const children = flattenChildren(props.children);
  const serializedChildren: FormFieldData[] = [];

  for (const child of children) {
    if (typeof child.type !== "string") {
      throw new Error("Form children must be form field components");
    }

    switch (child.type) {
      case "Form.TextField": {
        const fieldProps = child.props as FormTextFieldProps;
        serializedChildren.push({
          type: "Form.TextField",
          id: fieldProps.id,
          title: fieldProps.title,
          placeholder: fieldProps.placeholder,
          defaultValue: fieldProps.defaultValue,
          fieldType: fieldProps.fieldType,
          validation: fieldProps.validation,
        });
        break;
      }

      case "Form.Dropdown": {
        const fieldProps = child.props as FormDropdownProps;
        serializedChildren.push({
          type: "Form.Dropdown",
          id: fieldProps.id,
          title: fieldProps.title,
          defaultValue: fieldProps.defaultValue,
          options: fieldProps.options,
        });
        break;
      }

      case "Form.Checkbox": {
        const fieldProps = child.props as FormCheckboxProps;
        serializedChildren.push({
          type: "Form.Checkbox",
          id: fieldProps.id,
          title: fieldProps.title,
          label: fieldProps.label,
          defaultValue: fieldProps.defaultValue,
        });
        break;
      }

      case "Form.DatePicker": {
        const fieldProps = child.props as FormDatePickerProps;
        serializedChildren.push({
          type: "Form.DatePicker",
          id: fieldProps.id,
          title: fieldProps.title,
          defaultValue: fieldProps.defaultValue,
          includeTime: fieldProps.includeTime,
        });
        break;
      }

      default:
        throw new Error(`Invalid Form field type: ${child.type}`);
    }
  }

  return {
    type: "Form",
    isLoading: props.isLoading,
    onSubmit: props.onSubmit,
    onChange: props.onChange,
    children: serializedChildren,
  };
}

// ─────────────────────────────────────────────────────────────────────────────
// JSX Namespace for TypeScript
// ─────────────────────────────────────────────────────────────────────────────

export namespace JSX {
  export interface Element extends NovaElement {}

  export interface ElementChildrenAttribute {
    children: {};
  }

  export interface IntrinsicElements {
    // List components
    List: ListProps & { children?: unknown; key?: string };
    "List.Item": ListItemProps & { key?: string };
    "List.Section": ListSectionProps & { children?: unknown; key?: string };

    // Detail components
    Detail: DetailProps & { key?: string };
    "Detail.Metadata": DetailMetadataProps & { children?: unknown; key?: string };
    "Detail.Metadata.Item": MetadataItemProps & { key?: string };

    // Form components
    Form: FormProps & { children?: unknown; key?: string };
    "Form.TextField": FormTextFieldProps & { key?: string };
    "Form.Dropdown": FormDropdownProps & { key?: string };
    "Form.Checkbox": FormCheckboxProps & { key?: string };
    "Form.DatePicker": FormDatePickerProps & { key?: string };
  }
}
