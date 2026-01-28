'use strict';

var react = require('react');
var Reconciler = require('react-reconciler');
var ReactJSXRuntime = require('react/jsx-runtime');

function _interopDefault (e) { return e && e.__esModule ? e : { default: e }; }

var Reconciler__default = /*#__PURE__*/_interopDefault(Reconciler);
var ReactJSXRuntime__default = /*#__PURE__*/_interopDefault(ReactJSXRuntime);

// src/types/common.ts
var Icon = {
  system: (name) => ({ type: "system", name }),
  url: (url) => ({ type: "url", url }),
  asset: (name) => ({ type: "asset", name }),
  emoji: (emoji) => ({ type: "emoji", emoji }),
  text: (text, color) => ({ type: "text", text, color })
};
var Accessory = {
  text: (text) => ({ type: "text", text }),
  icon: (icon, text) => ({ type: "icon", icon, text }),
  tag: (value, color) => ({ type: "tag", value, color }),
  date: (date, format) => ({
    type: "date",
    date: typeof date === "string" ? date : date.toISOString(),
    format
  })
};
function shortcut(modifiers, key) {
  return { modifiers, key };
}
var ListRoot = (props) => {
  return react.createElement("List", props);
};
var ListItem = (props) => {
  return react.createElement("List.Item", props);
};
var ListSection = (props) => {
  return react.createElement("List.Section", props);
};
var List = Object.assign(ListRoot, {
  Item: ListItem,
  Section: ListSection
});
var DetailRoot = (props) => {
  return react.createElement("Detail", props);
};
var DetailMetadataComponent = (props) => {
  return react.createElement("Detail.Metadata", props);
};
var MetadataItem = (props) => {
  return react.createElement("Detail.Metadata.Item", props);
};
var DetailMetadata = Object.assign(DetailMetadataComponent, {
  Item: MetadataItem
});
var Detail = Object.assign(DetailRoot, {
  Metadata: DetailMetadata
});
var FormRoot = (props) => {
  return react.createElement("Form", props);
};
var FormTextField = (props) => {
  return react.createElement("Form.TextField", props);
};
var FormDropdown = (props) => {
  return react.createElement("Form.Dropdown", props);
};
var FormCheckbox = (props) => {
  return react.createElement("Form.Checkbox", props);
};
var FormDatePicker = (props) => {
  return react.createElement("Form.DatePicker", props);
};
var Form = Object.assign(FormRoot, {
  TextField: FormTextField,
  Dropdown: FormDropdown,
  Checkbox: FormCheckbox,
  DatePicker: FormDatePicker
});

