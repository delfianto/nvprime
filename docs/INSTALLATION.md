[← Back to README](../README.md)

# System Integration Files

This directory contains all system-level configuration files and installation scripts for nvprime.

## Directory Structure

```
system/
├── install.sh                    # Installation/uninstallation script
├── com.github.nvprime.conf       # D-Bus system policy
├── nvprime.service               # Systemd service unit
└── README.md                     # This file
```

## Installation Script

The `install.sh` script handles all system integration with proper root checking and error handling.

### Usage

```bash
# Install binaries and configuration
sudo ./install.sh install

# Install and enable the daemon service
sudo ./install.sh install-service

# Remove all installed files
sudo ./install.sh uninstall

# Show installation status
./install.sh status
```

### Features

- **Root Check:** Automatically verifies root privileges
- **Binary Validation:** Checks that release binaries exist before installing
- **Colored Output:** Clear visual feedback with color-coded messages
- **Error Handling:** Proper error messages and exit codes
- **Status Display:** Shows what's installed and service state
- **Idempotent:** Safe to run multiple times

### What Gets Installed

| Component | Source | Destination | Permissions |
|-----------|--------|-------------|-------------|
| Client binary | `target/release/nvprime` | `/usr/local/bin/nvprime` | `755` (rwxr-xr-x) |
| Daemon binary | `target/release/nvprime-sys` | `/usr/local/bin/nvprime-sys` | `755` (rwxr-xr-x) |
| D-Bus policy | `com.github.nvprime.conf` | `/usr/share/dbus-1/system.d/` | `644` (rw-r--r--) |
| Systemd service | `nvprime.service` | `/usr/lib/systemd/system/` | `644` (rw-r--r--) |

## Configuration Files

### D-Bus Policy (`com.github.nvprime.conf`)

Controls D-Bus access to the nvprime daemon.

```xml
<busconfig>
  <policy user="root">
    <allow own="com.github.nvprime"/>           <!-- Daemon ownership -->
  </policy>
  <policy context="default">
    <allow send_destination="com.github.nvprime"/>  <!-- User access -->
  </policy>
</busconfig>
```

**Purpose:**
- Allows the daemon to register on the system bus
- Allows any user to communicate with the daemon
- **Required** for D-Bus communication to work

### Systemd Service (`nvprime.service`)

Manages the nvprime daemon as a system service.

```ini
[Unit]
Description=NvPrime System Daemon
After=network.target

[Service]
Type=dbus
BusName=com.github.nvprime
ExecStart=/usr/local/bin/nvprime-sys
Restart=on-failure
User=root

[Install]
WantedBy=multi-user.target
```

**Features:**
- Runs as root for GPU/process management
- D-Bus activation support
- Automatic restart on failure
- Starts after network is available

## Using with Just

The installation script integrates with the `justfile` for convenient usage:

```bash
# Build and install
just install

# Build, install, and enable service
just install-service

# Remove everything
just uninstall

# Check installation status
just show-installed
```

## Manual Installation

If you prefer to install manually without the script:

```bash
# Build release binaries
cargo build --release

# Install binaries (requires root)
sudo install -Dm755 target/release/nvprime /usr/local/bin/nvprime
sudo install -Dm755 target/release/nvprime-sys /usr/local/bin/nvprime-sys

# Install D-Bus configuration
sudo install -Dm644 system/com.github.nvprime.conf /usr/share/dbus-1/system.d/com.github.nvprime.conf

# Install systemd service
sudo install -Dm644 system/nvprime.service /usr/lib/systemd/system/nvprime.service

# Reload systemd
sudo systemctl daemon-reload

# Enable and start service
sudo systemctl enable --now nvprime.service
```

## Troubleshooting

### Script says "must be run as root"

```bash
# Add sudo before the command
sudo ./system/install.sh install
```

### "Binary not found" error

```bash
# Build release binaries first
cargo build --release

# Then install
sudo ./system/install.sh install
```

### Service won't start

```bash
# Check service status
systemctl status nvprime.service

# View logs
journalctl -u nvprime.service -n 50

# Check D-Bus policy is installed
ls -l /usr/share/dbus-1/system.d/com.github.nvprime.conf
```

### D-Bus connection fails

```bash
# Ensure daemon is running
systemctl status nvprime.service

# Test D-Bus connection
busctl call com.github.nvprime /com/github/nvprime com.github.nvprime.Service ping
```

## Uninstallation

To completely remove nvprime from your system:

```bash
# Using the script (recommended)
sudo ./system/install.sh uninstall

# Or via just
just uninstall
```

This will:
1. Stop the daemon service
2. Disable the service
3. Remove binaries from `/usr/local/bin/`
4. Remove D-Bus policy from `/usr/share/dbus-1/system.d/`
5. Remove systemd service from `/usr/lib/systemd/system/`
6. Reload systemd daemon

## Integration with Package Managers

The installation script can be used as a reference for creating distribution packages:

### Arch Linux (PKGBUILD)

```bash
package() {
    install -Dm755 "$srcdir/target/release/nvprime" "$pkgdir/usr/local/bin/nvprime"
    install -Dm755 "$srcdir/target/release/nvprime-sys" "$pkgdir/usr/local/bin/nvprime-sys"
    install -Dm644 "$srcdir/system/com.github.nvprime.conf" "$pkgdir/usr/share/dbus-1/system.d/com.github.nvprime.conf"
    install -Dm644 "$srcdir/system/nvprime.service" "$pkgdir/usr/lib/systemd/system/nvprime.service"
}
```

### Debian/Ubuntu (.deb)

```bash
# debian/install
target/release/nvprime usr/local/bin/
target/release/nvprime-sys usr/local/bin/
system/com.github.nvprime.conf usr/share/dbus-1/system.d/
system/nvprime.service usr/lib/systemd/system/
```

## Security Considerations

- The daemon runs as **root** (required for GPU management and process priority)
- D-Bus policy allows **any user** to send commands (by design)
- No authentication required (suitable for single-user systems)
- For multi-user systems, consider adding polkit policies for finer access control

## See Also

- [CONFIGURATION.md](CONFIGURATION.md) - Detailed configuration documentation
- [DEVELOPMENT.md](DEVELOPMENT.md) - Just command reference
- [README.md](../README.md) - Project overview
