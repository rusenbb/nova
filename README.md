# Nova

**A keyboard-driven productivity launcher for Linux and macOS.** Think [Raycast](https://raycast.com), but open source and cross-platform.

Nova helps you launch apps, run custom commands, and access your favorite websites — all without touching the mouse.

## Features

- **App Launcher** — Instantly search and launch any application
- **Quicklinks** — Open URLs with optional search queries (e.g., `yt cats` → YouTube search for "cats")
- **Aliases** — Create shortcuts to launch apps with custom keywords
- **Scripts** — Run custom shell scripts with arguments and capture output
- **Calculator** — Evaluate math expressions inline
- **Clipboard History** — Access recently copied items
- **Command Mode** — Type a keyword + space to enter focused search mode with visual feedback
- **Themes** — Multiple built-in themes (Catppuccin, Nord, Dracula, Gruvbox, Tokyo Night, One Dark)
- **Cross-Platform** — Works on Linux (X11) and macOS

## Installation

### Download

Download the latest release for your platform from the [Releases](https://github.com/rusenbb/nova/releases) page:

| Platform | Download |
|----------|----------|
| **Linux (x86_64)** | `Nova-x86_64.AppImage` or `nova-linux-x86_64.tar.gz` |
| **macOS (Apple Silicon)** | `nova-macos-aarch64.dmg` |
| **macOS (Intel)** | `nova-macos-x86_64.dmg` |

### Linux

#### AppImage (Recommended)
```bash
# Download and make executable
chmod +x Nova-x86_64.AppImage
./Nova-x86_64.AppImage
```

#### From Tarball
```bash
tar -xzf nova-linux-x86_64.tar.gz
sudo cp nova-linux-x86_64/nova /usr/local/bin/
```

### macOS

1. Download the `.dmg` file for your Mac (Apple Silicon or Intel)
2. Open the DMG and drag Nova to Applications
3. Right-click Nova and select "Open" (first time only, to bypass Gatekeeper)

### From Source

Prerequisites:
- Rust (latest stable)

#### Linux Dependencies
```bash
# Ubuntu/Debian
sudo apt install libxdo-dev libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev

# Fedora
sudo dnf install libxdo-devel libxcb-devel

# Arch
sudo pacman -S libxdo libxcb
```

#### Build
```bash
git clone https://github.com/rusenbb/nova.git
cd nova
cargo build --release --no-default-features --features iced-ui
sudo cp target/release/nova-iced /usr/local/bin/nova
```

## Usage

| Action | Key |
|--------|-----|
| Open Nova | `Alt+Space` |
| Navigate results | `↑` / `↓` |
| Launch selected | `Enter` |
| Enter command mode | `keyword` + `Space` or `Tab` on selection |
| Exit command mode | `Backspace` (empty) or `Escape` |
| Open settings | Type `settings` or `,` |
| Close | `Escape` |

### Quicklinks

Add search shortcuts in Settings → Quicklinks:

```
Keyword: yt
Name: YouTube Search
URL: https://youtube.com/results?search_query={query}
```

Now type `yt ` to enter YouTube search mode — your query goes directly to YouTube.

### Scripts

Place executable scripts in `~/.config/nova/scripts/`:

```bash
#!/bin/bash
# ~/.config/nova/scripts/ip.sh
# @name My IP
# @description Show current public IP
# @output notification

curl -s ifconfig.me
```

Scripts support:
- `@argument <name>` — Accept user input
- `@output notification|clipboard|silent` — Control output handling

## Configuration

Config file location:
- **Linux**: `~/.config/nova/config.toml`
- **macOS**: `~/Library/Application Support/nova/config.toml`

```toml
[general]
hotkey = "<Alt>space"

[appearance]
theme = "catppuccin-mocha"  # nord, dracula, gruvbox-dark, tokyo-night, one-dark
accent_color = "#cba6f7"
opacity = 0.95
window_width = 600

[behavior]
autostart = false
max_results = 8

[[aliases]]
keyword = "ff"
name = "Firefox"
target = "firefox"

[[quicklinks]]
keyword = "gh"
name = "GitHub"
url = "https://github.com"

[[quicklinks]]
keyword = "ghs"
name = "GitHub Search"
url = "https://github.com/search?q={query}"

[scripts]
directory = "~/.config/nova/scripts"
enabled = true
```

## Platform Notes

### Linux
- **X11**: Fully supported with global hotkeys
- **Wayland**: Limited support. Global hotkeys require compositor configuration. Consider configuring your compositor (Sway, Hyprland, etc.) to launch Nova with a keybinding.

### macOS
- Nova runs as a menu bar app (no dock icon)
- First launch requires right-click → Open to bypass Gatekeeper
- Accessibility permissions may be required for global hotkeys

## Tech Stack

- **UI**: [iced](https://iced.rs) - Cross-platform Rust GUI framework
- **Hotkeys**: [global-hotkey](https://crates.io/crates/global-hotkey) - Cross-platform hotkey registration
- **Clipboard**: [arboard](https://crates.io/crates/arboard) - Cross-platform clipboard
- **Config**: TOML
- **IPC**: Local sockets for single-instance

## Contributing

Contributions are welcome! Feel free to open issues or submit PRs.

## License

MIT