// src/reconciler/host-config.ts
function serializeListItem(instance) {
  const { props } = instance;
  return {
    id: props.id,
    title: props.title,
    subtitle: props.subtitle,
    icon: props.icon,
    accessories: props.accessories,
    keywords: props.keywords,
    actions: props.actions
  };
}
function serializeListChild(instance) {
  if (instance.type === "List.Item") {
    return {
      type: "List.Item",
      ...serializeListItem(instance)
    };
  } else if (instance.type === "List.Section") {
    return {
      type: "List.Section",
      title: instance.props.title,
      subtitle: instance.props.subtitle,
      children: instance.children.map(serializeListItem)
    };
  }
  throw new Error(`Unknown List child type: ${instance.type}`);
}
function serializeFormField(instance) {
  const { type, props } = instance;
  switch (type) {
    case "Form.TextField":
      return {
        type: "Form.TextField",
        id: props.id,
        title: props.title,
        placeholder: props.placeholder,
        defaultValue: props.defaultValue,
        fieldType: props.fieldType,
        validation: props.validation
      };
    case "Form.Dropdown":
      return {
        type: "Form.Dropdown",
        id: props.id,
        title: props.title,
        defaultValue: props.defaultValue,
        options: props.options
      };
    case "Form.Checkbox":
      return {
        type: "Form.Checkbox",
        id: props.id,
        title: props.title,
        label: props.label,
        defaultValue: props.defaultValue
      };
    case "Form.DatePicker":
      return {
        type: "Form.DatePicker",
        id: props.id,
        title: props.title,
        defaultValue: props.defaultValue,
        includeTime: props.includeTime
      };
    default:
      throw new Error(`Unknown Form field type: ${type}`);
  }
}
function serializeInstance(instance) {
  const { type, props, children } = instance;
  switch (type) {
    case "List":
      return {
        type: "List",
        isLoading: props.isLoading,
        searchBarPlaceholder: props.searchBarPlaceholder,
        filtering: props.filtering,
        onSearchChange: props.onSearchChange,
        onSelectionChange: props.onSelectionChange,
        children: children.map(serializeListChild)
      };
    case "Detail":
      return {
        type: "Detail",
        markdown: props.markdown,
        isLoading: props.isLoading,
        actions: props.actions,
        metadata: props.metadata
      };
    case "Form":
      return {
        type: "Form",
        isLoading: props.isLoading,
        onSubmit: props.onSubmit,
        onChange: props.onChange,
        children: children.map(serializeFormField)
      };
    default:
      throw new Error(`Unknown root component type: ${type}`);
  }
}
var renderCallback = null;
function setRenderCallback(callback) {
  renderCallback = callback;
}
function createHostConfig() {
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
    createInstance(type, props, _rootContainer, _hostContext, _internalHandle) {
      const { children: _, ...restProps } = props;
      return {
        type,
        props: restProps,
        children: []
      };
    },
    createTextInstance(_text, _rootContainer, _hostContext, _internalHandle) {
      throw new Error("Text nodes are not supported in Nova. Use component props instead.");
    },
    // ─────────────────────────────────────────────────────────────────────────
    // Tree operations
    // ─────────────────────────────────────────────────────────────────────────
    appendInitialChild(parent, child) {
      parent.children.push(child);
    },
    appendChild(parent, child) {
      parent.children.push(child);
    },
    appendChildToContainer(container, child) {
      container.root = child;
    },
    insertBefore(parent, child, beforeChild) {
      const index = parent.children.indexOf(beforeChild);
      if (index >= 0) {
        parent.children.splice(index, 0, child);
      } else {
        parent.children.push(child);
      }
    },
    insertInContainerBefore(container, child, _beforeChild) {
      container.root = child;
    },
    removeChild(parent, child) {
      const index = parent.children.indexOf(child);
      if (index >= 0) {
        parent.children.splice(index, 1);
      }
    },
    removeChildFromContainer(container, _child) {
      container.root = null;
    },
    clearContainer(container) {
      container.root = null;
    },
    // ─────────────────────────────────────────────────────────────────────────
    // Updates
    // ─────────────────────────────────────────────────────────────────────────
    prepareUpdate(_instance, _type, oldProps, newProps, _rootContainer, _hostContext) {
      const { children: _oldChildren, ...oldRest } = oldProps;
      const { children: _newChildren, ...newRest } = newProps;
      const keys = /* @__PURE__ */ new Set([...Object.keys(oldRest), ...Object.keys(newRest)]);
      for (const key of keys) {
        if (oldRest[key] !== newRest[key]) {
          return newRest;
        }
      }
      return null;
    },
    commitUpdate(instance, updatePayload, _type, _prevProps, _nextProps, _internalHandle) {
      instance.props = { ...updatePayload };
    },
    // ─────────────────────────────────────────────────────────────────────────
    // Commit phase
    // ─────────────────────────────────────────────────────────────────────────
    prepareForCommit(_containerInfo) {
      return null;
    },
    resetAfterCommit(container) {
      if (container.root && renderCallback) {
        const data = serializeInstance(container.root);
        renderCallback(data);
      }
    },
    // ─────────────────────────────────────────────────────────────────────────
    // Context
    // ─────────────────────────────────────────────────────────────────────────
    getRootHostContext(_rootContainer) {
      return null;
    },
    getChildHostContext(_parentHostContext, _type, _rootContainer) {
      return null;
    },
    // ─────────────────────────────────────────────────────────────────────────
    // Misc required methods
    // ─────────────────────────────────────────────────────────────────────────
    getPublicInstance(instance) {
      return instance;
    },
    finalizeInitialChildren(_instance, _type, _props, _rootContainer, _hostContext) {
      return false;
    },
    shouldSetTextContent(_type, _props) {
      return false;
    },
    // ─────────────────────────────────────────────────────────────────────────
    // Scheduling
    // ─────────────────────────────────────────────────────────────────────────
    scheduleMicrotask: queueMicrotask,
    scheduleTimeout: setTimeout,
    cancelTimeout: clearTimeout,
    noTimeout: -1,
    getCurrentEventPriority() {
      return 16;
    },
    getInstanceFromNode() {
      return null;
    },
    beforeActiveInstanceBlur() {
    },
    afterActiveInstanceBlur() {
    },
    prepareScopeUpdate() {
    },
    getInstanceFromScope() {
      return null;
    },
    detachDeletedInstance() {
    },
    preparePortalMount() {
    },
    // ─────────────────────────────────────────────────────────────────────────
    // Not used but required
    // ─────────────────────────────────────────────────────────────────────────
    hideInstance() {
    },
    unhideInstance() {
    },
    hideTextInstance() {
    },
    unhideTextInstance() {
    },
    commitMount() {
    },
    resetTextContent() {
    },
    commitTextUpdate() {
    }
  };
}

