# nvprime - Just commands for building, testing, and system integration

# Default recipe (list available commands)
default:
    @just --list

# Build the project in debug mode
build:
    cargo build

# Build the project in release mode with optimizations
build-release:
    cargo build --release

# Build and run tests
test:
    cargo test

# Build, run tests, and check for warnings
check:
    cargo check --all-targets
    cargo clippy -- -D warnings
    cargo test

# Run the client (requires daemon to be running)
run *ARGS:
    cargo run --bin nvprime -- {{ ARGS }}

# Run the daemon (requires root)
run-daemon:
    @echo "Note: Daemon requires root privileges"
    cargo build --bin nvprime-sys
    sudo ./target/debug/nvprime-sys

# Clean build artifacts
clean:
    cargo clean

# Update dependencies to latest compatible versions
update:
    cargo update

# Format code
fmt:
    cargo fmt

# Install to system (requires root)
install: build-release
    ./system/install.sh install

# Install and enable the daemon service
install-service: build-release
    ./system/install.sh install-service

# Uninstall from system (requires root)
uninstall:
    ./system/install.sh uninstall

# Show installation status
show-installed:
    ./system/install.sh status

# Restart the daemon service
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

# Build and install in one command
all: build-release install

# Development workflow: format, check, test
dev: fmt check

# Full CI workflow: format check, clippy, tests, build
ci:
    cargo fmt -- --check
    cargo clippy -- -D warnings
    cargo test --all-targets
    cargo build --release

# Benchmark build times
bench-build:
    @echo "Benchmarking clean build..."
    cargo clean
    time cargo build --release

# Show dependency tree
deps:
    cargo tree

# Show outdated dependencies
outdated:
    cargo outdated

# Security audit of dependencies
audit:
    cargo audit

# Generate documentation
doc:
    cargo doc --no-deps --open

# Watch and rebuild on changes (requires cargo-watch)
watch:
    cargo watch -x check -x test

# Create a release build and show binary sizes
release-info: build-release
    @echo "Release binaries built:"
    @ls -lh target/release/nvprime target/release/nvprime-sys
    @echo ""
    @echo "Stripped sizes:"
    @du -h target/release/nvprime target/release/nvprime-sys
