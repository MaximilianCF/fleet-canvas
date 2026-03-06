#!/usr/bin/env bash
# Build an AppImage for RMF Site Editor.
# Prerequisites: cargo, linuxdeploy (or linuxdeploy-x86_64.AppImage in PATH)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
APP_NAME="rmf-site-editor"
APP_DIR="$PROJECT_ROOT/target/appimage/AppDir"

echo "==> Building release binary..."
cargo build --release --bin rmf_site_editor --manifest-path "$PROJECT_ROOT/Cargo.toml"

echo "==> Preparing AppDir..."
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/usr/bin"
mkdir -p "$APP_DIR/usr/share/applications"
mkdir -p "$APP_DIR/usr/share/icons/hicolor/256x256/apps"

cp "$PROJECT_ROOT/target/release/rmf_site_editor" "$APP_DIR/usr/bin/"
cp "$SCRIPT_DIR/rmf-site-editor.desktop" "$APP_DIR/usr/share/applications/"

# Use icon if available, otherwise generate a placeholder
if [ -f "$SCRIPT_DIR/rmf-site-editor.png" ]; then
    cp "$SCRIPT_DIR/rmf-site-editor.png" "$APP_DIR/usr/share/icons/hicolor/256x256/apps/"
else
    echo "Warning: No icon found at $SCRIPT_DIR/rmf-site-editor.png, AppImage will have no icon"
fi

# Symlinks required by AppImage spec
ln -sf usr/share/applications/rmf-site-editor.desktop "$APP_DIR/$APP_NAME.desktop"
if [ -f "$APP_DIR/usr/share/icons/hicolor/256x256/apps/rmf-site-editor.png" ]; then
    ln -sf usr/share/icons/hicolor/256x256/apps/rmf-site-editor.png "$APP_DIR/rmf-site-editor.png"
fi

echo "==> Building AppImage..."
# Download linuxdeploy if not present
LINUXDEPLOY="linuxdeploy-x86_64.AppImage"
if ! command -v linuxdeploy &>/dev/null && [ ! -f "$LINUXDEPLOY" ]; then
    echo "Downloading linuxdeploy..."
    curl -fsSL -o "$LINUXDEPLOY" "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage"
    chmod +x "$LINUXDEPLOY"
fi

DEPLOY_CMD="${LINUXDEPLOY_PATH:-linuxdeploy}"
if [ -f "./$LINUXDEPLOY" ]; then
    DEPLOY_CMD="./$LINUXDEPLOY"
fi

OUTPUT="${APP_NAME}-$(uname -m).AppImage"
export OUTPUT
LDAI_OUTPUT="$OUTPUT"
export LDAI_OUTPUT

"$DEPLOY_CMD" --appdir "$APP_DIR" --output appimage

echo "==> Done: $OUTPUT"
