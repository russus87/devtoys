#!/usr/bin/env bash
# Downloads the native Pdfium library (bblanchon/pdfium-binaries build) into the
# destination folder. Used both in development and in CI. Required at build time
# so the binary can bundle it as a Tauri resource (loaded at runtime).
#
# Usage:  scripts/fetch-pdfium.sh [dest_folder]
#   default dest: src-tauri/pdfium
#
# OS/arch are auto-detected but can be forced:
#   PDFIUM_OS   = linux | mac | win
#   PDFIUM_ARCH = x64 | arm64
set -euo pipefail

DEST="${1:-src-tauri/pdfium}"
REPO="bblanchon/pdfium-binaries"

os="${PDFIUM_OS:-}"
if [ -z "$os" ]; then
  case "$(uname -s)" in
    Linux*)  os="linux" ;;
    Darwin*) os="mac" ;;
    MINGW*|MSYS*|CYGWIN*) os="win" ;;
    *) echo "Unrecognized OS: $(uname -s)" >&2; exit 1 ;;
  esac
fi

arch="${PDFIUM_ARCH:-}"
if [ -z "$arch" ]; then
  case "$(uname -m)" in
    x86_64|amd64) arch="x64" ;;
    arm64|aarch64) arch="arm64" ;;
    *) echo "Unrecognized arch: $(uname -m)" >&2; exit 1 ;;
  esac
fi

asset="pdfium-${os}-${arch}.tgz"
url="https://github.com/${REPO}/releases/latest/download/${asset}"

echo "Downloading ${asset} -> ${DEST}"
mkdir -p "$DEST"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

curl -fL --retry 3 -o "$tmp/pdfium.tgz" "$url"
tar -xzf "$tmp/pdfium.tgz" -C "$tmp"

lib="$(find "$tmp" -maxdepth 2 -type f \
  \( -name 'pdfium.dll' -o -name 'libpdfium.so' -o -name 'libpdfium.dylib' \) | head -1)"

if [ -z "$lib" ]; then
  echo "Library not found in the archive" >&2
  find "$tmp" -type f >&2
  exit 1
fi

cp "$lib" "$DEST/"
echo "OK: $(basename "$lib") in $DEST/"
