import { defineConfig } from "tsup";

export default defineConfig([
  // Main entry
  {
    entry: ["src/index.ts"],
    format: ["esm", "cjs"],
    dts: true,
    clean: true,
    sourcemap: true,
    splitting: false,
    treeshake: true,
    minify: false,
    target: "es2022",
  },
  // JSX runtime (separate to avoid DTS conflicts)
  {
    entry: ["src/jsx-runtime.ts"],
    format: ["esm", "cjs"],
    dts: false, // Skip DTS for jsx-runtime to avoid conflict
    clean: false,
    sourcemap: true,
    splitting: false,
    treeshake: true,
    minify: false,
    target: "es2022",
  },
]);
