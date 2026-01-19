#!/bin/bash
# Setup Super+Space hotkey for Nova via GNOME settings-daemon
# This is the reliable way to register global hotkeys on GNOME

set -e

NOVA_PATH="${1:-$(which nova 2>/dev/null || echo "$HOME/.cargo/bin/nova")}"

# If nova binary doesn't exist yet, use the dev build path
if [ ! -f "$NOVA_PATH" ]; then
    NOVA_PATH="$(dirname "$(realpath "$0")")/../src-tauri/target/debug/nova"
fi

if [ ! -f "$NOVA_PATH" ]; then
    echo "Error: Nova binary not found. Build it first with 'bun run tauri build' or 'cargo build'"
    exit 1
fi

echo "Using Nova at: $NOVA_PATH"

# First, unbind Super+Space from any existing GNOME/IBus bindings
echo "Unbinding Super+Space from GNOME input source switcher..."
gsettings set org.gnome.desktop.wm.keybindings switch-input-source "['']" 2>/dev/null || true
gsettings set org.gnome.desktop.wm.keybindings switch-input-source-backward "['']" 2>/dev/null || true

echo "Unbinding Super+Space from IBus..."
gsettings set org.freedesktop.ibus.general.hotkey triggers "['<Control>space']" 2>/dev/null || true

# Get existing custom keybindings
EXISTING=$(gsettings get org.gnome.settings-daemon.plugins.media-keys custom-keybindings)

# Check if Nova binding already exists
if echo "$EXISTING" | grep -q "nova"; then
    echo "Nova hotkey already registered, updating..."
else
    echo "Adding Nova to custom keybindings..."
    # Add nova to the list
    if [ "$EXISTING" = "@as []" ]; then
        NEW_LIST="['/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/nova/']"
    else
        # Remove trailing ] and add nova
        NEW_LIST=$(echo "$EXISTING" | sed "s/]$/, '\/org\/gnome\/settings-daemon\/plugins\/media-keys\/custom-keybindings\/nova\/']/" | sed "s/\[, /[/")
    fi
    gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings "$NEW_LIST"
fi

# Set the actual keybinding properties
KEYBINDING_PATH="org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/nova/"

gsettings set $KEYBINDING_PATH name "Nova Launcher"
gsettings set $KEYBINDING_PATH command "$NOVA_PATH"
gsettings set $KEYBINDING_PATH binding "<Super>space"

echo ""
echo "Done! Super+Space is now bound to Nova."
echo ""
echo "To test:"
echo "  1. Start Nova daemon: $NOVA_PATH"
echo "  2. Press Super+Space to toggle"
echo ""
echo "To remove this hotkey later:"
echo "  gsettings reset org.gnome.settings-daemon.plugins.media-keys custom-keybindings"
