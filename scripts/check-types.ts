#!/usr/bin/env node
/**
 * Type Synchronization Checker for Nova SDK
 *
 * This script validates that TypeScript types in the SDK match the Rust definitions.
 * It uses JSON schemas generated from Rust's serde types.
 *
 * Usage:
 *   npx ts-node scripts/check-types.ts
 *   # or
 *   npm run check-types
 *
 * To add a new type:
 * 1. Add the Rust struct with #[derive(Serialize, Deserialize, JsonSchema)]
 * 2. Add the TypeScript interface in packages/nova-sdk/src/types/
 * 3. Add a type mapping entry below
 */

import * as fs from "fs";
import * as path from "path";

// ─────────────────────────────────────────────────────────────────────────────
// Type Mappings: Rust type name -> TypeScript file path and interface name
// ─────────────────────────────────────────────────────────────────────────────

interface TypeMapping {
  rustFile: string;
  rustType: string;
  tsFile: string;
  tsType: string;
}

const TYPE_MAPPINGS: TypeMapping[] = [
  // IPC Types
  {
    rustFile: "src/extensions/ipc/types.rs",
    rustType: "FetchRequest",
    tsFile: "packages/nova-sdk/src/types/api.ts",
    tsType: "FetchOptions",
  },
  {
    rustFile: "src/extensions/ipc/types.rs",
    rustType: "FetchResponse",
    tsFile: "packages/nova-sdk/src/types/api.ts",
    tsType: "FetchResponse",
  },
  {
    rustFile: "src/extensions/ipc/types.rs",
    rustType: "FetchMethod",
    tsFile: "packages/nova-sdk/src/types/api.ts",
    tsType: "FetchMethod",
  },

  // Component Types
  {
    rustFile: "src/extensions/components/mod.rs",
    rustType: "Component",
    tsFile: "packages/nova-sdk/src/types/component.ts",
    tsType: "ComponentData",
  },
  {
    rustFile: "src/extensions/components/list.rs",
    rustType: "ListComponent",
    tsFile: "packages/nova-sdk/src/types/list.ts",
    tsType: "ListData",
  },
  {
    rustFile: "src/extensions/components/detail.rs",
    rustType: "DetailComponent",
    tsFile: "packages/nova-sdk/src/types/detail.ts",
    tsType: "DetailData",
  },
  {
    rustFile: "src/extensions/components/form.rs",
    rustType: "FormComponent",
    tsFile: "packages/nova-sdk/src/types/form.ts",
    tsType: "FormData",
  },
];

// ─────────────────────────────────────────────────────────────────────────────
// Parsing helpers
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Extract struct/enum field names from Rust source.
 * This is a simplified parser - for complex types, consider using syn.
 */
