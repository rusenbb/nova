# Nova Session Notes

See `TASKS.md` for full roadmap.

## 2025-01-24
**Worked on:** Phase 2 - Extension Runtime + Event Dispatch

**State:** Extension system complete on `feature/native-core`
- `310e4c4` - Extension system with TypeScript SDK
- `ad1a2dc` - Quick Notes sample extension
- `cfc8cd3` - Event dispatch system

**Completed:**
- [x] 2.1–2.8 Deno embedding (isolate, host, LRU management)
- [x] 2.9–2.14 Extension loader (manifest, command indexing)
- [x] 2.15–2.24 IPC protocol (clipboard, storage, prefs, fetch, system, render, nav)
- [x] 2.25–2.28 Component renderer core (List, Detail, Form, ActionPanel)
- [x] 2.29–2.40 macOS renderer (Swift UI in `Extension/`)
- [x] 3.1–3.7 TypeScript SDK (JSX runtime, hooks, types)

**Next up (Phase 3 - Developer Experience):**
- [x] 3.9–3.13 `nova create extension` CLI scaffolding
- [x] 3.14–3.20 `nova dev` hot reload
- [x] 3.21–3.25 `nova build` bundling
- [x] 3.26–3.32 `nova install` from GitHub

**Also pending:**
- [ ] 2.47–2.52 Permissions system (consent dialogs)
- [ ] 2.53–2.57 Background execution
- [x] Create sample extension (`sample-extensions/quick-notes/`)

**Session 2 - Event Dispatch (cfc8cd3):**
- [x] 2.24 Event dispatch system implemented
  - JS: `__nova_dispatch_event` in runtime.js
  - Rust: `dispatch_event` on isolate and host
  - FFI: `nova_core_send_event` wired up
  - Swift: `NovaCore.sendEvent` ready to use
  - Flow: Swift UI -> FFI -> Rust Host -> JS Isolate -> callback -> updated component

**Known issues:**
- **Extension execution broken:** JSON serialization mismatch between Rust `ExecutionResult` and Swift `ExecuteResponse`
- SDK JSX types need alignment (component factories vs JSX elements)

## 2026-01-24
**Worked on:** Phase 3 - `nova create extension` CLI

**State:** CLI scaffolding complete
- Added `clap` + `dialoguer` for CLI parsing and prompts
- Created `src/cli/` module with create command
- Generates: nova.toml, src/index.tsx, tsconfig.json, package.json, assets/icon.png

**Implementation:**
- `src/cli/mod.rs` - CLI parser with subcommands (create, dev, build, install stubs)
- `src/cli/create.rs` - Extension scaffolding with interactive/non-interactive modes
- Integrated with main.rs (CLI runs before GTK init)
- Test via `cargo run --example cli_test create extension <name>`

**Next:** `nova dev` (3.14–3.20) - hot reload for extension development

---

**Session 2 - `nova dev` hot reload:**

**Implemented:**
- `src/cli/dev.rs` - Development server with file watching
- Uses `notify` crate for cross-platform file watching
- Uses `notify-debouncer-mini` for debouncing (300ms)
- Watches `src/` directory and `nova.toml`
- Runs `npm run build` (esbuild) on changes
- Validates manifest on nova.toml changes
- Graceful Ctrl+C handling with `ctrlc` crate

**Usage:**
```bash
# On macOS (without GTK)
cargo run --example nova_cli -- dev [path]

# On Linux (with GTK)
nova dev [path]
```

**Output:**
```
✓ Loaded manifest: Quick Notes
✓ Build successful
✓ Extension ready: quick-notes

Watching for changes...
Press Ctrl+C to stop.

[ File changed: index.tsx
→ Rebuilding... done
✓ Reload complete
```

**Fixed:**
- `mode = "view"` → `mode = "list"` in sample extension and create template

**Next:** `nova build` (3.21–3.25) - bundling for distribution

---

**Session 3 - `nova build` bundling:**

**Implemented:**
- `src/cli/build.rs` - Build command for distribution packaging
- Cleans and creates `dist/` directory
- Runs `npm run build` (esbuild) or direct esbuild fallback
- Copies `nova.toml` to dist
- Recursively copies `assets/` folder
- Shows file sizes and summary

**Output structure:**
```
dist/
├── nova.toml       # Manifest (copied)
├── index.js        # Bundled JS (single file)
└── assets/         # Assets (copied recursively)
```

**Usage:**
```bash
cargo run --example nova_cli -- build [path]
```

**Output:**
```
✓ Loaded manifest from nova.toml
✓ TypeScript compilation successful
✓ Bundled to dist/index.js (3.3KB)
✓ Copied nova.toml
✓ Copied assets (1 files)

Output: dist/
  ├── nova.toml
  ├── index.js (3.3KB)
  └── assets/ (1 files)

Ready for distribution!
```

