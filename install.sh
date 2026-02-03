#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="${HOME}/.local/bin"

mkdir -p "$BIN_DIR"
ln -sf "$SCRIPT_DIR/claude-sandbox" "$BIN_DIR/claude-sandbox"

echo "Installed: ${BIN_DIR}/claude-sandbox"
