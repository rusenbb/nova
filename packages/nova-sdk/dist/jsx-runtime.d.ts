/**
 * Common types used across Nova components.
 * These types mirror the Rust definitions in src/extensions/components/common.rs
 */
/**
 * Icon reference - can be a system icon, URL, asset, emoji, or text.
 * Discriminated union with "type" tag.
 */
type IconType = {
    type: "system";
    name: string;
} | {
    type: "url";
    url: string;
} | {
    type: "asset";
    name: string;
} | {
    type: "emoji";
    emoji: string;
} | {
    type: "text";
    text: string;
    color?: string;
};
/**
 * Accessory displayed on the right side of list items.
 * Discriminated union with "type" tag.
 */
type AccessoryType = {
    type: "text";
    text: string;
} | {
    type: "icon";
    icon: IconType;
    text?: string;
} | {
    type: "tag";
    value: string;
    color?: string;
} | {
    type: "date";
    date: string;
    format?: DateFormat;
};
/**
 * Date display format.
 */
type DateFormat = "relative" | "absolute" | "time";
/**
 * Keyboard shortcut definition.
 */
interface Shortcut {
    /** Modifier keys */
    modifiers: KeyModifier[];
    /** The key to press (e.g., "o", "enter", "backspace") */
    key: string;
}
/**
 * Keyboard modifier keys.
 */
type KeyModifier = "cmd" | "ctrl" | "alt" | "shift";

/**
 * Action and ActionPanel component definitions.
 * These types mirror the Rust definitions in src/extensions/components/action.rs
 */

/**
 * Visual style for an action.
 */
type ActionStyle = "default" | "destructive";
/**
 * A single action that can be triggered by the user.
 */
