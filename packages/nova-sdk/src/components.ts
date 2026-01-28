/**
 * Component Factory Functions
 *
 * These provide an alternative to JSX for creating Nova components.
 * Useful when JSX isn't available or for programmatic component creation.
 */

import type {
  ListProps,
  ListItemProps,
  ListSectionProps,
  DetailProps,
  MetadataItemProps,
  FormProps,
  FormTextFieldProps,
  FormDropdownProps,
  FormCheckboxProps,
  FormDatePickerProps,
  ListData,
  DetailData,
  FormData,
  ListChildData,
  ListItemData,
  FormFieldData,
  DetailMetadataData,
  MetadataItemData,
  Action,
  ActionPanel,
  ActionStyle,
  Shortcut,
  IconType,
} from "./types/index.js";

// ─────────────────────────────────────────────────────────────────────────────
// List Components
// ─────────────────────────────────────────────────────────────────────────────

interface ListFunction {
  (props: ListProps & { children?: ListChildData[] }): ListData;
  Item: (props: ListItemProps) => ListChildData;
  Section: (props: ListSectionProps & { items?: ListItemProps[] }) => ListChildData;
}

/**
 * Create a List component.
 */
export const List: ListFunction = Object.assign(
  function List(props: ListProps & { children?: ListChildData[] }): ListData {
    return {
      type: "List",
      isLoading: props.isLoading,
      searchBarPlaceholder: props.searchBarPlaceholder,
      filtering: props.filtering,
      onSearchChange: props.onSearchChange,
      onSelectionChange: props.onSelectionChange,
      children: props.children ?? [],
    };
  },
  {
    /**
     * Create a List.Item.
     */
    Item: function ListItem(props: ListItemProps): ListChildData {
      return {
        type: "List.Item" as const,
        id: props.id,
        title: props.title,
        subtitle: props.subtitle,
        icon: props.icon,
        accessories: props.accessories,
        keywords: props.keywords,
        actions: props.actions,
      };
    },
    /**
     * Create a List.Section.
     */
    Section: function ListSection(
      props: ListSectionProps & { items?: ListItemProps[] }
    ): ListChildData {
      const items = props.items ?? [];
      return {
        type: "List.Section" as const,
        title: props.title,
        subtitle: props.subtitle,
        children: items.map((item) => ({
          id: item.id,
          title: item.title,
          subtitle: item.subtitle,
          icon: item.icon,
          accessories: item.accessories,
          keywords: item.keywords,
          actions: item.actions,
        })),
      };
    },
  }
);

// ─────────────────────────────────────────────────────────────────────────────
// Detail Components
// ─────────────────────────────────────────────────────────────────────────────

interface DetailMetadataFunction {
  (props: { children?: MetadataItemProps[] }): DetailMetadataData;
  Item: (props: MetadataItemProps) => MetadataItemData;
}

interface DetailFunction {
  (props: DetailProps): DetailData;
  Metadata: DetailMetadataFunction;
}

const DetailMetadata: DetailMetadataFunction = Object.assign(
  function DetailMetadata(props: { children?: MetadataItemProps[] }): DetailMetadataData {
    return {
      children: (props.children ?? []).map((item): MetadataItemData => ({
        title: item.title,
        text: item.text,
        icon: item.icon,
        link: item.link,
      })),
    };
  },
  {
    /**
     * Create Detail.Metadata.Item.
     */
    Item: function MetadataItem(props: MetadataItemProps): MetadataItemData {
      return {
        title: props.title,
        text: props.text,
        icon: props.icon,
        link: props.link,
      };
    },
  }
);

/**
 * Create a Detail component.
 */
export const Detail: DetailFunction = Object.assign(
  function Detail(props: DetailProps): DetailData {
    return {
      type: "Detail",
      markdown: props.markdown,
      isLoading: props.isLoading,
      actions: props.actions,
      metadata: props.metadata,
    };
  },
  {
    Metadata: DetailMetadata,
  }
);

// ─────────────────────────────────────────────────────────────────────────────
// Form Components
// ─────────────────────────────────────────────────────────────────────────────

interface FormFunction {
  (props: FormProps & { children?: FormFieldData[] }): FormData;
  TextField: (props: FormTextFieldProps) => FormFieldData;
  Dropdown: (props: FormDropdownProps) => FormFieldData;
  Checkbox: (props: FormCheckboxProps) => FormFieldData;
  DatePicker: (props: FormDatePickerProps) => FormFieldData;
}

/**
 * Create a Form component.
 */
export const Form: FormFunction = Object.assign(
  function Form(props: FormProps & { children?: FormFieldData[] }): FormData {
    return {
      type: "Form",
      isLoading: props.isLoading,
      onSubmit: props.onSubmit,
      onChange: props.onChange,
      children: props.children ?? [],
    };
  },
  {
    /**
     * Create a Form.TextField.
     */
    TextField: function FormTextField(props: FormTextFieldProps): FormFieldData {
      return {
        type: "Form.TextField",
        id: props.id,
        title: props.title,
        placeholder: props.placeholder,
        defaultValue: props.defaultValue,
        fieldType: props.fieldType,
        validation: props.validation,
      };
    },
    /**
     * Create a Form.Dropdown.
     */
    Dropdown: function FormDropdown(props: FormDropdownProps): FormFieldData {
      return {
        type: "Form.Dropdown",
        id: props.id,
        title: props.title,
        defaultValue: props.defaultValue,
        options: props.options,
      };
    },
    /**
     * Create a Form.Checkbox.
     */
    Checkbox: function FormCheckbox(props: FormCheckboxProps): FormFieldData {
      return {
        type: "Form.Checkbox",
        id: props.id,
        title: props.title,
        label: props.label,
        defaultValue: props.defaultValue,
      };
    },
    /**
     * Create a Form.DatePicker.
     */
    DatePicker: function FormDatePicker(props: FormDatePickerProps): FormFieldData {
      return {
        type: "Form.DatePicker",
        id: props.id,
        title: props.title,
        defaultValue: props.defaultValue,
        includeTime: props.includeTime,
      };
    },
  }
);

// ─────────────────────────────────────────────────────────────────────────────
// Action Helpers
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Create an ActionPanel.
 */
export function createActionPanel(
  title: string | undefined,
  actions: Action[]
): ActionPanel {
  return { title, children: actions };
}

/**
 * Create an Action.
 */
export function createAction(props: {
  id: string;
  title: string;
  icon?: IconType;
  shortcut?: Shortcut;
  style?: ActionStyle;
  onAction?: string;
}): Action {
  return {
    id: props.id,
    title: props.title,
    icon: props.icon,
    shortcut: props.shortcut,
    style: props.style ?? "default",
    onAction: props.onAction,
  };
}
