#!/usr/bin/env bash
set -euo pipefail

echo "Building cyberplug (release)..."
cargo build --release

if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
    BIN_SRC="${CARGO_TARGET_DIR}/release/cyberplug"
elif [[ -f "$HOME/.cargo/config.toml" ]] && grep -q "target-dir" "$HOME/.cargo/config.toml"; then
    TARGET_DIR=$(grep "target-dir" "$HOME/.cargo/config.toml" | sed -E 's/.*=\s*"(.*)"/\1/')
    BIN_SRC="${TARGET_DIR}/release/cyberplug"
else
    BIN_SRC="target/release/cyberplug"
fi

if [[ ! -f "$BIN_SRC" ]]; then
    echo "Could not find built binary at: $BIN_SRC"
    echo "Check your cargo target-dir configuration."
    exit 1
fi

BIN_DST="$HOME/.local/bin/cyberplug"
mkdir -p "$HOME/.local/bin"
cp "$BIN_SRC" "$BIN_DST"
chmod +x "$BIN_DST"
echo "Installed to $BIN_DST"

if command -v cyberplug >/dev/null 2>&1; then
    echo "Done — run 'cyberplug' to start."
else
    echo "Installed, but ~/.local/bin isn't on your \$PATH yet."
    echo "Add this to your shell config, then restart your shell:"
    echo ""
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

echo ""
if [[ -d "$HOME/.config/omarchy/plugins" ]]; then
    read -r -p "Install the Omarchy bar icon too? [y/N] " reply
    if [[ "$reply" =~ ^[Yy]$ ]]; then
        PLUGIN_DST="$HOME/.config/omarchy/plugins/custom.cyberplug"
        mkdir -p "$PLUGIN_DST"
        cp omarchy-plugin/custom.cyberplug/manifest.json "$PLUGIN_DST/"
        cp omarchy-plugin/custom.cyberplug/BarWidget.qml "$PLUGIN_DST/"
        cp omarchy-plugin/cyberplug-toggle "$HOME/.local/bin/cyberplug-toggle"
        chmod +x "$HOME/.local/bin/cyberplug-toggle"

        omarchy plugin enable custom.cyberplug right 2>/dev/null || true

        echo "Bar icon installed."
        echo ""
        echo "To make it open as a floating window, add this line to"
        echo "~/.config/hypr/looknfeel.lua:"
        echo ""
        cat omarchy-plugin/looknfeel-snippet.lua | tail -1
        echo ""
        echo "Then run: hyprctl reload"
    fi
fi
