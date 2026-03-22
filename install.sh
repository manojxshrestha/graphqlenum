#!/bin/bash
#===============================================================================
# graphqlenum Installer
#===============================================================================

set -e

VERSION="2.0.0"
TOOL_NAME="graphqlenum"
INSTALL_DIR="${HOME}/.local/bin"
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[*]${NC} $1"; }
log_success() { echo -e "${GREEN}[+]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[!]${NC} $1"; }
log_error() { echo -e "${RED}[-]${NC} $1"; }

banner() {
    echo -e "${GREEN}"
    echo "╔═══════════════════════════════════════════════════════════════╗"
    echo "║                                                               ║"
    echo "║                  graphqlenum Installer                        ║"
    echo "║                                                               ║"
    echo "╚═══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

check_deps() {
    log_info "Checking dependencies..."
    command -v curl &>/dev/null || { log_error "curl not found"; exit 1; }
    command -v jq &>/dev/null || { log_error "jq not found"; exit 1; }
    log_success "Dependencies OK"
}

install_rust() {
    if [ ! -f "$REPO_DIR/target/release/graphqlenum" ]; then
        log_info "Building Rust tool..."
        if ! command -v cargo &>/dev/null; then
            log_info "Installing Rust..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source "${HOME}/.cargo/env"
        fi
        cd "$REPO_DIR"
        cargo build --release
        mv target/release/graphql-path-enum target/release/graphqlenum 2>/dev/null || true
        log_success "Rust tool built"
    else
        log_info "Rust binary already exists"
    fi
}

install_files() {
    log_info "Installing files..."
    mkdir -p "$INSTALL_DIR"
    mkdir -p "$INSTALL_DIR/target/release"
    
    cp "$REPO_DIR/graphqlenum" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/graphqlenum"
    
    if [ -f "$REPO_DIR/target/release/graphqlenum" ]; then
        cp "$REPO_DIR/target/release/graphqlenum" "$INSTALL_DIR/target/release/"
        chmod +x "$INSTALL_DIR/target/release/graphqlenum"
    fi
    
    if [ ! -f "$HOME/.bashrc" ]; then touch "$HOME/.bashrc"; fi
    
    if ! grep -q "$INSTALL_DIR" "$HOME/.bashrc" 2>/dev/null; then
        echo "" >> "$HOME/.bashrc"
        echo "# graphqlenum" >> "$HOME/.bashrc"
        echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> "$HOME/.bashrc"
        log_info "Added $INSTALL_DIR to PATH in ~/.bashrc"
    fi
    
    log_success "Installed to $INSTALL_DIR"
}

verify() {
    log_info "Verifying installation..."
    if command -v "$TOOL_NAME" &>/dev/null; then
        log_success "Installed successfully!"
        echo ""
        "$TOOL_NAME" -h
    else
        log_warn "Run: export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
        log_info "Or restart your terminal"
    fi
}

uninstall() {
    log_info "Uninstalling..."
    rm -f "$INSTALL_DIR/graphqlenum"
    rm -f "$INSTALL_DIR/target/release/graphqlenum"
    log_success "Uninstalled"
}

usage() {
    echo "Usage: $0 [install|uninstall|help]"
    echo ""
    echo "  install    - Install graphqlenum (default)"
    echo "  uninstall  - Remove from system"
    echo "  help       - Show this help"
}

main() {
    banner
    
    case "${1:-install}" in
        install)
            check_deps
            install_rust
            install_files
            verify
            ;;
        uninstall)
            uninstall
            ;;
        help|--help|-h)
            usage
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
}

main "$@"
