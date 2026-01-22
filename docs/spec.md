# Nova Cross-Platform Migration Specification

## Overview

Migrate Nova from GTK3 (Linux-only) to iced (cross-platform) while maintaining feature parity and adding Windows/macOS support.

## Target Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    iced UI Layer                            │
│  (NovaApp, SearchInput, ResultList, SettingsWindow)        │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                     Core Engine                             │
│  • Fuzzy search          • Config management                │
│  • Calculator            • Extension system                 │
│  • File search           • Result ranking                   │
│  • Emoji picker          • Command mode                     │
│  • Unit converter        • Clipboard history                │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                   Platform Abstraction                      │
│                                                             │
│  pub trait Platform {                                       │
│      fn discover_apps(&self) -> Vec<AppEntry>;             │
│      fn register_hotkey(&mut self, key: HotKey) -> Result; │
│      fn clipboard(&self) -> &dyn Clipboard;                │
│      fn open_url(&self, url: &str) -> Result;              │
│      fn open_file(&self, path: &Path) -> Result;           │
│      fn notify(&self, title: &str, body: &str) -> Result;  │
│      fn system_command(&self, cmd: SystemCmd) -> Result;   │
│      fn config_dir(&self) -> PathBuf;                      │
│      fn data_dir(&self) -> PathBuf;                        │
│  }                                                         │
└──────────────────────────┬──────────────────────────────────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
    ┌────▼────┐      ┌─────▼─────┐     ┌─────▼─────┐
    │  Linux  │      │   macOS   │     │  Windows  │
    │ Platform│      │  Platform │     │  Platform │
    └─────────┘      └───────────┘     └───────────┘
```

## Key Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| iced | 0.14 | Cross-platform GUI framework |
| global-hotkey | 0.7 | Global hotkey registration (all platforms) |
| arboard | 3.6 | Cross-platform clipboard |
| interprocess | 2 | Cross-platform IPC for single instance |
| notify-rust | 4 | Cross-platform notifications |
| directories | 5 | XDG-compatible paths on all platforms |

## Platform-Specific Implementations

### Linux

| Feature | Implementation |
|---------|----------------|
| App Discovery | Parse XDG .desktop files from /usr/share/applications, ~/.local/share/applications, Flatpak/Snap paths |
| Open URL/File | `xdg-open` command |
| Notifications | `notify-send` or notify-rust |
| System Commands | `loginctl lock-session`, `systemctl suspend/reboot/poweroff` |
| Config Directory | `~/.config/nova/` (XDG_CONFIG_HOME) |
| Data Directory | `~/.local/share/nova/` (XDG_DATA_HOME) |

### macOS

| Feature | Implementation |
|---------|----------------|
| App Discovery | Scan /Applications, ~/Applications, use `mdfind` for Spotlight |
| Open URL/File | `open` command |
| Notifications | notify-rust (NSUserNotificationCenter) |
| System Commands | `osascript` for various, `pmset sleepnow`, `launchctl reboot` |
| Config Directory | `~/Library/Application Support/nova/` |
| Data Directory | `~/Library/Application Support/nova/` |

### Windows

| Feature | Implementation |
|---------|----------------|
| App Discovery | Start Menu folders, Registry `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths`, UWP packages |
| Open URL/File | `ShellExecuteW` API or `start` command |
| Notifications | windows-rs toast notifications |
| System Commands | `LockWorkStation`, `ExitWindowsEx`, `shutdown.exe` |
| Config Directory | `%APPDATA%\nova\` |
| Data Directory | `%LOCALAPPDATA%\nova\` |

## iced Window Configuration

```rust
window::Settings {
    size: iced::Size::new(600.0, 400.0),
    position: window::Position::Centered,  // Or saved position
    decorations: false,                     // Borderless
    transparent: true,                      // For rounded corners/opacity
    level: window::Level::AlwaysOnTop,
    resizable: false,
    exit_on_close_request: false,          // Hide instead of close
    ..Default::default()
}
```

## Message/Event Flow (Elm Architecture)

```rust
pub enum Message {
    // Input events
    SearchChanged(String),
    KeyPressed(Key),

    // Navigation
    SelectNext,
    SelectPrevious,
    Execute,

    // Window management
    ToggleWindow,
    HideWindow,
    ShowWindow,

    // Mode changes
    EnterCommandMode(Extension),
    ExitCommandMode,

    // Async results
    AppsLoaded(Vec<AppEntry>),
    ClipboardPolled(Option<String>),
    ExecutionComplete(Result<(), String>),

    // Settings
    OpenSettings,
    ThemeChanged(Theme),
    ConfigSaved,
}
```

## Migration Strategy

### Phase 1-2: Foundation (Can ship as Linux-only)
- Create platform abstraction layer
- Extract core engine
- Linux platform implementation stays functional

### Phase 3-4: iced UI (Linux first)
- Implement iced UI targeting Linux
- Feature flag to switch between GTK and iced: `--features iced`
- Validate parity with GTK version

### Phase 5-6: Cross-Platform (Parallel)
- macOS and Windows implementations can be developed in parallel
- Each platform can be contributed independently

### Phase 7-10: Polish & Release
- Clipboard, settings, testing, packaging
- Release v0.2.0 with cross-platform support

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| iced API instability | Pin to 0.14, abstract iced-specific code behind internal traits |
| Window focus issues on macOS | Use accessibility APIs if needed, test early |
| Windows hotkey conflicts | Allow configurable hotkey, handle registration failures gracefully |
| App discovery edge cases | Fall back to empty list, let users report missing apps |
| Performance regression | Profile early, especially app indexing and fuzzy search |

## Testing Strategy

1. **Unit tests**: Core search logic, config parsing, calculator, units
2. **Integration tests**: Platform trait implementations (mocked and real)
3. **Manual testing matrix**:
   - Linux: Ubuntu, Fedora, Arch (X11 and Wayland)
   - macOS: Intel and Apple Silicon
   - Windows: Windows 10, Windows 11
