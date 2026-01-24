'use strict';

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

// src/components.ts
var List = Object.assign(
  function List2(props) {
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
    Item: function ListItem(props) {
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
    Section: function ListSection(props) {
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
var DetailMetadata = Object.assign(
  function DetailMetadata2(props) {
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
    Item: function MetadataItem(props) {
      return {
        title: props.title,
        text: props.text,
        icon: props.icon,
        link: props.link
      };
    }
  }
);
var Detail = Object.assign(
  function Detail2(props) {
    return {
      type: "Detail",
      markdown: props.markdown,
      isLoading: props.isLoading,
      actions: props.actions,
      metadata: props.metadata
    };
  },
  {
    Metadata: DetailMetadata
  }
);
var Form = Object.assign(
  function Form2(props) {
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
    TextField: function FormTextField(props) {
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
    Dropdown: function FormDropdown(props) {
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
    Checkbox: function FormCheckbox(props) {
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
    DatePicker: function FormDatePicker(props) {
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

// src/jsx-runtime.ts
var NOVA_ELEMENT_TYPE = /* @__PURE__ */ Symbol.for("nova.element");
function isNovaElement(value) {
  return typeof value === "object" && value !== null && value.$$typeof === NOVA_ELEMENT_TYPE;
}
function jsx(type, props, key) {
  return {
    $$typeof: NOVA_ELEMENT_TYPE,
    type,
    props,
    key: key ?? null
  };
}
function jsxs(type, props, key) {
  return jsx(type, props, key);
}
var Fragment = /* @__PURE__ */ Symbol.for("nova.fragment");
var jsxDEV = jsx;
function flattenChildren(children) {
  if (children == null || typeof children === "boolean") {
    return [];
  }
  if (Array.isArray(children)) {
    return children.flatMap(flattenChildren);
  }
  if (isNovaElement(children)) {
    return [children];
  }
  return [];
}
function serializeElement(element) {
  const { type, props } = element;
  if (typeof type === "function") {
    const rendered = type(props);
    if (rendered === null) {
      throw new Error("Component returned null - Nova requires a component tree");
    }
    return serializeElement(rendered);
  }
  switch (type) {
    case "List":
      return serializeList(props);
    case "Detail":
      return serializeDetail(props);
    case "Form":
      return serializeForm(props);
    default:
      throw new Error(`Unknown component type: ${type}`);
  }
}
function serializeList(props) {
  const children = flattenChildren(props.children);
  const serializedChildren = [];
  for (const child of children) {
    if (typeof child.type !== "string") {
      throw new Error("List children must be List.Item or List.Section");
    }
    switch (child.type) {
      case "List.Item": {
        const itemProps = child.props;
        serializedChildren.push({
          type: "List.Item",
          id: itemProps.id,
          title: itemProps.title,
          subtitle: itemProps.subtitle,
          icon: itemProps.icon,
          accessories: itemProps.accessories,
          keywords: itemProps.keywords,
          actions: itemProps.actions
        });
        break;
      }
      case "List.Section": {
        const sectionProps = child.props;
        const sectionChildren = flattenChildren(sectionProps.children);
        serializedChildren.push({
          type: "List.Section",
          title: sectionProps.title,
          subtitle: sectionProps.subtitle,
          children: sectionChildren.map((item) => {
            const itemProps = item.props;
            return {
              id: itemProps.id,
              title: itemProps.title,
              subtitle: itemProps.subtitle,
              icon: itemProps.icon,
              accessories: itemProps.accessories,
              keywords: itemProps.keywords,
              actions: itemProps.actions
            };
          })
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
    children: serializedChildren
  };
}
function serializeDetail(props) {
  return {
    type: "Detail",
    markdown: props.markdown,
    isLoading: props.isLoading,
    actions: props.actions,
    metadata: props.metadata
  };
}
function serializeForm(props) {
  const children = flattenChildren(props.children);
  const serializedChildren = [];
  for (const child of children) {
    if (typeof child.type !== "string") {
      throw new Error("Form children must be form field components");
    }
    switch (child.type) {
      case "Form.TextField": {
        const fieldProps = child.props;
        serializedChildren.push({
          type: "Form.TextField",
          id: fieldProps.id,
          title: fieldProps.title,
          placeholder: fieldProps.placeholder,
          defaultValue: fieldProps.defaultValue,
          fieldType: fieldProps.fieldType,
          validation: fieldProps.validation
        });
        break;
      }
      case "Form.Dropdown": {
        const fieldProps = child.props;
        serializedChildren.push({
          type: "Form.Dropdown",
          id: fieldProps.id,
          title: fieldProps.title,
          defaultValue: fieldProps.defaultValue,
          options: fieldProps.options
        });
        break;
      }
      case "Form.Checkbox": {
        const fieldProps = child.props;
        serializedChildren.push({
          type: "Form.Checkbox",
          id: fieldProps.id,
          title: fieldProps.title,
          label: fieldProps.label,
          defaultValue: fieldProps.defaultValue
        });
        break;
      }
      case "Form.DatePicker": {
        const fieldProps = child.props;
        serializedChildren.push({
          type: "Form.DatePicker",
          id: fieldProps.id,
          title: fieldProps.title,
          defaultValue: fieldProps.defaultValue,
          includeTime: fieldProps.includeTime
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
    children: serializedChildren
  };
}

// src/hooks.ts
var currentContext = null;
function getContext() {
  if (!currentContext) {
    throw new Error(
      "Hooks can only be called inside a component during render. Make sure you are not calling hooks conditionally or in event handlers."
    );
  }
  return currentContext;
}
function getHookState(initialValue) {
  const ctx = getContext();
  const index = ctx.hookIndex++;
  if (ctx.isFirstRender || index >= ctx.hookStates.length) {
    ctx.hookStates[index] = { value: initialValue() };
    return { state: ctx.hookStates[index].value, index, isNew: true };
  }
  return { state: ctx.hookStates[index].value, index, isNew: false };
}
function useState(initialState) {
  const ctx = getContext();
  const { state, index } = getHookState(
    () => typeof initialState === "function" ? initialState() : initialState
  );
  const setState = (action) => {
    const prevState = ctx.hookStates[index].value;
    const nextState = typeof action === "function" ? action(prevState) : action;
    if (!Object.is(prevState, nextState)) {
      ctx.hookStates[index].value = nextState;
      ctx.requestRender();
    }
  };
  return [state, setState];
}
function useEffect(effect, deps) {
  const ctx = getContext();
  const { index, isNew } = getHookState(() => void 0);
  const prevDeps = ctx.hookStates[index].deps;
  const depsChanged = isNew || !deps || !prevDeps || !depsEqual(prevDeps, deps);
  if (depsChanged) {
    ctx.hookStates[index].deps = deps ? [...deps] : void 0;
    ctx.pendingEffects.push(() => {
      const cleanup = ctx.effectCleanups.get(index);
      if (cleanup) {
        cleanup();
        ctx.effectCleanups.delete(index);
      }
      const newCleanup = effect();
      if (typeof newCleanup === "function") {
        ctx.effectCleanups.set(index, newCleanup);
      }
    });
  }
}
function useMemo(factory, deps) {
  const ctx = getContext();
  const { state, index, isNew } = getHookState(() => ({
    value: factory(),
    deps: [...deps]
  }));
  if (!isNew && depsEqual(state.deps, deps)) {
    return state.value;
  }
  const newValue = factory();
  ctx.hookStates[index].value = { value: newValue, deps: [...deps] };
  return newValue;
}
function useCallback(callback, deps) {
  return useMemo(() => callback, deps);
}
function useRef(initialValue) {
  const { state } = getHookState(() => ({
    current: initialValue
  }));
  return state;
}
function useReducer(reducer, initialState) {
  const ctx = getContext();
  const { state, index } = getHookState(() => initialState);
  const dispatch = (action) => {
    const prevState = ctx.hookStates[index].value;
    const nextState = reducer(prevState, action);
    if (!Object.is(prevState, nextState)) {
      ctx.hookStates[index].value = nextState;
      ctx.requestRender();
    }
  };
  return [state, dispatch];
}
var idCounter = 0;
function useId() {
  const { state } = getHookState(() => `nova-id-${++idCounter}`);
  return state;
}
function depsEqual(a, b) {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (!Object.is(a[i], b[i])) return false;
  }
  return true;
}
var componentInstances = /* @__PURE__ */ new Map();
function createRenderContext(componentId) {
  const existing = componentInstances.get(componentId);
  return {
    hookIndex: 0,
    hookStates: existing?.context.hookStates ?? [],
    isFirstRender: !existing,
    pendingEffects: [],
    effectCleanups: existing?.context.effectCleanups ?? /* @__PURE__ */ new Map(),
    requestRender: () => {
      queueMicrotask(() => {
        const instance = componentInstances.get(componentId);
        if (instance) {
          renderComponent(componentId, instance.component);
        }
      });
    }
  };
}
function renderComponent(componentId, component) {
  const context = createRenderContext(componentId);
  currentContext = context;
  try {
    const element = component();
    if (element === null) {
      throw new Error("Component returned null");
    }
    componentInstances.set(componentId, { component, context });
    const data = serializeElement(element);
    Nova.render(data);
    for (const effect of context.pendingEffects) {
      effect();
    }
  } finally {
    currentContext = null;
  }
}
function render(component) {
  const commandId = globalThis.__nova_current_command ?? "default";
  renderComponent(commandId, component);
}
function unmount(componentId) {
  const instance = componentInstances.get(componentId);
  if (instance) {
    for (const cleanup of instance.context.effectCleanups.values()) {
      cleanup();
    }
    componentInstances.delete(componentId);
  }
}

// src/navigation.ts
function useNavigation() {
  const [depth, setDepth] = useState(() => Nova.navigation.depth());
  const push = (view) => {
    const data = isNovaElement2(view) ? serializeElement(view) : view;
    Nova.navigation.push(data);
    setDepth((d) => d + 1);
  };
  const pop = () => {
    const result = Nova.navigation.pop();
    if (result) {
      setDepth((d) => Math.max(0, d - 1));
    }
    return result;
  };
  const getDepth = () => {
    return Nova.navigation.depth();
  };
  const canGoBack = depth > 0;
  return {
    push,
    pop,
    depth: getDepth,
    canGoBack
  };
}
function isNovaElement2(value) {
  return typeof value === "object" && value !== null && "$$typeof" in value && value.$$typeof === /* @__PURE__ */ Symbol.for("nova.element");
}
var callbacks = /* @__PURE__ */ new Map();
var callbackCounter = 0;
function registerCallback(callback) {
  const id = `cb_${++callbackCounter}_${Date.now()}`;
  callbacks.set(id, callback);
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
function renderComponent2(component) {
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

exports.Accessory = Accessory;
exports.Detail = Detail;
exports.Form = Form;
exports.Fragment = Fragment;
exports.Icon = Icon;
exports.List = List;
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
exports.isNovaElement = isNovaElement;
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
exports.renderComponent = renderComponent2;
exports.serializeElement = serializeElement;
exports.shortcut = shortcut;
exports.showNotification = showNotification;
exports.storageGet = storageGet;
exports.storageKeys = storageKeys;
exports.storageRemove = storageRemove;
exports.storageSet = storageSet;
exports.unmount = unmount;
exports.useCallback = useCallback;
exports.useEffect = useEffect;
exports.useId = useId;
exports.useMemo = useMemo;
exports.useNavigation = useNavigation;
exports.useReducer = useReducer;
exports.useRef = useRef;
exports.useState = useState;
//# sourceMappingURL=index.cjs.map
//# sourceMappingURL=index.cjs.map