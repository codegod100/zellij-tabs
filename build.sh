#!/usr/bin/env bash
# Build the tabs plugin for Zellij and install it.
set -euo pipefail

cd "$(dirname "$0")"
nix-shell -p gcc pkg-config openssl.dev --command \
  "CC=gcc cargo build --release --target wasm32-wasip1"

cp target/wasm32-wasip1/release/tabs.wasm ~/.config/zellij/tabs.wasm
echo "→ Installed tabs.wasm to ~/.config/zellij/tabs.wasm"
