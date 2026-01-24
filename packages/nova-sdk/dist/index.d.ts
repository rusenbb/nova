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
 * Helper object to create Icon values.
 * Use Icon.system("star.fill") instead of { type: "system", name: "star.fill" }
 */
declare const Icon: {
    readonly system: (name: string) => IconType;
    readonly url: (url: string) => IconType;
    readonly asset: (name: string) => IconType;
    readonly emoji: (emoji: string) => IconType;
    readonly text: (text: string, color?: string) => IconType;
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
 * Helper object to create Accessory values.
 * Use Accessory.tag("TypeScript", "#3178c6") instead of { type: "tag", value: "TypeScript", color: "#3178c6" }
 */
declare const Accessory: {
    readonly text: (text: string) => AccessoryType;
    readonly icon: (icon: IconType, text?: string) => AccessoryType;
    readonly tag: (value: string, color?: string) => AccessoryType;
    readonly date: (date: string | Date, format?: DateFormat) => AccessoryType;
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
 * Helper to create keyboard shortcuts.
 */
declare function shortcut(modifiers: KeyModifier[], key: string): Shortcut;

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
type ListElement = {
    $$type: "List";
    props: ListProps;
    children: ListChildElement[];
};

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
type DetailElement = {
    $$type: "Detail";
    props: DetailProps;
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
type FormElement = {
    $$type: "Form";
    props: FormProps;
    children: FormFieldElement[];
};

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
 * Internal JSX element representation.
 */
type NovaElement$1 = ListElement | DetailElement | FormElement;
/**
 * Any renderable Nova element or fragment.
 */
type NovaNode = NovaElement$1 | string | number | boolean | null | undefined | NovaNode[];

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
 * Component Factory Functions
 *
 * These provide an alternative to JSX for creating Nova components.
 * Useful when JSX isn't available or for programmatic component creation.
 */

interface ListFunction {
    (props: ListProps & {
        children?: ListChildData[];
    }): ListData;
    Item: (props: ListItemProps) => ListChildData;
    Section: (props: ListSectionProps & {
        items?: ListItemProps[];
    }) => ListChildData;
}
/**
 * Create a List component.
 */
declare const List: ListFunction;
interface DetailMetadataFunction {
    (props: {
        children?: MetadataItemProps[];
    }): DetailMetadataData;
    Item: (props: MetadataItemProps) => MetadataItemData;
}
interface DetailFunction {
    (props: DetailProps): DetailData;
    Metadata: DetailMetadataFunction;
}
/**
 * Create a Detail component.
 */
declare const Detail: DetailFunction;
interface FormFunction {
    (props: FormProps & {
        children?: FormFieldData[];
    }): FormData;
    TextField: (props: FormTextFieldProps) => FormFieldData;
    Dropdown: (props: FormDropdownProps) => FormFieldData;
    Checkbox: (props: FormCheckboxProps) => FormFieldData;
    DatePicker: (props: FormDatePickerProps) => FormFieldData;
}
/**
 * Create a Form component.
 */
declare const Form: FormFunction;
/**
 * Create an ActionPanel.
 */
declare function createActionPanel(title: string | undefined, actions: Action[]): ActionPanel;
/**
 * Create an Action.
 */
declare function createAction(props: {
    id: string;
    title: string;
    icon?: IconType;
    shortcut?: Shortcut;
    style?: ActionStyle;
    onAction?: string;
}): Action;

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

/**
 * Nova Hooks System
 *
 * Implements React-like hooks for Nova components.
 * Uses a simple render context that tracks hook state.
 */

type SetStateAction<S> = S | ((prevState: S) => S);
type Dispatch<A> = (action: A) => void;
/**
 * Returns a stateful value and a function to update it.
 *
 * @param initialState - Initial state value or function that returns it
 * @returns Tuple of [state, setState]
 */
declare function useState<S>(initialState: S | (() => S)): [S, Dispatch<SetStateAction<S>>];
type EffectCallback = () => void | (() => void);
type DependencyList = readonly unknown[];
/**
 * Accepts a function that contains imperative, possibly effectful code.
 * Effects run after render is complete.
 *
 * @param effect - Effect function (can return a cleanup function)
 * @param deps - Dependencies array (effect re-runs when these change)
 */
declare function useEffect(effect: EffectCallback, deps?: DependencyList): void;
/**
 * Returns a memoized value. Re-computes only when dependencies change.
 *
 * @param factory - Function that computes the value
 * @param deps - Dependencies array
 * @returns The memoized value
 */
declare function useMemo<T>(factory: () => T, deps: DependencyList): T;
/**
 * Returns a memoized callback. Only changes when dependencies change.
 *
 * @param callback - The callback function
 * @param deps - Dependencies array
 * @returns The memoized callback
 */
declare function useCallback<T extends (...args: unknown[]) => unknown>(callback: T, deps: DependencyList): T;
interface MutableRefObject<T> {
    current: T;
}
/**
 * Returns a mutable ref object.
 *
 * @param initialValue - Initial value for ref.current
 * @returns A ref object
 */
declare function useRef<T>(initialValue: T): MutableRefObject<T>;
type Reducer<S, A> = (prevState: S, action: A) => S;
/**
 * Alternative to useState for complex state logic.
 *
 * @param reducer - Reducer function
 * @param initialState - Initial state
 * @returns Tuple of [state, dispatch]
 */
declare function useReducer<S, A>(reducer: Reducer<S, A>, initialState: S): [S, Dispatch<A>];
/**
 * Generate a unique ID for accessibility attributes.
 *
 * @returns A unique ID string
 */
declare function useId(): string;
/**
 * Mount a component as the root of a command.
 * This should be called from Nova.registerCommand handlers.
 *
 * @param component - The component function to render
 */
declare function render(component: () => NovaElement | null): void;
/**
 * Unmount a component and run cleanup effects.
 */
declare function unmount(componentId: string): void;
declare global {
    var __nova_current_command: string | null;
}

/**
 * Navigation Hook
 *
 * Provides a hook for multi-view navigation in Nova extensions.
 */

/**
 * Navigation state and methods.
 */
interface UseNavigationReturn {
    /**
     * Push a new view onto the navigation stack.
     * Accepts either a serialized component or a JSX element.
     */
    push: (view: ComponentData | NovaElement) => void;
    /**
     * Pop the current view from the navigation stack.
     * Returns true if a view was popped, false if at root.
     */
    pop: () => boolean;
    /**
     * Get the current stack depth.
     */
    depth: () => number;
    /**
     * Whether we can go back (depth > 0).
     */
    canGoBack: boolean;
}
/**
 * Hook for managing multi-view navigation.
 *
 * @example
 * ```tsx
 * function MyCommand() {
 *   const { push, pop, canGoBack } = useNavigation();
 *
 *   return (
 *     <List>
 *       <List.Item
 *         id="item-1"
 *         title="View Details"
 *         actions={{
 *           children: [{
 *             id: "open",
 *             title: "Open",
 *             onAction: "open-detail"
 *           }]
 *         }}
 *       />
 *     </List>
 *   );
 *
 *   // In action handler:
 *   // push(<DetailView item={selectedItem} />);
 * }
 * ```
 */
declare function useNavigation(): UseNavigationReturn;
/**
 * Register a callback for use in action handlers.
 * Returns a callback ID that can be used in onAction.
 *
 * @example
 * ```tsx
 * const handleOpen = registerCallback((itemId) => {
 *   navigation.push(<DetailView itemId={itemId} />);
 * });
 *
 * <List.Item
 *   id="1"
 *   title="Item"
 *   actions={{
 *     children: [{
 *       id: "open",
 *       title: "Open",
 *       onAction: handleOpen
 *     }]
 *   }}
 * />
 * ```
 */
declare function registerCallback<T extends (...args: unknown[]) => void>(callback: T): string;
/**
 * Get a registered callback by ID.
 * Used internally by the runtime.
 */
declare function getCallback(id: string): ((...args: unknown[]) => void) | undefined;
/**
 * Clear a registered callback.
 */
declare function clearCallback(id: string): void;

/**
 * IPC Bridge
 *
 * Type-safe wrappers around the Nova global API.
 * These functions provide better TypeScript integration and
 * can be used directly instead of calling Nova.* methods.
 */

/**
 * Copy text to the system clipboard.
 * Requires "clipboard" permission in nova.toml.
 *
 * @param text - The text to copy
 */
declare function clipboardCopy(text: string): void;
/**
 * Read text from the system clipboard.
 * Requires "clipboard" permission in nova.toml.
 *
 * @returns The clipboard contents
 */
declare function clipboardRead(): string;
/**
 * Get a value from persistent storage.
 *
 * @param key - The key to retrieve
 * @returns The stored value, or undefined if not found
 */
declare function storageGet<T = unknown>(key: string): T | undefined;
/**
 * Set a value in persistent storage.
 *
 * @param key - The key to set
 * @param value - The value to store (must be JSON-serializable)
 */
declare function storageSet<T = unknown>(key: string, value: T): void;
/**
 * Remove a key from persistent storage.
 *
 * @param key - The key to remove
 */
declare function storageRemove(key: string): void;
/**
 * Get all keys in persistent storage.
 *
 * @returns Array of all storage keys
 */
declare function storageKeys(): string[];
/**
 * Get a user-configured preference value.
 *
 * @param key - The preference key (as defined in nova.toml)
 * @returns The preference value, or undefined if not set
 */
declare function getPreference<T = unknown>(key: string): T | undefined;
/**
 * Get all user-configured preferences.
 *
 * @returns Object containing all preferences
 */
declare function getAllPreferences(): Record<string, unknown>;
/**
 * Perform an HTTP fetch request.
 * Requires the target domain to be listed in permissions.network.
 *
 * @param url - The URL to fetch
 * @param options - Fetch options (method, headers, body)
 * @returns Response with status, headers, and body
 */
declare function fetch(url: string, options?: FetchOptions): Promise<FetchResponse>;
/**
 * Convenience method for JSON GET requests.
 *
 * @param url - The URL to fetch
 * @param headers - Optional additional headers
 * @returns Parsed JSON response
 */
declare function fetchJson<T = unknown>(url: string, headers?: Record<string, string>): Promise<T>;
/**
 * Convenience method for JSON POST requests.
 *
 * @param url - The URL to post to
 * @param body - JSON body to send
 * @param headers - Optional additional headers
 * @returns Parsed JSON response
 */
declare function postJson<T = unknown, R = unknown>(url: string, body: T, headers?: Record<string, string>): Promise<R>;
/**
 * Open a URL in the default browser.
 *
 * @param url - The URL to open
 */
declare function openUrl(url: string): void;
/**
 * Open a file or directory in the default application.
 *
 * @param path - The file path to open
 */
declare function openPath(path: string): void;
/**
 * Show a system notification.
 * Requires "notifications" permission in nova.toml.
 *
 * @param title - Notification title
 * @param body - Optional notification body
 */
declare function showNotification(title: string, body?: string): void;
/**
 * Close the Nova window.
 */
declare function closeWindow(): void;
/**
 * Render a component tree to the Nova UI.
 * Typically called via the render() function from hooks.ts instead.
 *
 * @param component - Serialized component data
 */
declare function renderComponent(component: ComponentData): void;
/**
 * Push a new view onto the navigation stack.
 *
 * @param component - The component to push
 */
declare function navigationPush(component: ComponentData): void;
/**
 * Pop the top view from the navigation stack.
 *
 * @returns True if a view was popped, false if stack was empty
 */
declare function navigationPop(): boolean;
/**
 * Get the current navigation stack depth.
 *
 * @returns The number of views in the stack
 */
declare function navigationDepth(): number;

/**
 * Register a command handler.
 *
 * @param name - Command name (must match command id in nova.toml)
 * @param handler - Handler function
 */
declare function registerCommand(name: string, handler: CommandHandler): void;

export { Accessory, type AccessoryType, type Action, type ActionPanel, type ActionStyle, type ClipboardAPI, type CommandHandler, type CommandProps, type ComponentData, type DateFormat, type DependencyList, Detail, type DetailData, type DetailMetadataData, type DetailMetadataProps, type DetailProps, type Dispatch, type DropdownOption, type EffectCallback, type FetchMethod, type FetchOptions, type FetchResponse, type FieldValidation, Form, type FormCheckboxData, type FormCheckboxProps, type FormData, type FormDatePickerData, type FormDatePickerProps, type FormDropdownData, type FormDropdownProps, type FormFieldData, type FormProps, type FormTextFieldData, type FormTextFieldProps, Fragment, Icon, type IconType, type NovaElement as JSXElement, type KeyModifier, List, type ListChildData, type ListData, type ListFiltering, type ListItemData, type ListItemProps, type ListProps, type ListSectionData, type ListSectionProps, type MetadataItemData, type MetadataItemProps, type MetadataLink, type MutableRefObject, type NavigationAPI, type NovaAPI, type NovaComponent, type NovaElement$1 as NovaElement, type NovaNode, type PreferencesAPI, type Reducer, type SetStateAction, type Shortcut, type StorageAPI, type SystemAPI, type TextFieldType, type UseNavigationReturn, clearCallback, clipboardCopy, clipboardRead, closeWindow, createAction, createActionPanel, fetch, fetchJson, getAllPreferences, getCallback, getPreference, isNovaElement, jsx, jsxDEV, jsxs, navigationDepth, navigationPop, navigationPush, openPath, openUrl, postJson, registerCallback, registerCommand, render, renderComponent, serializeElement, shortcut, showNotification, storageGet, storageKeys, storageRemove, storageSet, unmount, useCallback, useEffect, useId, useMemo, useNavigation, useReducer, useRef, useState };
