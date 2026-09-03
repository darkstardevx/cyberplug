#!/usr/bin/env bash
set -euo pipefail

echo "Building cyberplug (release)..."
cargo build --release

BIN_SRC="target/release/cyberplug"
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
