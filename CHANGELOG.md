# Changelog

All notable changes to Nova will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Permission system for Nova extensions (clipboard, network, filesystem, system, storage)
- Background execution system for extensions with battery-aware throttling
- Unified dark mode theme system with `theme.toml` as single source of truth
- GTK frontend now uses library types for better maintainability

### Changed
- Refactored GTK main.rs to use library `SearchResult` instead of local enum

### Fixed
- Extension execution JSON serialization mismatch between Rust and Swift
- GTK clipboard polling now uses platform trait API

## [0.1.0] - 2024-01-24

### Added
- **Core Features**
  - App launcher with fuzzy search
  - Quicklinks with URL templates and `{query}` substitution
  - Aliases for custom app shortcuts
  - Scripts with `@argument`, `@output` directives
  - Calculator with inline math evaluation
  - Clipboard history
  - Command mode (keyword + space)

- **Extension System**
  - Deno-based JavaScript/TypeScript runtime
  - TypeScript SDK (`@aspect/nova`)
  - Component system (List, Detail, Form)
  - IPC protocol for clipboard, storage, preferences, fetch, system, navigation, render
  - Event dispatch for UI callbacks

- **CLI Tools**
  - `nova create extension` - Scaffold new extensions
  - `nova dev` - Hot reload development server
  - `nova build` - Bundle for distribution
  - `nova install` - Install from path, URL, or GitHub

- **Platforms**
  - macOS native frontend (Swift/AppKit) with menu bar app
  - Linux GTK frontend
  - Global hotkey support (Alt+Space)

- **Sample Extensions**
  - Quick Notes - Simple note-taking extension

### Infrastructure
- Cross-platform CI/CD with GitHub Actions
- Release automation for macOS DMG and Linux AppImage
- Performance benchmarks with Criterion

[Unreleased]: https://github.com/rusenbb/nova/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/rusenbb/nova/releases/tag/v0.1.0
