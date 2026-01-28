/**
 * Nova JSX Runtime
 *
 * Re-exports React's JSX runtime for use with the automatic JSX transform.
 *
 * Usage in tsconfig.json:
 *   "jsx": "react-jsx",
 *   "jsxImportSource": "@aspect/nova"
 */

import ReactJSXRuntime from "react/jsx-runtime";

// Re-export React's JSX runtime functions
export const jsx = ReactJSXRuntime.jsx;
export const jsxs = ReactJSXRuntime.jsxs;
export const Fragment = ReactJSXRuntime.Fragment;

// Development mode
export const jsxDEV = ReactJSXRuntime.jsx;

// Re-export JSX namespace types
export type { JSX } from "react";
