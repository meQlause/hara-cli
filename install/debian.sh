#!/usr/bin/env bash
# HARA CLI - Debian/Linux Installer
set -euo pipefail

REPO="meQlause/hara-cli"
BIN_NAME="hara"
INSTALL_DIR="$HOME/.local/bin"

echo -e "\n>>> HARA Installer for Debian/Linux <<<\n"

# 1. Dependency Check
for cmd in curl tar grep sed; do
    if ! command -v "$cmd" &> /dev/null; then
        echo "Error: $cmd is required. Please install it first."
        exit 1
    fi
done

# 2. API Check
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep '"tag_name"' \
  | head -1 \
  | sed 's/.*"tag_name": *"\(.*\)".*/\1/')

if [[ -z "$LATEST" ]]; then
  echo "Error: Could not determine latest version."
  exit 1
fi
echo "[+] Found latest version: $LATEST"

# 3. Download
ASSET="hara-linux-x86_64.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST}/${ASSET}"
TMP_DIR="$(mktemp -d)"

echo "[+] Downloading ${ASSET}..."
curl -fsSL "$DOWNLOAD_URL" -o "${TMP_DIR}/${ASSET}"

# 4. Extract & Install
echo "[+] Extracting..."
tar -xzf "${TMP_DIR}/${ASSET}" -C "$TMP_DIR"
mkdir -p "$INSTALL_DIR"
mv "${TMP_DIR}/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"
chmod +x "${INSTALL_DIR}/${BIN_NAME}"

# 5. Path Configuration
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    SHELL_PROFILE=""
    if [[ -f "$HOME/.bashrc" ]]; then SHELL_PROFILE="$HOME/.bashrc"
    elif [[ -f "$HOME/.zshrc" ]]; then SHELL_PROFILE="$HOME/.zshrc"
    fi

    if [[ -n "$SHELL_PROFILE" ]]; then
        echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> "$SHELL_PROFILE"
        echo "[+] Added to ${SHELL_PROFILE}. Please run 'source ${SHELL_PROFILE}'"
    else
        echo "[!] Warning: Could not find shell profile. Add ${INSTALL_DIR} to your PATH manually."
    fi
fi

rm -rf "$TMP_DIR"
echo -e "\n✨ Success! Run 'hara --version' to verify.\n"
