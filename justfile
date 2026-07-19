# nvprime — baseline multi-bin (client + daemon); no install.sh (option A)

bins    := "nvprime nvprime-sys"
bin_dir := env_var("HOME") / ".local/bin"
sys_dir := "/usr/local/bin"

# List available recipes
default:
    @just --list

# Build release binaries
build:
    cargo build --release

# Build in debug mode
build-debug:
    cargo build

# Run unit/integration tests that do not need live external services
test:
    cargo test

# Auto-format the tree
fmt:
    cargo fmt --all

# Check formatting (CI gate)
fmt-check:
    cargo fmt --all -- --check

# Lint — warnings denied (CI gate)
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Full local gate, mirrors CI (fmt + clippy + tests)
check: fmt-check lint test

# Compress every release binary with upx (skips a binary if already packed)
compress: build
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v upx >/dev/null 2>&1; then
        echo "compress: upx not found in PATH" >&2
        exit 1
    fi
    for b in {{bins}}; do
        path="target/release/$b"
        if [ ! -f "$path" ]; then
            echo "compress: missing $path (is bins= correct?)" >&2
            exit 1
        fi
        upx -t "$path" >/dev/null 2>&1 || upx --best --lzma "$path"
        echo "compressed $path"
    done

# Install both binaries into ~/.local/bin (default) or /usr/local/bin (--system)
install *flags: compress
    #!/usr/bin/env bash
    set -euo pipefail
    dir="{{bin_dir}}"
    sudo=""
    for f in {{flags}}; do
        case "$f" in
            --system) dir="{{sys_dir}}"; sudo="sudo" ;;
            *) echo "install: unknown flag '$f' (only --system is supported)" >&2; exit 1 ;;
        esac
    done
    for b in {{bins}}; do
        $sudo install -Dm755 "target/release/$b" "$dir/$b"
        echo "installed $dir/$b"
    done

# Remove both installed binaries (pass --system for /usr/local/bin via sudo)
uninstall *flags:
    #!/usr/bin/env bash
    set -euo pipefail
    dir="{{bin_dir}}"
    sudo=""
    for f in {{flags}}; do
        case "$f" in
            --system) dir="{{sys_dir}}"; sudo="sudo" ;;
            *) echo "uninstall: unknown flag '$f' (only --system is supported)" >&2; exit 1 ;;
        esac
    done
    for b in {{bins}}; do
        $sudo rm -f "$dir/$b"
        echo "removed $dir/$b"
    done

# Remove build artifacts
clean:
    cargo clean

# ---------------------------------------------------------------------------
# Specials — local dev helpers (unit/dbus stay in system/ for manual install)
# ---------------------------------------------------------------------------

# Run the client (debug)
run *args:
    cargo run --bin nvprime -- {{args}}

# Run the daemon in debug (requires root)
run-daemon:
    @echo "Note: daemon requires root privileges"
    cargo build --bin nvprime-sys
    sudo ./target/debug/nvprime-sys

# Restart the daemon service (if installed manually)
restart-daemon:
    sudo systemctl restart nvprime.service
    systemctl status nvprime.service --no-pager

# View daemon logs
logs:
    journalctl -u nvprime.service -f

# View recent daemon logs
logs-recent:
    journalctl -u nvprime.service -n 50 --no-pager

# Check daemon status
status:
    systemctl status nvprime.service --no-pager

# Test D-Bus connection
test-dbus:
    @echo "Testing D-Bus connection to nvprime daemon..."
    busctl call com.github.nvprime /com/github/nvprime com.github.nvprime.Service ping
