#!/usr/bin/env bash
set -e

REPO="akirco/sr"
VERSION="v0.1.1"
FILENAME="sr-portable-linux-x86_64.tar.gz"
URL="https://github.com/$REPO/releases/download/$VERSION/$FILENAME"
INSTALL_DIR="$HOME/.sr"
BIN_DIR="$HOME/.local/bin"
LINK_TARGET="$BIN_DIR/sr"

mkdir -p "$INSTALL_DIR" "$BIN_DIR"

echo "Downloading $FILENAME..."
curl -fsSL "$URL" -o /tmp/sr-portable.tar.gz

echo "Extracting to $INSTALL_DIR..."
tar xzf /tmp/sr-portable.tar.gz -C "$INSTALL_DIR"
rm /tmp/sr-portable.tar.gz

ln -sf "$INSTALL_DIR/run.sh" "$LINK_TARGET"
chmod +x "$INSTALL_DIR/run.sh"

echo "Installed. Run with: sr -i input.jpg -o output.png"
