# Nova

**A keyboard-driven productivity launcher for Linux.** Think [Raycast](https://raycast.com), but open source and built for Linux.

Nova helps you launch apps, run custom commands, and access your favorite websites — all without touching the mouse.

## Features

- **App Launcher** — Instantly search and launch any application
- **Quicklinks** — Open URLs with optional search queries (e.g., `yt cats` → YouTube search for "cats")
- **Aliases** — Create shortcuts to launch apps with custom keywords
- **Scripts** — Run custom shell scripts with arguments and capture output
- **Command Mode** — Type a keyword + space to enter focused search mode with visual feedback
- **Themes** — Multiple built-in themes (Catppuccin, Nord, Dracula, Gruvbox, Tokyo Night, One Dark)
- **Customizable** — Configure accent colors, opacity, hotkeys, and more

## Installation

### From Source

Prerequisites:
- Rust (latest stable)
- GTK 3 development libraries

```bash
# Ubuntu/Debian
sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev

# Fedora
sudo dnf install gtk3-devel webkit2gtk4.1-devel

# Arch
sudo pacman -S gtk3 webkit2gtk-4.1
```

Build and install:

```bash
git clone https://github.com/rusenbb/nova.git
cd nova/src-tauri
cargo build --release
sudo cp target/release/nova /usr/local/bin/
```

### Set Up Hotkey

Nova registers `Alt+Space` by default. You can change this in Settings or manually:

```bash
# GNOME
gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings "['/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/nova/']"
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/nova/ name 'Nova'
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/nova/ command 'nova'
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/nova/ binding '<Alt>space'
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

Config file: `~/.config/nova/config.toml`

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

## Roadmap

- [ ] Plugin system
- [ ] Clipboard history
- [ ] Calculator
- [ ] File search
- [ ] Window management
- [ ] Snippets

## Tech Stack

- **Backend**: Rust + GTK3 (pure Rust, no Electron)
- **Config**: TOML
- **IPC**: Unix sockets for single-instance

## Contributing

Contributions are welcome! Feel free to open issues or submit PRs.

## License

MIT
