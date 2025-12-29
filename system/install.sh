#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Installation paths
BIN_DIR="/usr/local/bin"
DBUS_DIR="/usr/share/dbus-1/system.d"
SYSTEMD_DIR="/usr/lib/systemd/system"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run as root"
        log_info "Try: sudo $0 $*"
        exit 1
    fi
}

# Check if binaries exist
check_binaries() {
    local release_dir="$PROJECT_ROOT/target/release"

    if [[ ! -f "$release_dir/nvprime" ]]; then
        log_error "Binary not found: $release_dir/nvprime"
        log_info "Run 'cargo build --release' first"
        exit 1
    fi

    if [[ ! -f "$release_dir/nvprime-sys" ]]; then
        log_error "Binary not found: $release_dir/nvprime-sys"
        log_info "Run 'cargo build --release' first"
        exit 1
    fi
}

# Install function
install() {
    check_root "$@"
    check_binaries

    log_info "Installing nvprime to system..."

    # Install binaries
    log_info "Installing binaries to $BIN_DIR..."
    install -Dm755 "$PROJECT_ROOT/target/release/nvprime" "$BIN_DIR/nvprime"
    install -Dm755 "$PROJECT_ROOT/target/release/nvprime-sys" "$BIN_DIR/nvprime-sys"
    log_info "  ✓ nvprime → $BIN_DIR/nvprime"
    log_info "  ✓ nvprime-sys → $BIN_DIR/nvprime-sys"

    # Install D-Bus configuration
    log_info "Installing D-Bus configuration to $DBUS_DIR..."
    install -Dm644 "$SCRIPT_DIR/com.github.nvprime.conf" "$DBUS_DIR/com.github.nvprime.conf"
    log_info "  ✓ D-Bus policy → $DBUS_DIR/com.github.nvprime.conf"

    # Install systemd service
    log_info "Installing systemd service to $SYSTEMD_DIR..."
    install -Dm644 "$SCRIPT_DIR/nvprime.service" "$SYSTEMD_DIR/nvprime.service"
    log_info "  ✓ Systemd service → $SYSTEMD_DIR/nvprime.service"

    # Reload systemd
    log_info "Reloading systemd daemon..."
    systemctl daemon-reload
    log_info "  ✓ Systemd daemon reloaded"

    echo ""
    log_info "Installation complete!"
    echo ""
    echo "Next steps:"
    echo "  Enable and start the daemon:"
    echo "    sudo systemctl enable --now nvprime.service"
    echo ""
    echo "  Check daemon status:"
    echo "    systemctl status nvprime.service"
    echo ""
    echo "  View logs:"
    echo "    journalctl -u nvprime.service -f"
}

# Install and enable service
install_service() {
    install "$@"

    echo ""
    log_info "Enabling and starting nvprime daemon..."
    systemctl enable nvprime.service
    systemctl start nvprime.service
    log_info "  ✓ Service enabled and started"

    echo ""
    systemctl status nvprime.service --no-pager || true
}

