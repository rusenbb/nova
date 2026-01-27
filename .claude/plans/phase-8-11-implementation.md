# Implementation Plan: Phases 8-11

## Executive Summary

Based on deep research, here's the prioritized implementation plan for making Nova production-ready.

---

## Phase Priority Order (Recommended)

1. **Phase 9: Frecency** - Highest user impact, builds on existing code
2. **Phase 11: Window Management** - Types exist, needs platform code
3. **Phase 8: macOS Polish** - Production quality for native frontend
4. **Phase 10: Extension Registry** - Largest effort, needs infrastructure

---

## Phase 9: Frecency Enhancement (2-3 days)

### Current State
- ✅ Algorithm implemented (Firefox exponential decay)
- ✅ FFI integration exists
- ✅ Search engine accepts frecency parameter
- ❌ No result-type-specific weighting
- ❌ No GTK integration
- ❌ No user controls (reset, boost, stats)

### Implementation Tasks

#### 9.1 Add Result-Type Weighting
**File:** `src/services/frecency.rs`

```rust
// Add kind-specific multipliers
const KIND_WEIGHTS: [(ResultKind, f64); 8] = [
    (ResultKind::App, 1.0),        // Standard weighting
    (ResultKind::Script, 1.2),     // Boost scripts (reuse pattern)
    (ResultKind::Alias, 0.8),      // Slightly less weight
    (ResultKind::Quicklink, 0.6),  // Decay faster
    (ResultKind::Command, 0.7),    // System commands
    (ResultKind::Extension, 0.9),  // Similar to apps
    (ResultKind::File, 0.4),       // Files change often
    (ResultKind::Clipboard, 0.0),  // Don't track
];
```

#### 9.2 Add FFI Management Functions
**File:** `src/ffi.rs`

New functions:
- `nova_core_get_frecency_stats()` - Return usage statistics
- `nova_core_get_top_frecency(limit)` - Top N items by score
- `nova_core_clear_frecency()` - Reset all usage data
- `nova_core_boost_frecency(id, multiplier)` - Manual boost

#### 9.3 Add Helper Methods to FrecencyData
**File:** `src/services/frecency.rs`

```rust
pub fn top_by_score(&self, limit: usize) -> Vec<(&String, f64)>
pub fn clear(&mut self)
pub fn boost(&mut self, id: &str, multiplier: f64)
pub fn penalize(&mut self, id: &str, divisor: f64)
```

#### 9.4 GTK Integration (Future)
- Settings panel with usage statistics
- "Reset history" button
- Top-used items display

---

## Phase 11: Window Management (3-4 days)

### Current State
- ✅ Types defined: `WindowInfo`, `WindowFrame`, `WindowPosition`, `ScreenInfo`
- ✅ Search commands exist in `core/search.rs`
- ✅ FFI stubs exist in `ffi.rs`
- ❌ Platform implementations are stubs

### Implementation Tasks

#### 11.1 macOS Implementation
**File:** `src/platform/macos.rs`

Use AppleScript via `osascript` for cross-version compatibility:

```rust
fn get_focused_window(&self) -> Result<WindowInfo, String> {
    // AppleScript to get frontmost window
}

fn set_window_frame(&self, window_id: u64, frame: WindowFrame) -> Result<(), String> {
    // AppleScript to set position + size
}

fn list_windows(&self) -> Result<Vec<WindowInfo>, String> {
    // AppleScript to enumerate windows
}

fn list_screens(&self) -> Result<Vec<ScreenInfo>, String> {
    // NSScreen via AppleScript or system_profiler
}
```

**Permission:** Accessibility permission required - add check in AppDelegate.swift

#### 11.2 Linux Implementation
**File:** `src/platform/linux.rs`

X11 via wmctrl (most compatible):

```rust
fn get_focused_window(&self) -> Result<WindowInfo, String> {
    // xdotool getactivewindow + wmctrl -l
}

fn set_window_frame(&self, window_id: u64, frame: WindowFrame) -> Result<(), String> {
    // wmctrl -i -r <id> -e 0,x,y,w,h
}
```

Wayland detection for graceful degradation.

#### 11.3 Windows Implementation (Future)
**File:** `src/platform/windows.rs`

Win32 API via `windows` crate:
- `GetForegroundWindow`, `SetWindowPos`, `EnumWindows`

