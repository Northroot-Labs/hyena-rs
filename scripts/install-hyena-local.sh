#!/usr/bin/env bash
# Install built binary from release/ into bin/ (no GitHub). Run from hyena-rs root.
# Use after build-release.sh so Cursor MCP can use HYENA_BIN=../../bin/hyena.
set -e
REPO_ROOT="${1:-$(cd "$(dirname "$0")/.." && pwd)}"
cd "$REPO_ROOT"
CHECKPOINT_ID=$(grep -E '^CHECKPOINT_ID=' release-pin.txt 2>/dev/null | cut -d= -f2-)
if [[ -z "$CHECKPOINT_ID" ]]; then
  echo "Missing CHECKPOINT_ID in release-pin.txt" >&2
  exit 1
fi
ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARTIFACT="hyena-${CHECKPOINT_ID}-${ARCH}-${OS}"
if [[ ! -f "release/$ARTIFACT" ]]; then
  echo "Run ./scripts/build-release.sh first. Missing release/$ARTIFACT" >&2
  exit 1
fi
mkdir -p bin
cp "release/$ARTIFACT" "bin/$ARTIFACT"
ln -sf "$ARTIFACT" bin/hyena
echo "Installed bin/$ARTIFACT -> bin/hyena"
