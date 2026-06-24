#!/usr/bin/env bash
# Build the tabs plugin for Zellij and install it.
set -euo pipefail
cd "$(dirname "$0")"

# Use nix-shell for a consistent build environment (much faster cached)
if command -v nix-shell &>/dev/null; then
  nix-shell --run "cargo build --release --target wasm32-wasip1"
else
  cargo build --release --target wasm32-wasip1
fi

cp target/wasm32-wasip1/release/tabs.wasm ~/.config/zellij/tabs.wasm
echo "→ Installed tabs.wasm to ~/.config/zellij/tabs.wasm"
