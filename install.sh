#!/usr/bin/env bash
# Install workflowy-cli from GitHub releases
# Usage: curl -fsSL https://raw.githubusercontent.com/dhanesh/workflowy_cli/main/install.sh | bash

set -euo pipefail

REPO="dhanesh/workflowy_cli"
BINARY="workflowy-cli"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)  OS_TARGET="unknown-linux-gnu" ;;
    Darwin) OS_TARGET="apple-darwin" ;;
    *)      echo "Error: Unsupported OS: $OS" >&2; exit 1 ;;
esac

case "$ARCH" in
    x86_64|amd64)  ARCH_TARGET="x86_64" ;;
    arm64|aarch64) ARCH_TARGET="aarch64" ;;
    *)             echo "Error: Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

TARGET="${ARCH_TARGET}-${OS_TARGET}"

# Get latest release tag
echo "Fetching latest release..."
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')

if [ -z "$LATEST" ]; then
    echo "Error: Could not determine latest release. Check https://github.com/${REPO}/releases" >&2
    exit 1
fi

echo "Installing ${BINARY} ${LATEST} for ${TARGET}..."

# Download and extract
ARCHIVE_URL="https://github.com/${REPO}/releases/download/${LATEST}/${BINARY}-${TARGET}.tar.gz"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

curl -fsSL "$ARCHIVE_URL" -o "${TMP_DIR}/archive.tar.gz"
tar xzf "${TMP_DIR}/archive.tar.gz" -C "$TMP_DIR"

# Install
if [ -w "$INSTALL_DIR" ]; then
    mv "${TMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
else
    echo "Installing to ${INSTALL_DIR} (requires sudo)..."
    sudo mv "${TMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
fi

chmod +x "${INSTALL_DIR}/${BINARY}"

echo "Installed ${BINARY} ${LATEST} to ${INSTALL_DIR}/${BINARY}"
echo ""
echo "Get started:"
echo "  export WORKFLOWY_API_KEY=\"your-key\"  # from https://beta.workflowy.com/api-key"
echo "  ${BINARY} prime                       # see available commands"
