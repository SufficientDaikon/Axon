#!/usr/bin/env bash
set -euo pipefail

REPO="axon-lang/axon"
BINARY="axonc"
INSTALL_DIR="/usr/local/bin"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[info]${NC} $*"; }
warn()  { echo -e "${YELLOW}[warn]${NC} $*"; }
error() { echo -e "${RED}[error]${NC} $*" >&2; exit 1; }

# Detect OS
detect_os() {
    local uname_s
    uname_s="$(uname -s)"
    case "$uname_s" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
        *)       error "Unsupported OS: $uname_s" ;;
    esac
}

# Detect architecture
detect_arch() {
    local uname_m
    uname_m="$(uname -m)"
    case "$uname_m" in
        x86_64|amd64)  echo "x86_64" ;;
        arm64|aarch64) echo "aarch64" ;;
        *)             error "Unsupported architecture: $uname_m" ;;
    esac
}

# Map OS/arch to Rust target triple
get_target() {
    local os="$1" arch="$2"
    case "${os}-${arch}" in
        linux-x86_64)  echo "x86_64-unknown-linux-gnu" ;;
        macos-x86_64)  echo "x86_64-apple-darwin" ;;
        macos-aarch64) echo "aarch64-apple-darwin" ;;
        *)             error "Unsupported platform: ${os}-${arch}" ;;
    esac
}

# Get latest release tag from GitHub
get_latest_version() {
    if command -v curl &>/dev/null; then
        curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
            | grep '"tag_name"' | head -1 | sed -E 's/.*"([^"]+)".*/\1/'
    elif command -v wget &>/dev/null; then
        wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" \
            | grep '"tag_name"' | head -1 | sed -E 's/.*"([^"]+)".*/\1/'
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
}

# Download binary
download() {
    local url="$1" dest="$2"
    info "Downloading from $url"
    if command -v curl &>/dev/null; then
        curl -fsSL -o "$dest" "$url"
    elif command -v wget &>/dev/null; then
        wget -qO "$dest" "$url"
    fi
}

main() {
    local version="${1:-}"

    info "Axon Compiler Installer"
    echo ""

    local os arch target
    os="$(detect_os)"
    arch="$(detect_arch)"
    target="$(get_target "$os" "$arch")"
    info "Detected platform: ${os}/${arch} (${target})"

    if [ -z "$version" ]; then
        info "Fetching latest release..."
        version="$(get_latest_version)"
        [ -z "$version" ] && error "Could not determine latest version"
    fi
    info "Version: ${version}"

    local artifact_name="${BINARY}-${target}"
    local download_url="https://github.com/${REPO}/releases/download/${version}/${artifact_name}"

    local tmp_dir
    tmp_dir="$(mktemp -d)"
    trap 'rm -rf "$tmp_dir"' EXIT

    download "$download_url" "${tmp_dir}/${BINARY}"
    chmod +x "${tmp_dir}/${BINARY}"

    # Install
    if [ -w "$INSTALL_DIR" ]; then
        mv "${tmp_dir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    else
        info "Installing to ${INSTALL_DIR} (requires sudo)"
        sudo mv "${tmp_dir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    fi

    info "Installed ${BINARY} ${version} to ${INSTALL_DIR}/${BINARY}"
    echo ""
    info "Run 'axonc --help' to get started."
}

main "$@"
