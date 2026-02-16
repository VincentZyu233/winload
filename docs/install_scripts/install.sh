#!/bin/bash
# winload installer — supports apt (deb) and dnf (rpm) on x86_64 / aarch64
# Usage: curl -fsSL https://raw.githubusercontent.com/VincentZyuApps/winload/main/docs/install_scripts/install.sh | bash
set -e

REPO="VincentZyuApps/winload"
API_URL="https://api.github.com/repos/${REPO}/releases/latest"

# ── Detect architecture ──────────────────────────────────
ARCH=$(uname -m)
case "$ARCH" in
  x86_64|amd64)  ARCH_NAME="x86_64" ;;
  aarch64|arm64)  ARCH_NAME="aarch64" ;;
  *)
    echo "❌ Unsupported architecture: $ARCH"
    echo "   Only x86_64 and aarch64 are supported."
    echo "   Please visit https://github.com/${REPO}/releases to download manually,"
    echo "   or build from source."
    exit 1
    ;;
esac

# ── Detect package manager ───────────────────────────────
if command -v apt-get >/dev/null 2>&1; then
  PKG_MGR="apt"
elif command -v dnf >/dev/null 2>&1; then
  PKG_MGR="dnf"
else
  echo "❌ Unsupported package manager."
  echo "   This installer only supports apt (Debian/Ubuntu) and dnf (Fedora/RHEL)."
  echo "   Please visit https://github.com/${REPO}/releases to download manually,"
  echo "   or build from source."
  exit 1
fi

echo "🔍 Detected: arch=$ARCH pkg_mgr=$PKG_MGR"

# ── Fetch latest release version ─────────────────────────
echo "📡 Fetching latest version..."
VERSION=$(curl -fsSL "$API_URL" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
if [ -z "$VERSION" ]; then
  echo "❌ Failed to fetch latest version from GitHub API."
  exit 1
fi
echo "📦 Latest version: $VERSION"

# ── Download & Install ───────────────────────────────────
BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
PLATFORM="linux-${ARCH_NAME}"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

if [ "$PKG_MGR" = "apt" ]; then
  PKG_FILE="winload-${PLATFORM}-${VERSION}.deb"
  echo "📥 Downloading ${PKG_FILE}..."
  curl -fSL -o "${TMP_DIR}/${PKG_FILE}" "${BASE_URL}/${PKG_FILE}"
  echo "📦 Installing via apt..."
  sudo dpkg -i "${TMP_DIR}/${PKG_FILE}" || sudo apt-get install -f -y
elif [ "$PKG_MGR" = "dnf" ]; then
  PKG_FILE="winload-${PLATFORM}-${VERSION}.rpm"
  echo "📥 Downloading ${PKG_FILE}..."
  curl -fSL -o "${TMP_DIR}/${PKG_FILE}" "${BASE_URL}/${PKG_FILE}"
  echo "📦 Installing via dnf..."
  sudo dnf install -y "${TMP_DIR}/${PKG_FILE}"
fi

echo ""
echo "✅ winload installed successfully!"
echo "   Run 'winload' to start monitoring."
echo ""
echo "   To uninstall:"
if [ "$PKG_MGR" = "apt" ]; then
  echo "   sudo apt remove winload"
elif [ "$PKG_MGR" = "dnf" ]; then
  echo "   sudo dnf remove winload"
fi
