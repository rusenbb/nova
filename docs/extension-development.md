# Extension Development Guide

This guide covers everything you need to know to build Nova extensions.

## Quick Start

### 1. Create a new extension

```bash
nova create extension my-extension
cd my-extension
npm install
```

This generates:
```
my-extension/
├── nova.toml        # Extension manifest
├── package.json     # npm dependencies
├── tsconfig.json    # TypeScript config
├── src/
│   └── index.tsx    # Entry point
└── assets/
    └── icon.png     # Extension icon
```

### 2. Development workflow

```bash
# Start dev server with hot reload
nova dev

# Build for distribution
nova build
```

### 3. Install and test

```bash
# Install locally
nova install .

# Or install from a directory
nova install /path/to/my-extension
```

---

## Extension Manifest (nova.toml)

The manifest defines your extension's metadata, commands, and permissions.

```toml
[extension]
name = "my-extension"         # Unique identifier (lowercase, hyphens)
title = "My Extension"        # Display name
description = "Does amazing things"
version = "1.0.0"
icon = "star.fill"            # SF Symbols name or path to icon

# Commands exposed to Nova's search
[[commands]]
name = "main"                 # Command identifier
title = "Do Something"        # Display name in search
description = "Description shown in search results"
keywords = ["alias", "tags"]  # Additional search terms
mode = "list"                 # "list", "detail", or "form"
hasArgument = false           # Whether command accepts free-form input

# Another command
[[commands]]
name = "settings"
title = "Extension Settings"
mode = "form"

# Permissions (request only what you need)
[permissions]
clipboard = true              # Read/write clipboard
storage = true                # Persistent key-value storage (always granted)
network = ["api.example.com"] # Allowed domains for fetch
system = true                 # Open URLs, show notifications
filesystem = ["/tmp"]         # Read paths (not yet implemented)
```

---

## Components

Nova extensions render JSX components. There are three root component types:

### List

Display a searchable list of items.

```tsx
import { List, Icon, Accessory, createAction, createActionPanel } from "@aspect/nova";

function MyList() {
  const items = [
    { id: "1", title: "First Item", subtitle: "Description" },
    { id: "2", title: "Second Item", subtitle: "Another one" },
  ];

  return (
    <List searchBarPlaceholder="Search items...">
      {items.map((item) => (
        <List.Item
          id={item.id}
          title={item.title}
          subtitle={item.subtitle}
          icon={Icon.system("star")}
          accessories={[Accessory.text("Tag")]}
          keywords={["extra", "search", "terms"]}
          actions={createActionPanel("Actions", [
            createAction({
              id: "open",
              title: "Open",
              icon: Icon.system("arrow.right"),
              shortcut: { key: "return", modifiers: [] },
              onAction: `open:${item.id}`,
            }),
          ])}
        />
      ))}
    </List>
  );
}
```

#### List.Item Props

| Prop | Type | Description |
|------|------|-------------|
| `id` | `string` | Unique identifier |
| `title` | `string` | Primary text |
| `subtitle` | `string?` | Secondary text |
| `icon` | `Icon?` | Leading icon |
| `accessories` | `Accessory[]?` | Trailing badges/text |
| `keywords` | `string[]?` | Extra search terms |
| `actions` | `ActionPanel?` | Context menu actions |

### Detail

Show rich content with metadata.

```tsx
import { Detail, Icon } from "@aspect/nova";

function MyDetail() {
  return (
    <Detail
      markdown={`
# Title

Some **markdown** content here.

- Bullet point
- Another point
      `}
      metadata={[
        { label: "Status", value: "Active" },
        { label: "Created", value: "2024-01-15" },
      ]}
      actions={createActionPanel("Actions", [
        createAction({
          id: "copy",
          title: "Copy",
          icon: Icon.system("doc.on.doc"),
          onAction: "copy",
        }),
      ])}
    />
  );
}
```

### Form

Collect user input.

```tsx
import { Form } from "@aspect/nova";

function MyForm() {
  return (
    <Form onSubmit="submit">
      <Form.TextField
        id="name"
        title="Name"
        placeholder="Enter your name"
        validation={{ required: true, minLength: 2 }}
      />
      <Form.TextField
        id="email"
        title="Email"
        placeholder="you@example.com"
        validation={{ pattern: "^[^@]+@[^@]+$" }}
      />
      <Form.Checkbox
        id="subscribe"
        title="Subscribe to newsletter"
        defaultValue={true}
      />
      <Form.Dropdown
        id="priority"
        title="Priority"
        options={[
          { value: "low", label: "Low" },
          { value: "medium", label: "Medium" },
          { value: "high", label: "High" },
        ]}
      />
    </Form>
  );
}
```

#### Form Field Types

- `Form.TextField` - Single-line text input
- `Form.TextArea` - Multi-line text input
- `Form.Checkbox` - Boolean toggle
- `Form.Dropdown` - Select from options
- `Form.DatePicker` - Date selection

---

## Icons

Use SF Symbols (macOS) or icon paths.

```tsx
import { Icon } from "@aspect/nova";

// SF Symbol
Icon.system("star.fill")
Icon.system("doc.text")
Icon.system("arrow.right.circle")

// Custom icon from assets
Icon.asset("my-icon.png")
```