interface Action {
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
interface ActionPanel {
    /** Optional title for the action panel section */
    title?: string;
    /** List of actions */
    children: Action[];
}

/**
 * List component definitions.
 * These types mirror the Rust definitions in src/extensions/components/list.rs
 */

/**
 * Filtering behavior for the list.
 */
type ListFiltering = "default" | "none" | "custom";
/**
 * List component props - displays a searchable list of items.
 */
interface ListProps {
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
interface ListItemProps {
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
interface ListSectionProps {
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
interface ListItemData {
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
interface ListSectionData {
    type: "List.Section";
    title?: string;
    subtitle?: string;
    children: Omit<ListItemData, "type">[];
}
/**
 * Union type for list children (serialized form).
 */
type ListChildData = ListItemData | ListSectionData;
/**
 * Serialized List component (with type discriminator).
 */
interface ListData {
    type: "List";
    isLoading?: boolean;
    searchBarPlaceholder?: string;
    filtering?: ListFiltering;
    onSearchChange?: string;
    onSelectionChange?: string;
    children: ListChildData[];
}
type ListItemElement = {
    $$type: "List.Item";
    props: ListItemProps;
};
type ListSectionElement = {
    $$type: "List.Section";
    props: ListSectionProps;
    children: ListItemElement[];
};
type ListChildElement = ListItemElement | ListSectionElement;

/**
 * Detail component definitions.
 * These types mirror the Rust definitions in src/extensions/components/detail.rs
 */

/**
 * A clickable link in metadata.
 */
interface MetadataLink {
    /** Display text */
    text: string;
    /** URL to open */
    url: string;
}
/**
 * A single metadata item (key-value pair).
 */
interface MetadataItemProps {
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
interface DetailMetadataProps {
    /** Metadata items */
    children?: MetadataItemElement[];
}
/**
 * Detail component props - displays markdown content with metadata.
 */
interface DetailProps {
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
interface MetadataItemData {
    title: string;
    text?: string;
    icon?: IconType;
    link?: MetadataLink;
}
/**
 * Serialized DetailMetadata.
 */
interface DetailMetadataData {
    children: MetadataItemData[];
}
/**
 * Serialized Detail component (with type discriminator).
 */
interface DetailData {
    type: "Detail";
    markdown?: string;
    isLoading?: boolean;
    actions?: ActionPanel;
    metadata?: DetailMetadataData;
}
type MetadataItemElement = {
    $$type: "Detail.Metadata.Item";
    props: MetadataItemProps;
};

/**
 * Form component definitions.
 * These types mirror the Rust definitions in src/extensions/components/form.rs
 */

/**
 * Text field input type.
 */
type TextFieldType = "text" | "password" | "number";
/**
 * Validation rules for form fields.
 */
interface FieldValidation {
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
interface DropdownOption {
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
interface FormTextFieldProps {
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
interface FormDropdownProps {
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
interface FormCheckboxProps {
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
interface FormDatePickerProps {
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
interface FormProps {
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
interface FormTextFieldData {
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
interface FormDropdownData {
    type: "Form.Dropdown";
    id: string;
    title: string;
    defaultValue?: string;
    options: DropdownOption[];
}
/**
 * Serialized FormCheckbox (with type discriminator).
 */
interface FormCheckboxData {
    type: "Form.Checkbox";
    id: string;
    title: string;
    label?: string;
    defaultValue?: boolean;
}
/**
 * Serialized FormDatePicker (with type discriminator).
 */
interface FormDatePickerData {
    type: "Form.DatePicker";
    id: string;
    title: string;
    defaultValue?: string;
    includeTime?: boolean;
}
/**
 * Union type for form fields (serialized form).
 */
type FormFieldData = FormTextFieldData | FormDropdownData | FormCheckboxData | FormDatePickerData;
/**
 * Serialized Form component (with type discriminator).
 */
interface FormData {
    type: "Form";
    isLoading?: boolean;
    onSubmit?: string;
    onChange?: string;
    children: FormFieldData[];
}
type FormTextFieldElement = {
    $$type: "Form.TextField";
    props: FormTextFieldProps;
};
type FormDropdownElement = {
    $$type: "Form.Dropdown";
    props: FormDropdownProps;
};
type FormCheckboxElement = {
    $$type: "Form.Checkbox";
    props: FormCheckboxProps;
};
type FormDatePickerElement = {
    $$type: "Form.DatePicker";
    props: FormDatePickerProps;
};
type FormFieldElement = FormTextFieldElement | FormDropdownElement | FormCheckboxElement | FormDatePickerElement;

/**
 * Root component type definitions.
 * These types mirror the Rust definitions in src/extensions/components/mod.rs
 */

/**
 * Serialized component data (matches Rust serde JSON format).
 * This is what gets sent to Nova.render().
 */
type ComponentData = ListData | DetailData | FormData;

/**
 * Nova API type definitions.
 * These types describe the global Nova API available in extensions.
 */

/**
 * HTTP fetch method.
 */
type FetchMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS";
/**
 * Options for Nova.fetch().
 */
interface FetchOptions {
    /** HTTP method (default: "GET") */
    method?: FetchMethod;
    /** Request headers */
    headers?: Record<string, string>;
    /** Request body */
    body?: string;
}
/**
 * Response from Nova.fetch().
 */
interface FetchResponse {
    /** HTTP status code */
    status: number;
    /** Response headers */
    headers: Record<string, string>;
    /** Response body */
    body: string;
}
/**
 * Clipboard API.
 */
interface ClipboardAPI {
    /** Copy text to the system clipboard */
    copy(text: string): void;
    /** Read text from the system clipboard */
    read(): string;
}
/**
 * Storage API for persistent key-value storage.
 */
interface StorageAPI {
    /** Get a value from storage */
    get<T = unknown>(key: string): T | undefined;
    /** Set a value in storage */
    set<T = unknown>(key: string, value: T): void;
    /** Remove a key from storage */
    remove(key: string): void;
    /** Get all keys in storage */
    keys(): string[];
}
/**
 * Preferences API for user-configured settings.
 */
interface PreferencesAPI {
    /** Get a preference value by key */
    get<T = unknown>(key: string): T | undefined;
    /** Get all preferences as an object */
    all(): Record<string, unknown>;
}
/**
 * System API for OS interactions.
 */
interface SystemAPI {
    /** Open a URL in the default browser */
    openUrl(url: string): void;
    /** Open a file or directory in the default application */
    openPath(path: string): void;
    /** Show a system notification */
    notify(title: string, body?: string): void;
    /** Close the Nova window */
    closeWindow(): void;
}
/**
 * Navigation API for multi-view extensions.
 */
interface NavigationAPI {
    /** Push a new view onto the navigation stack */
    push(component: ComponentData): void;
    /** Pop the top view from the navigation stack */
    pop(): boolean;
    /** Get the current navigation stack depth */
    depth(): number;
}
/**
 * Command handler function type.
 */
type CommandHandler = (props: CommandProps) => void | Promise<void>;
/**
 * Props passed to a command handler.
 */
interface CommandProps {
    /** Optional argument passed to the command */
    argument?: string;
    /** Additional context */
    [key: string]: unknown;
}
/**
 * The global Nova API interface.
 */
interface NovaAPI {
    /** Clipboard operations (requires "clipboard" permission) */
    clipboard: ClipboardAPI;
    /** Persistent key-value storage */
    storage: StorageAPI;
    /** User-configured preferences */
    preferences: PreferencesAPI;
    /** System operations */
    system: SystemAPI;
    /** Navigation stack for multi-view extensions */
    navigation: NavigationAPI;
    /**
     * Perform an HTTP fetch request.
     * Requires the target domain to be listed in permissions.network.
     */
    fetch(url: string, options?: FetchOptions): Promise<FetchResponse>;
    /**
     * Render a component tree to the Nova UI.
     */
    render(component: ComponentData): void;
    /**
     * Register a command handler.
     */
    registerCommand(name: string, handler: CommandHandler): void;
}
/**
 * Declare the global Nova API.
 */
declare global {
    const Nova: NovaAPI;
}

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

declare const NOVA_ELEMENT_TYPE: unique symbol;
/**
 * Internal Nova element structure.
 */
interface NovaElement<P = unknown> {
    $$typeof: typeof NOVA_ELEMENT_TYPE;
    type: string | NovaComponent<P>;
    props: P;
    key: string | null;
}
/**
 * A Nova component function.
 */
type NovaComponent<P = unknown> = (props: P) => NovaElement | null;
/**
 * Check if a value is a Nova element.
 */
declare function isNovaElement(value: unknown): value is NovaElement;
/**
 * Creates a Nova element (single child case).
 * Used by the automatic JSX transform.
 */
declare function jsx<P extends Record<string, unknown>>(type: string | NovaComponent<P>, props: P & {
    children?: unknown;
}, key?: string): NovaElement<P>;
/**
 * Creates a Nova element (multiple children case).
 * Used by the automatic JSX transform.
 */
declare function jsxs<P extends Record<string, unknown>>(type: string | NovaComponent<P>, props: P & {
    children?: unknown[];
}, key?: string): NovaElement<P>;
/**
 * Fragment - groups children without a wrapper element.
 */
declare const Fragment: unique symbol;
/**
 * Creates a Nova element (development mode).
 * Same as jsx but could include additional debug info.
 */
declare const jsxDEV: typeof jsx;
/**
 * Serialize a Nova element tree to the JSON format expected by Rust.
 */
declare function serializeElement(element: NovaElement): ComponentData;
declare namespace JSX {
    interface Element extends NovaElement {
    }
    interface ElementChildrenAttribute {
        children: {};
    }
    interface IntrinsicElements {
        List: ListProps & {
            children?: unknown;
            key?: string;
        };
        "List.Item": ListItemProps & {
            key?: string;
        };
        "List.Section": ListSectionProps & {
            children?: unknown;
            key?: string;
        };
        Detail: DetailProps & {
            key?: string;
        };
        "Detail.Metadata": DetailMetadataProps & {
            children?: unknown;
            key?: string;
        };
        "Detail.Metadata.Item": MetadataItemProps & {
            key?: string;
        };
        Form: FormProps & {
            children?: unknown;
            key?: string;
        };
        "Form.TextField": FormTextFieldProps & {
            key?: string;
        };
        "Form.Dropdown": FormDropdownProps & {
            key?: string;
        };
        "Form.Checkbox": FormCheckboxProps & {
            key?: string;
        };
        "Form.DatePicker": FormDatePickerProps & {
            key?: string;
        };
    }
}

export { Fragment, JSX, type NovaComponent, type NovaElement, isNovaElement, jsx, jsxDEV, jsxs, serializeElement };