// src/reconciler/render.ts
var hostConfig = createHostConfig();
var reconciler = Reconciler__default.default(hostConfig);
var containers = /* @__PURE__ */ new Map();
setRenderCallback((data) => {
  if (typeof Nova !== "undefined" && Nova.render) {
    Nova.render(data);
  } else {
    console.log("[Nova SDK] Rendered:", JSON.stringify(data, null, 2));
  }
});
function render(element) {
  const commandId = globalThis.__nova_current_command ?? "default";
  let entry = containers.get(commandId);
  if (!entry) {
    const containerInfo = { root: null };
    const container = reconciler.createContainer(
      containerInfo,
      0,
      // ConcurrentRoot would be 1, but we use legacy mode for simplicity
      null,
      // hydrationCallbacks
      false,
      // isStrictMode
      null,
      // concurrentUpdatesByDefaultOverride
      commandId,
      // identifierPrefix
      (error) => {
        console.error("[Nova SDK] Render error:", error);
      },
      null
      // transitionCallbacks
    );
    entry = { container, containerInfo };
    containers.set(commandId, entry);
  }
  reconciler.updateContainer(element, entry.container, null, () => {
  });
}
function unmount(commandId) {
  const id = commandId ?? globalThis.__nova_current_command ?? "default";
  const entry = containers.get(id);
  if (entry) {
    reconciler.updateContainer(null, entry.container, null, () => {
      containers.delete(id);
    });
  }
}