# Uninstall function
uninstall() {
    check_root "$@"

    log_info "Uninstalling nvprime from system..."

    # Stop and disable service
    log_info "Stopping and disabling nvprime daemon..."
    if systemctl is-active nvprime.service &>/dev/null; then
        systemctl stop nvprime.service
        log_info "  ✓ Service stopped"
    else
        log_warn "  Service not running"
    fi

    if systemctl is-enabled nvprime.service &>/dev/null; then
        systemctl disable nvprime.service
        log_info "  ✓ Service disabled"
    else
        log_warn "  Service not enabled"
    fi

    # Remove binaries
    log_info "Removing binaries from $BIN_DIR..."
    if [[ -f "$BIN_DIR/nvprime" ]]; then
        rm -f "$BIN_DIR/nvprime"
        log_info "  ✓ Removed $BIN_DIR/nvprime"
    else
        log_warn "  $BIN_DIR/nvprime not found"
    fi

    if [[ -f "$BIN_DIR/nvprime-sys" ]]; then
        rm -f "$BIN_DIR/nvprime-sys"
        log_info "  ✓ Removed $BIN_DIR/nvprime-sys"
    else
        log_warn "  $BIN_DIR/nvprime-sys not found"
    fi

    # Remove D-Bus configuration
    log_info "Removing D-Bus configuration from $DBUS_DIR..."
    if [[ -f "$DBUS_DIR/com.github.nvprime.conf" ]]; then
        rm -f "$DBUS_DIR/com.github.nvprime.conf"
        log_info "  ✓ Removed $DBUS_DIR/com.github.nvprime.conf"
    else
        log_warn "  $DBUS_DIR/com.github.nvprime.conf not found"
    fi

    # Remove systemd service
    log_info "Removing systemd service from $SYSTEMD_DIR..."
    if [[ -f "$SYSTEMD_DIR/nvprime.service" ]]; then
        rm -f "$SYSTEMD_DIR/nvprime.service"
        log_info "  ✓ Removed $SYSTEMD_DIR/nvprime.service"
    else
        log_warn "  $SYSTEMD_DIR/nvprime.service not found"
    fi

    # Reload systemd
    log_info "Reloading systemd daemon..."
    systemctl daemon-reload
    log_info "  ✓ Systemd daemon reloaded"

    echo ""
    log_info "Uninstallation complete!"
}

# Show installation status
status() {
    echo "Installation Status:"
    echo ""

    echo "Binaries:"
    if [[ -f "$BIN_DIR/nvprime" ]]; then
        echo "  ✓ $BIN_DIR/nvprime ($(stat -c%s "$BIN_DIR/nvprime" | numfmt --to=iec))"
    else
        echo "  ✗ $BIN_DIR/nvprime (not installed)"
    fi

    if [[ -f "$BIN_DIR/nvprime-sys" ]]; then
        echo "  ✓ $BIN_DIR/nvprime-sys ($(stat -c%s "$BIN_DIR/nvprime-sys" | numfmt --to=iec))"
    else
        echo "  ✗ $BIN_DIR/nvprime-sys (not installed)"
    fi

    echo ""
    echo "Configuration:"
    if [[ -f "$DBUS_DIR/com.github.nvprime.conf" ]]; then
        echo "  ✓ $DBUS_DIR/com.github.nvprime.conf"
    else
        echo "  ✗ $DBUS_DIR/com.github.nvprime.conf (not installed)"
    fi

    if [[ -f "$SYSTEMD_DIR/nvprime.service" ]]; then
        echo "  ✓ $SYSTEMD_DIR/nvprime.service"
    else
        echo "  ✗ $SYSTEMD_DIR/nvprime.service (not installed)"
    fi

    echo ""
    echo "Service Status:"
    if systemctl is-enabled nvprime.service &>/dev/null; then
        echo "  ✓ Enabled"
    else
        echo "  ✗ Not enabled"
    fi

    if systemctl is-active nvprime.service &>/dev/null; then
        echo "  ✓ Running"
    else
        echo "  ✗ Not running"
    fi
}

# Show usage
usage() {
    cat << EOF
Usage: $0 <command>

Commands:
    install         Install nvprime to system (requires root)
    install-service Install and enable the daemon service (requires root)
    uninstall       Remove nvprime from system (requires root)
    status          Show installation status

Examples:
    sudo $0 install
    sudo $0 install-service
    sudo $0 uninstall
    $0 status
EOF
}

# Main command handler
main() {
    case "${1:-}" in
        install)
            install "${@:2}"
            ;;
        install-service)
            install_service "${@:2}"
            ;;
        uninstall)
            uninstall "${@:2}"
            ;;
        status)
            status
            ;;
        -h|--help|help)
            usage
            ;;
        *)
            log_error "Unknown command: ${1:-}"
            echo ""
            usage
            exit 1
            ;;
    esac
}

main "$@"
