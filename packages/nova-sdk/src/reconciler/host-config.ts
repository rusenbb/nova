/**
 * React Reconciler Host Config for Nova
 *
 * Implements the platform-specific operations for react-reconciler.
 * This enables React components to be rendered to Nova's native UI.
 */

import type {
  ListData,
  ListChildData,
  ListItemData,
  ListSectionData,
  DetailData,
  FormData,
  FormFieldData,
  ComponentData,
} from "../types/index.js";

// ─────────────────────────────────────────────────────────────────────────────
// Instance Types
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Nova component types that can be created.
 */
export type NovaType =
  | "List"
  | "List.Item"
  | "List.Section"
  | "Detail"
  | "Detail.Metadata"
  | "Detail.Metadata.Item"
  | "Form"
  | "Form.TextField"
  | "Form.Dropdown"
  | "Form.Checkbox"
  | "Form.DatePicker";

/**
 * Props for Nova instances (generic object).
 */
export type NovaProps = Record<string, unknown>;

/**
 * A Nova component instance in the virtual tree.
 */
export interface NovaInstance {
  type: NovaType;
  props: NovaProps;
  children: NovaInstance[];
}

/**
 * Container that holds the root instance.
 */
export interface NovaContainer {
  root: NovaInstance | null;
}

/**
 * Text instance (not supported in Nova).
 */
export type NovaTextInstance = never;

/**
 * Host context (unused in Nova).
 */
export type NovaHostContext = null;

/**
 * Update payload for commitUpdate.
 */
export type NovaUpdatePayload = NovaProps;

// ─────────────────────────────────────────────────────────────────────────────
// Serialization Functions
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Serialize a List.Item instance.
 */
function serializeListItem(instance: NovaInstance): Omit<ListItemData, "type"> {
  const { props } = instance;
  return {
    id: props.id as string,
    title: props.title as string,
    subtitle: props.subtitle as string | undefined,
    icon: props.icon as ListItemData["icon"],
    accessories: props.accessories as ListItemData["accessories"],
    keywords: props.keywords as string[] | undefined,
    actions: props.actions as ListItemData["actions"],
  };
}

/**
 * Serialize a List child (Item or Section).
 */
function serializeListChild(instance: NovaInstance): ListChildData {
  if (instance.type === "List.Item") {
    return {
      type: "List.Item",
      ...serializeListItem(instance),
    };
  } else if (instance.type === "List.Section") {
    return {
      type: "List.Section",
      title: instance.props.title as string | undefined,
      subtitle: instance.props.subtitle as string | undefined,
      children: instance.children.map(serializeListItem),
    } as ListSectionData;
  }
  throw new Error(`Unknown List child type: ${instance.type}`);
}

/**
 * Serialize a Form field instance.
 */
function serializeFormField(instance: NovaInstance): FormFieldData {
  const { type, props } = instance;
  switch (type) {
    case "Form.TextField":
      return {
        type: "Form.TextField",
        id: props.id as string,
        title: props.title as string,
        placeholder: props.placeholder as string | undefined,
        defaultValue: props.defaultValue as string | undefined,
        fieldType: props.fieldType as FormFieldData extends { fieldType?: infer T } ? T : never,
        validation: props.validation as FormFieldData extends { validation?: infer T } ? T : never,
      };
    case "Form.Dropdown":
      return {
        type: "Form.Dropdown",
        id: props.id as string,
        title: props.title as string,
        defaultValue: props.defaultValue as string | undefined,
        options: props.options as Array<{ value: string; title: string }>,
      };
    case "Form.Checkbox":
      return {
        type: "Form.Checkbox",
        id: props.id as string,
        title: props.title as string,
        label: props.label as string | undefined,
        defaultValue: props.defaultValue as boolean | undefined,
      };
    case "Form.DatePicker":
      return {
        type: "Form.DatePicker",
        id: props.id as string,
        title: props.title as string,
        defaultValue: props.defaultValue as string | undefined,
        includeTime: props.includeTime as boolean | undefined,
      };
    default:
      throw new Error(`Unknown Form field type: ${type}`);
  }
}