// src/components.ts
var List2 = Object.assign(
  function List3(props) {
    return {
      type: "List",
      isLoading: props.isLoading,
      searchBarPlaceholder: props.searchBarPlaceholder,
      filtering: props.filtering,
      onSearchChange: props.onSearchChange,
      onSelectionChange: props.onSelectionChange,
      children: props.children ?? []
    };
  },
  {
    /**
     * Create a List.Item.
     */
    Item: function ListItem2(props) {
      return {
        type: "List.Item",
        id: props.id,
        title: props.title,
        subtitle: props.subtitle,
        icon: props.icon,
        accessories: props.accessories,
        keywords: props.keywords,
        actions: props.actions
      };
    },
    /**
     * Create a List.Section.
     */
    Section: function ListSection2(props) {
      const items = props.items ?? [];
      return {
        type: "List.Section",
        title: props.title,
        subtitle: props.subtitle,
        children: items.map((item) => ({
          id: item.id,
          title: item.title,
          subtitle: item.subtitle,
          icon: item.icon,
          accessories: item.accessories,
          keywords: item.keywords,
          actions: item.actions
        }))
      };
    }
  }
);
var DetailMetadata2 = Object.assign(
  function DetailMetadata3(props) {
    return {
      children: (props.children ?? []).map((item) => ({
        title: item.title,
        text: item.text,
        icon: item.icon,
        link: item.link
      }))
    };
  },
  {
    /**
     * Create Detail.Metadata.Item.
     */
    Item: function MetadataItem2(props) {
      return {
        title: props.title,
        text: props.text,
        icon: props.icon,
        link: props.link
      };
    }
  }
);
var Detail2 = Object.assign(
  function Detail3(props) {
    return {
      type: "Detail",
      markdown: props.markdown,
      isLoading: props.isLoading,
      actions: props.actions,
      metadata: props.metadata
    };
  },
  {
    Metadata: DetailMetadata2
  }
);
var Form2 = Object.assign(
  function Form3(props) {
    return {
      type: "Form",
      isLoading: props.isLoading,
      onSubmit: props.onSubmit,
      onChange: props.onChange,
      children: props.children ?? []
    };
  },
  {
    /**
     * Create a Form.TextField.
     */
    TextField: function FormTextField2(props) {
      return {
        type: "Form.TextField",
        id: props.id,
        title: props.title,
        placeholder: props.placeholder,
        defaultValue: props.defaultValue,
        fieldType: props.fieldType,
        validation: props.validation
      };
    },
    /**
     * Create a Form.Dropdown.
     */
    Dropdown: function FormDropdown2(props) {
      return {
        type: "Form.Dropdown",
        id: props.id,
        title: props.title,
        defaultValue: props.defaultValue,
        options: props.options
      };
    },
    /**
     * Create a Form.Checkbox.
     */
    Checkbox: function FormCheckbox2(props) {
      return {
        type: "Form.Checkbox",
        id: props.id,
        title: props.title,
        label: props.label,
        defaultValue: props.defaultValue
      };
    },
    /**
     * Create a Form.DatePicker.
     */
    DatePicker: function FormDatePicker2(props) {
      return {
        type: "Form.DatePicker",
        id: props.id,
        title: props.title,
        defaultValue: props.defaultValue,
        includeTime: props.includeTime
      };
    }
  }
);
function createActionPanel(title, actions) {
  return { title, children: actions };
}
function createAction(props) {
  return {
    id: props.id,
    title: props.title,
    icon: props.icon,
    shortcut: props.shortcut,
    style: props.style ?? "default",
    onAction: props.onAction
  };
}
function useNavigation() {
  const [stackDepth, setStackDepth] = react.useState(() => {
    if (typeof Nova !== "undefined" && Nova.navigation) {
      return Nova.navigation.depth();
    }
    return 0;
  });
  const push = react.useCallback((view) => {
    if (typeof Nova !== "undefined" && Nova.navigation) {
      Nova.navigation.push(view);
      setStackDepth((d) => d + 1);
    }
  }, []);
  const pop = react.useCallback(() => {
    if (typeof Nova !== "undefined" && Nova.navigation) {
      const result = Nova.navigation.pop();
      if (result) {
        setStackDepth((d) => Math.max(0, d - 1));
      }
      return result;
    }
    return false;
  }, []);
  const getDepth = react.useCallback(() => {
    if (typeof Nova !== "undefined" && Nova.navigation) {
      return Nova.navigation.depth();
    }
    return stackDepth;
  }, [stackDepth]);
  const canGoBack = stackDepth > 0;
  return react.useMemo(
    () => ({
      push,
      pop,
      depth: getDepth,
      canGoBack
    }),
    [push, pop, getDepth, canGoBack]
  );
}
var callbacks = /* @__PURE__ */ new Map();
var callbackCounter = 0;
function registerCallback(callback) {
  const id = `cb_${++callbackCounter}_${Date.now()}`;
  callbacks.set(id, callback);
  if (typeof Nova !== "undefined" && Nova.registerEventHandler) {
    Nova.registerEventHandler(id, callback);
  }
  return id;
}
function getCallback(id) {
  return callbacks.get(id);
}
function clearCallback(id) {
  callbacks.delete(id);
}

// src/ipc.ts
function clipboardCopy(text) {
  Nova.clipboard.copy(text);
}
function clipboardRead() {
  return Nova.clipboard.read();
}
function storageGet(key) {
  return Nova.storage.get(key);
}
function storageSet(key, value) {
  Nova.storage.set(key, value);
}
function storageRemove(key) {
  Nova.storage.remove(key);
}
function storageKeys() {
  return Nova.storage.keys();
}
function getPreference(key) {
  return Nova.preferences.get(key);
}
function getAllPreferences() {
  return Nova.preferences.all();
}
async function fetch(url, options) {
  return Nova.fetch(url, options);
}
async function fetchJson(url, headers) {
  const response = await Nova.fetch(url, {
    method: "GET",
    headers: {
      Accept: "application/json",
      ...headers
    }
  });
  if (response.status >= 400) {
    throw new Error(`HTTP ${response.status}: ${response.body}`);
  }
  return JSON.parse(response.body);
}
async function postJson(url, body, headers) {
  const response = await Nova.fetch(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
      ...headers
    },
    body: JSON.stringify(body)
  });
  if (response.status >= 400) {
    throw new Error(`HTTP ${response.status}: ${response.body}`);
  }
  return JSON.parse(response.body);
}
function openUrl(url) {
  Nova.system.openUrl(url);
}
function openPath(path) {
  Nova.system.openPath(path);
}
function showNotification(title, body) {
  Nova.system.notify(title, body ?? "");
}
function closeWindow() {
  Nova.system.closeWindow();
}
function renderComponent(component) {
  Nova.render(component);
}
function navigationPush(component) {
  Nova.navigation.push(component);
}
function navigationPop() {
  return Nova.navigation.pop();
}
function navigationDepth() {
  return Nova.navigation.depth();
}
function registerCommand(name, handler) {
  Nova.registerCommand(name, handler);
}
var jsx = ReactJSXRuntime__default.default.jsx;
var jsxs = ReactJSXRuntime__default.default.jsxs;
var Fragment = ReactJSXRuntime__default.default.Fragment;
var jsxDEV = ReactJSXRuntime__default.default.jsx;

