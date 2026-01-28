import * as react from 'react';
export { JSX } from 'react';
import ReactJSXRuntime from 'react/jsx-runtime';

declare const jsx: typeof ReactJSXRuntime.jsx;
declare const jsxs: typeof ReactJSXRuntime.jsxs;
declare const Fragment: react.ExoticComponent<{
    children?: react.ReactNode | undefined;
}>;
declare const jsxDEV: typeof ReactJSXRuntime.jsx;

export { Fragment, jsx, jsxDEV, jsxs };
