[← Back to README](../README.md)

# Justfile Usage Guide

This project uses [just](https://github.com/casey/just) as a command runner for common development and deployment tasks.

## Installation

If you don't have `just` installed:

```bash
# Arch Linux
sudo pacman -S just

# Cargo
cargo install just

# Other systems: https://github.com/casey/just#installation
```

## Quick Start

List all available commands:

```bash
just
```

## Development Commands

### Building

```bash
# Build in debug mode
just build

# Build optimized release
just build-release

# Show release binary info
just release-info
```

### Testing

```bash
# Run tests
just test

# Run checks (clippy, tests)
just check

# Development workflow (format + check)
just dev

# Full CI workflow
just ci
```

### Code Quality

```bash
# Format code
just fmt

# Show dependency tree
just deps

# Check for outdated dependencies
just outdated

# Security audit
just audit
```

### Running Locally

```bash
# Build both binaries
just build --debug

# Run the daemon (requires root and the installed D-Bus policy)
sudo env NVPRIME_CONFIG="$PWD/system/nvprime.toml" ./target/debug/nvprimed

# Run the client (in another terminal)
NVPRIME_CONFIG="$PWD/system/nvprime.toml" just run <game-command>

# Example
NVPRIME_CONFIG="$PWD/system/nvprime.toml" just run steam
```

## System Installation

### Install to System

```bash
# Install binaries, configuration, and enable the daemon service
just setup
```

**Installation locations:**

- Binaries are `/usr/local/bin/nvprime` and `/usr/local/bin/nvprimed`.
- D-Bus policy is `/etc/dbus-1/system.d/com.github.nvprime.conf`.
- Systemd unit is `/etc/systemd/system/nvprime.service`.
- System-wide configuration is `/etc/nvprime.toml`.

### System Management

```bash
# Check daemon status
just status

# Restart daemon
just restart

# View live logs
just logs

# View recent logs
just logs-recent

# Test D-Bus connection
just test-dbus

# Show what's installed
just show-installed
```

### Uninstall

```bash
# Remove all installed files and stop service
just teardown
```

## Advanced Commands

### Maintenance

```bash
# Clean build artifacts
just clean

# Update dependencies
just update
```

### Documentation

```bash
# Generate and open docs
just doc
```

### Development Tools

```bash
# Watch for changes and rebuild (requires cargo-watch)
just watch

# Benchmark build times
just bench-build
```

## Workflow Examples

### Local Development

```bash
# Make code changes
# Format and check
just dev

# Run tests
just test

# Test locally (terminal 1)
sudo env NVPRIME_CONFIG="$PWD/system/nvprime.toml" ./target/debug/nvprimed

# Test client (terminal 2)
NVPRIME_CONFIG="$PWD/system/nvprime.toml" just run game.exe --arg1 --arg2
```

### Preparing a Release

```bash
# Run full CI checks
just ci

# Build optimized release
just build-release

# Check binary sizes
just release-info
```

### Installing for Production

```bash
# Build and install
just setup

# Verify installation
just status
just show-installed

# Test D-Bus connection
just test-dbus

# Monitor logs
just logs
```

### Troubleshooting

```bash
# Check if daemon is running
just status

# View recent errors
just logs-recent

# Restart daemon
just restart

# Test D-Bus connection
just test-dbus

# Reinstall if needed
just teardown
just setup
```

## Common Issues

### "just: command not found"

Install just using one of the methods listed in the Installation section above.

### "Permission denied" during installation

The install/uninstall commands require root privileges and use `sudo` internally.

### Daemon won't start

Check logs with `just logs-recent` to see error messages.

### D-Bus connection fails

Ensure:

- Daemon is running: `just status`
- D-Bus config is installed: `just show-installed`
- You have necessary permissions

## Tips

- Use `just` (no arguments) to see all available commands
- All commands are defined in `justfile` at the project root
- Commands can be combined: `just fmt check test`
- Use tab completion if your shell supports it