**Next:** `nova install` (3.26–3.32) - install from GitHub

---

**Session 4 - `nova install` + Testing:**

**Implemented:**
- `src/cli/install.rs` - Install from local path, URL, or GitHub
- Parses sources: `github:user/repo`, URLs, local paths
- Clones/copies, builds if needed, installs to extensions dir

**SDK Setup:**
- SDK at `packages/nova-sdk/` - not published to npm yet
- Use `npm link` for local development:
```bash
cd packages/nova-sdk && npm link
cd my-ext && npm link @aspect/nova
```

**macOS Build Workflow:**
- Build via **Xcode** (open `frontends/macos/Nova.xcodeproj`)
- Xcode handles Rust library compilation and Swift frontend
- Hotkey: **Option + Space** (⌥ + Space)

**Known issues:**
- First launch after granting accessibility permission requires restart for hotkey to work
- SDK not published to npm - use `npm link` for now

---

## 2026-01-27
**Worked on:** Bug fixes, Permissions, Background Execution, UI Theme

**Commits on `feature/native-core`:**
- `d436e7e` Fix extension execution and add Deno command support in UI
- `b449346` Add CLI tools for extension development
- `c123bf0` Apply rustfmt and remove deprecated build script
- `fd5e4d8` Add permission system for Nova extensions
- `65d62b2` Add background execution system for extensions
- `ca56c4d` Add unified dark mode theme system
- `e29a3b6` (HEAD) CI fixes

**Completed:**
- [x] **Extension execution fix** - JSON serialization mismatch resolved
- [x] **2.47–2.52 Permissions system**
  - `PermissionSet` struct with clipboard, network, filesystem, system, storage
  - Permission checks in IPC handlers
  - macOS consent dialog (`PermissionConsentView.swift`)
  - Permissions manager UI for settings
  - FFI: grant/revoke/list permissions
  - Persistence to `~/.nova/permissions.json`
- [x] **2.53–2.57 Background execution**
  - `BackgroundScheduler` with tokio async
  - Battery-aware throttling (macOS: pmset, Linux: /sys/class/power_supply)
  - User toggle per-extension with persistence
  - FFI: enable/disable, power state, list background extensions
- [x] **Unified dark theme system**
  - `assets/theme.toml` - single source of truth for design tokens
  - `src/theme.rs` - Rust loader with compile-time embedding
  - `Theme.swift` - Swift wrapper with NSColor helpers
  - Updated SearchPanel, ExtensionListCell, ExtensionDetailView

**Parallel worktree workflow:**
- Used `git worktree` to create 3 branches in parallel
- Spawned 3 subagents to implement features concurrently
- Merged all back to `feature/native-core`

**CI Status:** ✅ Passing (library tests, macOS, Ubuntu, GTK clippy)

---

**Session 5 - GTK Refactoring (b24a27f):**
- [x] Refactored GTK main.rs to use library types
  - Replaced local `SearchResult` enum with `core::search::SearchResult`
  - Added `result_to_action()` helper function
  - Added `to_platform_app()` conversion for `services::AppEntry` → `platform::AppEntry`
  - Updated key handler to use Platform trait for app launching and system commands
  - Added handling for `DenoCommand` variants
- [x] Fixed GTK clippy warnings
  - Simplified alias matching logic (removed `if_same_then_else`)
  - Removed unnecessary borrow in `application_id()` call
  - Used `.flatten()` for IPC listener iteration
  - Fixed clipboard polling to use `poll_with_content()` API
- [x] Added `#[allow(dead_code)]` to library modules used by FFI but not GTK
  - `core/search.rs`, `executor.rs`, `error.rs`, `search.rs`, `platform/mod.rs`, `platform/linux.rs`
  - `services/app_index.rs`, `services/clipboard.rs`
- [x] Re-enabled GTK clippy in CI

---

## Remaining Work

**Phase 2 (Extension Runtime):**
- [x] 2.1–2.28 Core extension system
- [x] 2.29–2.40 macOS renderer
- [x] 2.41–2.46 Linux GTK renderer (refactored, now uses library types)
- [x] 2.47–2.52 Permissions system
- [x] 2.53–2.57 Background execution
- [ ] 2.58–2.62 Preferences editor UI

**Phase 3 (Developer Experience):**
- [x] 3.1–3.8 TypeScript SDK
- [x] 3.9–3.32 CLI tools (create, dev, build, install)
- [ ] 3.33–3.41 Documentation

**Phase 1 (Core Parity) - not started:**
- [ ] 1.1–1.7 Settings UI
- [ ] 1.8–1.12 Snippets
- [ ] 1.13–1.18 Window management
- [ ] 1.19–1.23 System commands
- [ ] 1.24–1.28 File search improvements
- [ ] 1.29–1.32 Search ranking (frecency)
- [ ] 1.33–1.36 Onboarding

