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

export { Fragment, isNovaElement, jsx, jsxDEV, jsxs, serializeElement };
//# sourceMappingURL=jsx-runtime.js.map
//# sourceMappingURL=jsx-runtime.js.map