Object.defineProperty(exports, "useCallback", {
  enumerable: true,
  get: function () { return react.useCallback; }
});
Object.defineProperty(exports, "useContext", {
  enumerable: true,
  get: function () { return react.useContext; }
});
Object.defineProperty(exports, "useDebugValue", {
  enumerable: true,
  get: function () { return react.useDebugValue; }
});
Object.defineProperty(exports, "useDeferredValue", {
  enumerable: true,
  get: function () { return react.useDeferredValue; }
});
Object.defineProperty(exports, "useEffect", {
  enumerable: true,
  get: function () { return react.useEffect; }
});
Object.defineProperty(exports, "useId", {
  enumerable: true,
  get: function () { return react.useId; }
});
Object.defineProperty(exports, "useImperativeHandle", {
  enumerable: true,
  get: function () { return react.useImperativeHandle; }
});
Object.defineProperty(exports, "useInsertionEffect", {
  enumerable: true,
  get: function () { return react.useInsertionEffect; }
});
Object.defineProperty(exports, "useLayoutEffect", {
  enumerable: true,
  get: function () { return react.useLayoutEffect; }
});
Object.defineProperty(exports, "useMemo", {
  enumerable: true,
  get: function () { return react.useMemo; }
});
Object.defineProperty(exports, "useReducer", {
  enumerable: true,
  get: function () { return react.useReducer; }
});
Object.defineProperty(exports, "useRef", {
  enumerable: true,
  get: function () { return react.useRef; }
});
Object.defineProperty(exports, "useState", {
  enumerable: true,
  get: function () { return react.useState; }
});
Object.defineProperty(exports, "useSyncExternalStore", {
  enumerable: true,
  get: function () { return react.useSyncExternalStore; }
});
Object.defineProperty(exports, "useTransition", {
  enumerable: true,
  get: function () { return react.useTransition; }
});
exports.Accessory = Accessory;
exports.Detail = Detail;
exports.DetailFactory = Detail2;
exports.Form = Form;
exports.FormFactory = Form2;
exports.Fragment = Fragment;
exports.Icon = Icon;
exports.List = List;
exports.ListFactory = List2;
exports.clearCallback = clearCallback;
exports.clipboardCopy = clipboardCopy;
exports.clipboardRead = clipboardRead;
exports.closeWindow = closeWindow;
exports.createAction = createAction;
exports.createActionPanel = createActionPanel;
exports.fetch = fetch;
exports.fetchJson = fetchJson;
exports.getAllPreferences = getAllPreferences;
exports.getCallback = getCallback;
exports.getPreference = getPreference;
exports.jsx = jsx;
exports.jsxDEV = jsxDEV;
exports.jsxs = jsxs;
exports.navigationDepth = navigationDepth;
exports.navigationPop = navigationPop;
exports.navigationPush = navigationPush;
exports.openPath = openPath;
exports.openUrl = openUrl;
exports.postJson = postJson;
exports.registerCallback = registerCallback;
exports.registerCommand = registerCommand;
exports.render = render;
exports.renderComponent = renderComponent;
exports.shortcut = shortcut;
exports.showNotification = showNotification;
exports.storageGet = storageGet;
exports.storageKeys = storageKeys;
exports.storageRemove = storageRemove;
exports.storageSet = storageSet;
exports.unmount = unmount;
exports.useNavigation = useNavigation;
//# sourceMappingURL=index.cjs.map
//# sourceMappingURL=index.cjs.map