[SF Symbols Browser](https://developer.apple.com/sf-symbols/)

---

## Actions

Actions appear in the action panel (⌘K) and can have keyboard shortcuts.

```tsx
import { createAction, createActionPanel } from "@aspect/nova";

const panel = createActionPanel("Title", [
  createAction({
    id: "primary",
    title: "Primary Action",
    icon: Icon.system("return"),
    shortcut: { key: "return", modifiers: [] },
    onAction: "primary",
  }),
  createAction({
    id: "copy",
    title: "Copy to Clipboard",
    icon: Icon.system("doc.on.doc"),
    shortcut: { key: "c", modifiers: ["cmd"] },
    onAction: "copy",
  }),
  createAction({
    id: "delete",
    title: "Delete",
    icon: Icon.system("trash"),
    style: "destructive",
    shortcut: { key: "backspace", modifiers: ["cmd"] },
    onAction: "delete",
  }),
]);
```

### Shortcut Modifiers

- `cmd` - Command (⌘)
- `opt` - Option (⌥)
- `ctrl` - Control (⌃)
- `shift` - Shift (⇧)

---

## API Reference

### State Management

```tsx
import { useState } from "@aspect/nova";

function Counter() {
  const [count, setCount] = useState(0);

  // Update triggers re-render
  const increment = () => setCount(count + 1);

  return <List>...</List>;
}
```

### Storage

Persist data between sessions. Always available (no permission needed).

```tsx
import { storageGet, storageSet, storageRemove, storageKeys } from "@aspect/nova";

// Get value (returns undefined if not found)
const value = storageGet<MyType>("key");

// Set value (any JSON-serializable data)
storageSet("key", { foo: "bar" });

// Remove key
storageRemove("key");

// List all keys
const keys = storageKeys();
```

### Clipboard

Requires `clipboard = true` in permissions.

```tsx
import { clipboardCopy, clipboardRead } from "@aspect/nova";

// Copy text
clipboardCopy("Hello, world!");

// Read text (may be empty)
const text = clipboardRead();
```

### HTTP Fetch

Requires domain in `permissions.network`.

```tsx
import { fetch } from "@aspect/nova";

const response = await fetch("https://api.example.com/data", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ query: "test" }),
});

console.log(response.status);  // 200
console.log(response.body);    // Response text
```

### System

Requires `system = true` in permissions.

```tsx
import { openUrl, openPath, notify, closeWindow } from "@aspect/nova";

// Open URL in browser
openUrl("https://example.com");

// Open file/folder
openPath("/Users/me/Documents");

// Show notification
notify("Title", "Body text");

// Close Nova window
closeWindow();
```

### Navigation

For multi-view extensions.

```tsx
import { navigationPush, navigationPop, navigationDepth, render } from "@aspect/nova";

// Push a new view
navigationPush(<DetailView item={item} />);

// Go back
navigationPop();

// Check depth
if (navigationDepth() > 0) {
  // Show back button
}
```

### Preferences

Access user-configured settings defined in manifest.

```tsx
import { preferencesGet, preferencesAll } from "@aspect/nova";

// Get single preference
const apiKey = preferencesGet<string>("apiKey");

// Get all preferences
const prefs = preferencesAll();
```

Define preferences in nova.toml:

```toml
[[preferences]]
name = "apiKey"
title = "API Key"
description = "Your API key for the service"
type = "string"
required = true

[[preferences]]
name = "maxResults"
title = "Max Results"
type = "number"
default = 10
```

---

## Event Handling

Actions trigger events via `onAction` strings. Handle them in your command registration:

```tsx
registerCommand("main", (props) => {
  render(() => <MyList />);
});

// The onAction string is passed back when user triggers the action
// Parse it to determine what to do:
// - "copy:item-123" -> copy item with id "item-123"
// - "delete:item-123" -> delete item
```

Event handling is currently string-based. The action string from `onAction` is passed to your extension when the user triggers it.

---

## Best Practices

### 1. Request minimal permissions

Only request permissions you actually need. Users see what permissions you request.

### 2. Handle empty states

Always show helpful UI when there's no data:

```tsx
if (items.length === 0) {
  return (
    <List>
      <List.Item
        id="empty"
        title="No items yet"
        subtitle="Create your first item to get started"
      />
    </List>
  );
}
```

### 3. Use keywords for discoverability

Add relevant keywords so users can find your commands:

```toml
[[commands]]
keywords = ["github", "gh", "repo", "repository", "pr", "pull request"]
```

### 4. Provide keyboard shortcuts

Power users expect keyboard shortcuts for common actions:

```tsx
createAction({
  shortcut: { key: "c", modifiers: ["cmd"] },  // ⌘C
  ...
})
```

### 5. Close window after actions

Don't leave Nova open after completing an action:

```tsx
const handleCopy = () => {
  clipboardCopy(content);
  closeWindow();  // Clean up
};
```

---

## CLI Reference

```bash
# Create new extension
nova create extension <name>

# Development with hot reload
nova dev [path]

# Build for distribution
nova build [path]

# Install extension
nova install <source>
# source: local path, URL, or github:user/repo
```

---

## Troubleshooting

### Extension not showing up

1. Check `nova.toml` syntax (run `nova build` to validate)
2. Verify extension is in `~/.nova/extensions/`
3. Restart Nova

### Permission denied errors

Add the required permission to `nova.toml`:

```toml
[permissions]
clipboard = true
network = ["api.example.com"]
```

### Build errors

```bash
# Check TypeScript errors
npm run build

# Verify SDK is linked
npm list @aspect/nova
```

### Hot reload not working

1. Ensure `nova dev` is running
2. Check terminal for errors
3. Save the file to trigger rebuild

---

## Example Extensions

### Quick Notes
A note-taking extension with search, create, and delete.
See: `sample-extensions/quick-notes/`

### GitHub Search (coming soon)
Search GitHub repositories and issues.

### Calculator (coming soon)
Inline calculator with history.