/**
 * Serialize the root instance to ComponentData.
 */
export function serializeInstance(instance: NovaInstance): ComponentData {
  const { type, props, children } = instance;

  switch (type) {
    case "List":
      return {
        type: "List",
        isLoading: props.isLoading as boolean | undefined,
        searchBarPlaceholder: props.searchBarPlaceholder as string | undefined,
        filtering: props.filtering as ListData["filtering"],
        onSearchChange: props.onSearchChange as string | undefined,
        onSelectionChange: props.onSelectionChange as string | undefined,
        children: children.map(serializeListChild),
      } as ListData;

    case "Detail":
      return {
        type: "Detail",
        markdown: props.markdown as string | undefined,
        isLoading: props.isLoading as boolean | undefined,
        actions: props.actions as DetailData["actions"],
        metadata: props.metadata as DetailData["metadata"],
      } as DetailData;

    case "Form":
      return {
        type: "Form",
        isLoading: props.isLoading as boolean | undefined,
        onSubmit: props.onSubmit as string | undefined,
        onChange: props.onChange as string | undefined,
        children: children.map(serializeFormField),
      } as FormData;

    default:
      throw new Error(`Unknown root component type: ${type}`);
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Host Config Implementation
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Callback to send rendered component data to Nova.
 */
let renderCallback: ((data: ComponentData) => void) | null = null;

/**
 * Set the render callback that sends data to Nova.
 */
export function setRenderCallback(callback: (data: ComponentData) => void): void {
  renderCallback = callback;
}

/**
 * Create the host config object for react-reconciler.
 */
export function createHostConfig() {
  return {
    // ─────────────────────────────────────────────────────────────────────────
    // Feature flags
    // ─────────────────────────────────────────────────────────────────────────
    supportsMutation: true,
    supportsPersistence: false,
    supportsHydration: false,
    isPrimaryRenderer: true,
    warnsIfNotActing: false,

    // ─────────────────────────────────────────────────────────────────────────
    // Instance creation
    // ─────────────────────────────────────────────────────────────────────────
    createInstance(
      type: NovaType,
      props: NovaProps,
      _rootContainer: NovaContainer,
      _hostContext: NovaHostContext,
      _internalHandle: unknown
    ): NovaInstance {
      // Remove children from props (they're handled separately)
      const { children: _, ...restProps } = props;
      return {
        type,
        props: restProps,
        children: [],
      };
    },

    createTextInstance(
      _text: string,
      _rootContainer: NovaContainer,
      _hostContext: NovaHostContext,
      _internalHandle: unknown
    ): NovaTextInstance {
      throw new Error("Text nodes are not supported in Nova. Use component props instead.");
    },

    // ─────────────────────────────────────────────────────────────────────────
    // Tree operations
    // ─────────────────────────────────────────────────────────────────────────
    appendInitialChild(parent: NovaInstance, child: NovaInstance): void {
      parent.children.push(child);
    },

    appendChild(parent: NovaInstance, child: NovaInstance): void {
      parent.children.push(child);
    },

    appendChildToContainer(container: NovaContainer, child: NovaInstance): void {
      container.root = child;
    },

    insertBefore(
      parent: NovaInstance,
      child: NovaInstance,
      beforeChild: NovaInstance
    ): void {
      const index = parent.children.indexOf(beforeChild);
      if (index >= 0) {
        parent.children.splice(index, 0, child);
      } else {
        parent.children.push(child);
      }
    },

    insertInContainerBefore(
      container: NovaContainer,
      child: NovaInstance,
      _beforeChild: NovaInstance
    ): void {
      // Container only has one root
      container.root = child;
    },

    removeChild(parent: NovaInstance, child: NovaInstance): void {
      const index = parent.children.indexOf(child);
      if (index >= 0) {
        parent.children.splice(index, 1);
      }
    },

    removeChildFromContainer(container: NovaContainer, _child: NovaInstance): void {
      container.root = null;
    },

    clearContainer(container: NovaContainer): void {
      container.root = null;
    },

    // ─────────────────────────────────────────────────────────────────────────
    // Updates
    // ─────────────────────────────────────────────────────────────────────────
    prepareUpdate(
      _instance: NovaInstance,
      _type: NovaType,
      oldProps: NovaProps,
      newProps: NovaProps,
      _rootContainer: NovaContainer,
      _hostContext: NovaHostContext
    ): NovaUpdatePayload | null {
      // Return new props if anything changed
      const { children: _oldChildren, ...oldRest } = oldProps;
      const { children: _newChildren, ...newRest } = newProps;

      // Simple shallow comparison
      const keys = new Set([...Object.keys(oldRest), ...Object.keys(newRest)]);
      for (const key of keys) {
        if (oldRest[key] !== newRest[key]) {
          return newRest;
        }
      }
      return null;
    },

    commitUpdate(
      instance: NovaInstance,
      updatePayload: NovaUpdatePayload,
      _type: NovaType,
      _prevProps: NovaProps,
      _nextProps: NovaProps,
      _internalHandle: unknown
    ): void {
      instance.props = { ...updatePayload };
    },

    // ─────────────────────────────────────────────────────────────────────────
    // Commit phase
    // ─────────────────────────────────────────────────────────────────────────
    prepareForCommit(_containerInfo: NovaContainer): Record<string, unknown> | null {
      return null;
    },

    resetAfterCommit(container: NovaContainer): void {
      if (container.root && renderCallback) {
        const data = serializeInstance(container.root);
        renderCallback(data);
      }
    },

    // ─────────────────────────────────────────────────────────────────────────
    // Context
    // ─────────────────────────────────────────────────────────────────────────
    getRootHostContext(_rootContainer: NovaContainer): NovaHostContext {
      return null;
    },

    getChildHostContext(
      _parentHostContext: NovaHostContext,
      _type: NovaType,
      _rootContainer: NovaContainer
    ): NovaHostContext {
      return null;
    },

    // ─────────────────────────────────────────────────────────────────────────
    // Misc required methods
    // ─────────────────────────────────────────────────────────────────────────
    getPublicInstance(instance: NovaInstance): NovaInstance {
      return instance;
    },

    finalizeInitialChildren(
      _instance: NovaInstance,
      _type: NovaType,
      _props: NovaProps,
      _rootContainer: NovaContainer,
      _hostContext: NovaHostContext
    ): boolean {
      return false;
    },

    shouldSetTextContent(_type: NovaType, _props: NovaProps): boolean {
      return false;
    },

    // ─────────────────────────────────────────────────────────────────────────
    // Scheduling
    // ─────────────────────────────────────────────────────────────────────────
    scheduleMicrotask: queueMicrotask,

    scheduleTimeout: setTimeout,
    cancelTimeout: clearTimeout,
    noTimeout: -1,

    getCurrentEventPriority(): number {
      return 16; // DefaultEventPriority
    },

    getInstanceFromNode(): null {
      return null;
    },

    beforeActiveInstanceBlur(): void {},
    afterActiveInstanceBlur(): void {},
    prepareScopeUpdate(): void {},
    getInstanceFromScope(): null {
      return null;
    },

    detachDeletedInstance(): void {},

    preparePortalMount(): void {},

    // ─────────────────────────────────────────────────────────────────────────
    // Not used but required
    // ─────────────────────────────────────────────────────────────────────────
    hideInstance(): void {},
    unhideInstance(): void {},
    hideTextInstance(): void {},
    unhideTextInstance(): void {},
    commitMount(): void {},
    resetTextContent(): void {},
    commitTextUpdate(): void {},
  };
}
