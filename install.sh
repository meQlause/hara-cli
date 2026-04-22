#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────────────────
# HARA CLI Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/meQlause/hara-cli/master/install.sh | bash
# ──────────────────────────────────────────────────────────────────────────────
set -euo pipefail

REPO="meQlause/hara-cli"
BIN_NAME="hara"
INSTALL_DIR="$HOME/.local/bin"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  MINGW* | MSYS* | CYGWIN*)
    # Windows Git Bash / MSYS2
    ASSET="hara-windows-x86_64.zip"
    BIN_NAME="hara.exe"
    IS_WINDOWS=true
    ;;
  *)
    echo "Error: HARA CLI is specialized for Windows Git Bash / MSYS2 environments."
    echo "Your OS ($OS) is not supported."
    exit 1
    ;;
esac

echo "Fetching latest HARA release..."
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep '"tag_name"' \
  | head -1 \
  | sed 's/.*"tag_name": *"\(.*\)".*/\1/')

if [[ -z "$LATEST" ]]; then
  echo "Could not determine latest release. Check your internet connection."
  exit 1
fi

echo "   Latest version: $LATEST"

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST}/${ASSET}"
TMP_DIR="$(mktemp -d)"

echo "⬇Downloading ${ASSET}..."
curl -fsSL "$DOWNLOAD_URL" -o "${TMP_DIR}/${ASSET}"

echo "Extracting..."
case "$ASSET" in
  *.tar.gz)
    tar -xzf "${TMP_DIR}/${ASSET}" -C "$TMP_DIR"
    ;;
  *.zip)
    unzip -q "${TMP_DIR}/${ASSET}" -d "$TMP_DIR"
    ;;
esac

# ── Install ───────────────────────────────────────────────────────────────────
mkdir -p "$INSTALL_DIR"

if [[ "${IS_WINDOWS:-false}" == "true" ]]; then
  install -m 755 "${TMP_DIR}/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"
  chmod +x "${INSTALL_DIR}/${BIN_NAME}"
  printf '%s\n' \
    '#!/usr/bin/env bash' \
    '"$(dirname "${BASH_SOURCE[0]}")/hara.exe" "$@"' \
    > "${INSTALL_DIR}/hara"
  chmod +x "${INSTALL_DIR}/hara"
else
  install -m 755 "${TMP_DIR}/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"
  chmod +x "${INSTALL_DIR}/${BIN_NAME}"
fi

rm -rf "$TMP_DIR"

echo "HARA ${LATEST} installed to: ${INSTALL_DIR}/${BIN_NAME}"

if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
  echo ""
  echo "${INSTALL_DIR} is not in your PATH."
  echo "   Add the following line to your ~/.bashrc (or ~/.bash_profile):"
  echo ""
  echo "     export PATH=\"\$HOME/.local/bin:\$PATH\""
  echo ""

  PROFILE_FILE=""
  if [[ -f "$HOME/.bashrc" ]]; then
    PROFILE_FILE="$HOME/.bashrc"
  elif [[ -f "$HOME/.bash_profile" ]]; then
    PROFILE_FILE="$HOME/.bash_profile"
  else
    # Default to .bashrc for Git Bash/MSYS2 environments if none exists
    PROFILE_FILE="$HOME/.bashrc"
    touch "$PROFILE_FILE"
    echo "   Created new profile: ${PROFILE_FILE}"
  fi

  if [[ -n "$PROFILE_FILE" ]]; then
    if ! grep -q 'HOME/.local/bin' "$PROFILE_FILE"; then
      echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$PROFILE_FILE"
      echo "   Auto-added to ${PROFILE_FILE}"
      echo "   Run: source ${PROFILE_FILE}"
    fi
  fi
fi

echo ""
echo "Get started:"
echo "   hara foundry install               — install Foundry toolchain"
echo "   hara foundry init                  — initialise a HARA project"
echo "   hara foundry contract uc <Name>    — scaffold an upgradeable contract"