**Phase 4 (Distribution):**
- [ ] 4.1–4.6 In-app extension browser
- [ ] 4.7–4.12 Central registry
- [ ] 4.13–4.16 Extension updates
- [ ] 4.17–4.22 Deep linking
- [ ] 4.27–4.30 Windows frontend

**Infrastructure:**
- [x] CI/CD (library, macOS, Ubuntu, GTK)
- [ ] Code signing (macOS notarization)
- [ ] Integration/E2E tests

**Technical debt:**
- [x] Refactor GTK main.rs to use library types
- [x] Unify duplicate AppEntry types
- [x] Clean up lib.rs public API
- [ ] Publish SDK to npm
- [x] Make GTK use library SearchEngine (eliminated 354 lines)
- [x] Extract GTK UI into modules (main.rs: 1229→93 lines)

---

## 2026-01-27 (Session 6) - Codebase Quality Refactoring

**Worked on:** Deep codebase analysis and quality improvements

**Commit:** `5fa0cb1` - Improve codebase quality and add open source files

**Analysis performed:**
- Spawned 6 explore agents to audit: architecture, performance, extension DX, macOS frontend, open source hygiene
- Identified 47 specific issues across 5 categories
- Created 8-phase refactoring plan

**Completed phases:**

| Phase | Description | Impact |
|-------|-------------|--------|
| **0** | Open source hygiene | +5 files (LICENSE, CONTRIBUTING, CHANGELOG, templates) |
| **1** | Unify AppEntry types | -42 lines, single source of truth |
| **2** | Search performance quick wins | Eliminated collect() allocations |
| **5** | thiserror for errors | -14 lines, idiomatic Rust |
| **6** | Clean lib.rs API | Better docs and re-exports |

**Key findings from analysis:**
- 40+ allocations per keystroke in search (to_lowercase in loops)
- GTK main.rs duplicates ~500 lines from SearchEngine
- 11 files needed to add a UI component (Rust + TS + Swift)
- Missing open source files blocking contributors

**Deferred phases (larger refactors):**
- [x] Phase 3: Make GTK use library SearchEngine (DONE - 354 lines removed)
- [x] Phase 4: Extract GTK UI into modules (DONE - main.rs: 1229→93 lines)
- Phase 7: Extension DX improvements (macros, type generation)
- Phase 8: macOS frontend polish

---

## 2026-01-27 (Session 7) - GTK Modularization

**Worked on:** Phases 3 and 4 from refactoring plan

**Commits on `feature/native-core`:**
- `d81b3e9` Refactor GTK to use library SearchEngine
- `e75e80c` Extract GTK UI into modular structure

**Phase 3 - SearchEngine Integration:**
- Replaced local `search_with_commands()` with `SearchEngine.search()`
- Replaced local `search_in_command_mode()` with `SearchEngine.search_in_command_mode()`
- Deleted `get_system_commands()` (now provided by SearchEngine)
- Added `entries()` method to AppIndex
- **Result:** -354 lines from main.rs

**Phase 4 - GTK Modularization:**
- Created `src/gtk/` module structure:
  - `mod.rs` (17 lines) - Re-exports
  - `state.rs` (291 lines) - UIState, CommandModeState, result_to_action
  - `window.rs` (562 lines) - build_ui, position_window, render_results_list
  - `exec.rs` (140 lines) - Execution helpers
  - `ipc.rs` (22 lines) - Socket communication
  - `shortcut.rs` (162 lines) - GNOME shortcut config
- main.rs reduced from 1229 to 93 lines (-1136 lines)
- Total GTK code: 1287 lines (organized across 7 files)

**CI Status:** Tests pass (96/96)

**Next:** Phase 7 (Extension DX) or Phase 8 (macOS polish)

---

## 2026-01-28 - Raycast Parity Assessment

**Fixed today:** V8 JIT entitlement, form submission callbacks, UI styling

### What's Good
- Architecture: Clean Rust core + FFI + native frontends
- Search: Frecency + fuzzy, <20ms
- Extension system: Deno isolates, TypeScript SDK, React JSX
- macOS frontend: 6K LOC Swift, polished

### What's Missing vs Raycast
- Settings UI (can't configure hotkey in-app)
- In-app extension store (CLI only)
- Window management UI
- Snippets
- AI integration
- Windows support
- Advanced forms (multiselect, file picker)

### What's Not Maintainable
- 11 files to add a UI component (needs codegen)
- GTK frontend outdated
- No integration tests
- SDK not published to npm

### Priority Order
1. Settings UI
2. In-app extension browser
3. Window management UI
4. Publish SDK to npm
5. Modernize GTK
6. Windows support

**Nova is ~60% to Raycast parity. Core is solid, needs UI/UX polish.**
