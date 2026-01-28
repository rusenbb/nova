# Type Synchronization Guide

Nova uses both Rust (core) and TypeScript (SDK) to define types that cross the IPC boundary. This guide explains how to keep them in sync.

## Type Locations

| Type Category | Rust Location | TypeScript Location |
|---------------|---------------|---------------------|
| IPC Types | `src/extensions/ipc/types.rs` | `packages/nova-sdk/src/types/api.ts` |
| Components | `src/extensions/components/*.rs` | `packages/nova-sdk/src/types/*.ts` |
| Manifest | `src/extensions/manifest.rs` | N/A (TOML, not typed in TS) |

## Naming Conventions

| Rust | TypeScript | Example |
|------|------------|---------|
| `snake_case` fields | `camelCase` fields | `max_results` → `maxResults` |
| `PascalCase` types | `PascalCase` types | `FetchResponse` → `FetchResponse` |
| `SCREAMING_SNAKE` | `UPPER_CASE` | `DEFAULT_TIMEOUT` → `DEFAULT_TIMEOUT` |

Serde automatically handles the case conversion via `#[serde(rename_all = "camelCase")]`.

## Adding a New IPC Type

### 1. Define in Rust

```rust
// src/extensions/ipc/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MyNewType {
    pub required_field: String,
    pub optional_field: Option<i32>,
    pub nested: NestedType,
}
```

### 2. Mirror in TypeScript

```typescript
// packages/nova-sdk/src/types/api.ts
export interface MyNewType {
  requiredField: string;
  optionalField?: number;
  nested: NestedType;
}
```

### 3. Add Type Mapping

Update `scripts/check-types.ts`:

```typescript
const TYPE_MAPPINGS: TypeMapping[] = [
  // ...existing mappings...
  {
    rustFile: "src/extensions/ipc/types.rs",
    rustType: "MyNewType",
    tsFile: "packages/nova-sdk/src/types/api.ts",
    tsType: "MyNewType",
  },
];
```

### 4. Verify

```bash
npx ts-node scripts/check-types.ts
```

## Type Mapping Rules

### Primitives

| Rust | TypeScript |
|------|------------|
| `String` | `string` |
| `i32`, `i64`, `u32`, `u64` | `number` |
| `f32`, `f64` | `number` |
| `bool` | `boolean` |
| `()` | `void` |

### Collections

| Rust | TypeScript |
|------|------------|
| `Vec<T>` | `T[]` |
| `HashMap<K, V>` | `Record<K, V>` |
| `Option<T>` | `T \| undefined` or `T?` |
| `Result<T, E>` | `T` (errors thrown) |

### Enums

Rust enums map to TypeScript discriminated unions:

```rust
// Rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Component {
    List(ListComponent),
    Detail(DetailComponent),
    Form(FormComponent),
}
```

```typescript
// TypeScript
export type ComponentData =
  | { type: "list" } & ListData
  | { type: "detail" } & DetailData
  | { type: "form" } & FormData;
```

## Component Types

Components are the most complex types. See the full mapping in:

- Rust: `src/extensions/components/mod.rs`
- TypeScript: `packages/nova-sdk/src/types/component.ts`

### List Component

```rust
// Rust (src/extensions/components/list.rs)
pub struct ListComponent {
    pub search_bar_placeholder: Option<String>,
    pub items: Vec<ListItem>,
}

pub struct ListItem {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: Option<Icon>,
    pub accessories: Vec<Accessory>,
    pub keywords: Vec<String>,
    pub actions: Option<ActionPanel>,
}
```

```typescript
// TypeScript (packages/nova-sdk/src/types/list.ts)
export interface ListData {
  type: "list";
  searchBarPlaceholder?: string;
  items: ListItemData[];
}

export interface ListItemData {
  id: string;
  title: string;
  subtitle?: string;
  icon?: IconData;
  accessories?: AccessoryData[];
  keywords?: string[];
  actions?: ActionPanelData;
}
```

## Validation

The SDK validates component data at runtime before sending to Nova:

```typescript
// packages/nova-sdk/src/components/validate.ts
export function validateComponent(data: ComponentData): void {
  // Type-specific validation
}
```

## Best Practices

1. **Always use `#[serde(rename_all = "camelCase")]`** on Rust structs
2. **Keep optional fields aligned** - `Option<T>` in Rust = `T?` in TypeScript
3. **Test both directions** - Rust can read TS output, TS can read Rust output
4. **Run type check before commits** that modify IPC types
5. **Document breaking changes** in CHANGELOG.md

## Common Issues

### Field name mismatch

```
❌ FetchRequest (Rust) vs FetchOptions (TS)
   Missing in TypeScript: time_out
```

Fix: Add `#[serde(rename_all = "camelCase")]` to Rust struct, or rename TS field.

### Optional vs required mismatch

Rust `Option<T>` must map to TypeScript `T?` or `T | undefined`.

### Enum variant mismatch

Ensure Rust enum variants use `#[serde(rename_all = "camelCase")]` and TypeScript uses the same casing.