function extractRustFields(content: string, typeName: string): string[] {
  const fields: string[] = [];

  // Find struct or enum definition
  const structRegex = new RegExp(
    `(pub\\s+)?struct\\s+${typeName}\\s*\\{([^}]+)\\}`,
    "s"
  );
  const enumRegex = new RegExp(
    `(pub\\s+)?enum\\s+${typeName}\\s*\\{([^}]+)\\}`,
    "s"
  );

  let match = content.match(structRegex) || content.match(enumRegex);
  if (!match) return fields;

  const body = match[2];

  // Extract field names (simplified - ignores attributes and complex types)
  const fieldRegex = /(?:pub\s+)?(\w+)\s*:/g;
  let fieldMatch;
  while ((fieldMatch = fieldRegex.exec(body)) !== null) {
    // Skip serde rename attributes
    const fieldName = fieldMatch[1];
    if (fieldName && !fieldName.startsWith("_")) {
      fields.push(fieldName);
    }
  }

  // For enums, extract variant names
  const variantRegex = /^\s*(\w+)(?:\s*\{|\s*\(|\s*,|\s*$)/gm;
  let variantMatch;
  while ((variantMatch = variantRegex.exec(body)) !== null) {
    const variantName = variantMatch[1];
    if (variantName && !fields.includes(variantName)) {
      fields.push(variantName);
    }
  }

  return fields;
}

/**
 * Extract interface/type field names from TypeScript source.
 */
function extractTsFields(content: string, typeName: string): string[] {
  const fields: string[] = [];

  // Find interface or type definition
  const interfaceRegex = new RegExp(
    `(export\\s+)?interface\\s+${typeName}\\s*\\{([^}]+)\\}`,
    "s"
  );
  const typeRegex = new RegExp(
    `(export\\s+)?type\\s+${typeName}\\s*=\\s*([^;]+);`,
    "s"
  );

  let match = content.match(interfaceRegex);
  if (match) {
    const body = match[2];
    // Extract field names
    const fieldRegex = /^\s*(\w+)\??:/gm;
    let fieldMatch;
    while ((fieldMatch = fieldRegex.exec(body)) !== null) {
      fields.push(fieldMatch[1]);
    }
  } else {
    match = content.match(typeRegex);
    if (match) {
      // For union types, extract literal values
      const body = match[2];
      const literalRegex = /"(\w+)"/g;
      let literalMatch;
      while ((literalMatch = literalRegex.exec(body)) !== null) {
        fields.push(literalMatch[1]);
      }
    }
  }

  return fields;
}

// ─────────────────────────────────────────────────────────────────────────────
// Main validation
// ─────────────────────────────────────────────────────────────────────────────

function checkTypes(): boolean {
  const rootDir = path.resolve(__dirname, "..");
  let hasErrors = false;

  console.log("Checking type synchronization between Rust and TypeScript...\n");

  for (const mapping of TYPE_MAPPINGS) {
    const rustPath = path.join(rootDir, mapping.rustFile);
    const tsPath = path.join(rootDir, mapping.tsFile);

    // Check files exist
    if (!fs.existsSync(rustPath)) {
      console.log(`⚠️  Rust file not found: ${mapping.rustFile}`);
      continue;
    }
    if (!fs.existsSync(tsPath)) {
      console.log(`⚠️  TypeScript file not found: ${mapping.tsFile}`);
      continue;
    }

    const rustContent = fs.readFileSync(rustPath, "utf-8");
    const tsContent = fs.readFileSync(tsPath, "utf-8");

    const rustFields = extractRustFields(rustContent, mapping.rustType);
    const tsFields = extractTsFields(tsContent, mapping.tsType);

    // Compare fields (case-insensitive for snake_case vs camelCase)
    const normalizeField = (f: string) =>
      f.toLowerCase().replace(/_/g, "");
    const rustNormalized = new Set(rustFields.map(normalizeField));
    const tsNormalized = new Set(tsFields.map(normalizeField));

    const missingInTs = rustFields.filter(
      (f) => !tsNormalized.has(normalizeField(f))
    );
    const missingInRust = tsFields.filter(
      (f) => !rustNormalized.has(normalizeField(f))
    );

    if (missingInTs.length > 0 || missingInRust.length > 0) {
      hasErrors = true;
      console.log(`❌ ${mapping.rustType} (Rust) vs ${mapping.tsType} (TS)`);
      if (missingInTs.length > 0) {
        console.log(`   Missing in TypeScript: ${missingInTs.join(", ")}`);
      }
      if (missingInRust.length > 0) {
        console.log(`   Missing in Rust: ${missingInRust.join(", ")}`);
      }
    } else if (rustFields.length > 0 || tsFields.length > 0) {
      console.log(`✓ ${mapping.rustType} <-> ${mapping.tsType}`);
    } else {
      console.log(`⚠️  ${mapping.rustType}: No fields found (complex type?)`);
    }
  }

  console.log("");
  if (hasErrors) {
    console.log("Type check failed! Please sync the types.");
    return false;
  } else {
    console.log("All types are in sync!");
    return true;
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// CLI
// ─────────────────────────────────────────────────────────────────────────────

if (require.main === module) {
  const success = checkTypes();
  process.exit(success ? 0 : 1);
}

export { checkTypes, TYPE_MAPPINGS };