#### 11.4 Wire Up Execution
**File:** `src/ffi.rs`

Handle `ExecutionAction::SetWindowPosition` in `result_to_action()`.

---

## Phase 8: macOS Polish (4-5 days)

### Critical Issues Found

| Issue | File | Priority |
|-------|------|----------|
| Permission dialog uses ext ID, not title | AppDelegate.swift:106 | High |
| 15+ hardcoded spacing values | Multiple | High |
| No accessibility descriptors | All custom views | High |
| Basic markdown (no lists/tables) | ExtensionDetailView.swift | Medium |
| Thread safety in SearchPanel | SearchPanel.swift | Medium |
| WKWebView private API usage | ExtensionDetailView.swift:148 | Medium |

### Implementation Tasks

#### 8.1 Fix Permission Dialog Title
**File:** `frontends/macos/Nova/AppDelegate.swift`

```swift
// Line 106: Get title from manifest
let title = await novaCore.getExtensionTitle(extensionId) ?? extensionId
```

#### 8.2 Extract Hardcoded Values to Theme
Create theme extension methods:
```swift
extension Theme {
    var panelWidth: CGFloat { 620 }
    var panelHeight: CGFloat { 400 }
    var rowHeight: CGFloat { 48 }
    var searchFieldHeight: CGFloat { 28 }
    // ... etc
}
```

Update all hardcoded values to use `theme.*`.

#### 8.3 Add Accessibility Descriptors
All custom NSView subclasses need:
```swift
override func accessibilityRole() -> NSAccessibility.Role? { .list }
override func accessibilityLabel() -> String? { "Search results" }
```

#### 8.4 Improve Markdown Rendering
Options:
1. Use SwiftMarkdown package (recommended)
2. Or extend current regex parser with lists, tables

#### 8.5 Thread Safety
Add `DispatchQueue.main.async` guards in all callbacks that mutate state.

---

## Phase 10: Extension Registry (6+ weeks)

### Architecture Summary

```
┌─────────────────────────────────────────────────────┐
│  Nova Registry Server (Rust + axum)                 │
├─────────────────────────────────────────────────────┤
│  PostgreSQL (publishers, extensions, versions)      │
│  S3/R2 (tarball storage)                           │
│  GitHub OAuth (authentication)                      │
│  Security Scanner (static analysis + Snyk)          │
└─────────────────────────────────────────────────────┘
```

### API Endpoints (Already Defined in Nova)

```
GET  /api/v1/extensions/search?q=...
GET  /api/v1/extensions/{pub}/{name}
GET  /api/v1/extensions/{pub}/{name}/download
POST /api/v1/extensions (authenticated)
GET  /api/v1/updates
```

### Implementation Order

1. **Week 1-2:** Database schema + GitHub OAuth
2. **Week 2-3:** Search + discovery endpoints
3. **Week 3-4:** Publish endpoint + validation
4. **Week 4-5:** Security scanning pipeline
5. **Week 5-6:** Testing, documentation, deployment

### Hosting Recommendation

**Fly.io** with:
- 1x Web service (axum): ~$5/month
- 1x PostgreSQL: ~$7/month
- Cloudflare R2 for storage: ~$5/month (no egress)

Total: ~$15-20/month

---

## Implementation Order

### Week 1: Phase 9 (Frecency)
- Day 1: Add kind weights + helper methods
- Day 2: Add FFI management functions
- Day 3: Testing + documentation

### Week 2: Phase 11 (Window Management)
- Day 1-2: macOS implementation (AppleScript)
- Day 3: Linux implementation (wmctrl)
- Day 4: Testing + permission handling

### Week 3-4: Phase 8 (macOS Polish)
- Day 1: Permission dialog fix + theme extraction
- Day 2-3: Accessibility descriptors
- Day 4-5: Markdown improvements + thread safety

### Week 5-10: Phase 10 (Extension Registry)
- Separate project: `nova-registry/`
- Requires backend infrastructure setup

---

## Starting Point: Phase 9

Since Phase 9 builds directly on existing code and has highest immediate user impact, I recommend starting there.

Tasks:
1. Add `KIND_WEIGHTS` to frecency.rs
2. Add helper methods (top_by_score, clear, boost)
3. Add FFI functions for native frontend control
4. Test with existing macOS frontend
