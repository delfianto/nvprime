[← Back to README](../README.md)

# System Integration Files

The `system/` directory contains the configuration files used by the Arch package and source installs.

## Directory Structure

```
system/
├── com.github.nvprime.conf       # D-Bus system policy
├── nvprime.service               # Systemd service unit
└── nvprime.toml                  # Default system-wide configuration
```

## Manual Installation Paths

| Component | Source | Destination | Permissions |
|-----------|--------|-------------|-------------|
| Client binary | `target/release/nvprime` | `/usr/local/bin/nvprime` | `755` (rwxr-xr-x) |
| Daemon binary | `target/release/nvprimed` | `/usr/local/bin/nvprimed` | `755` (rwxr-xr-x) |
| Configuration | `system/nvprime.toml` | `/etc/nvprime.toml` | `644` (rw-r--r--) |
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
ExecStart=/usr/bin/nvprimed
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

`just setup` installs the binaries in `/usr/local/bin`, adjusts the installed unit's `ExecStart` to match, and installs the D-Bus policy under `/etc`. It creates `/etc/nvprime.toml` if needed.

```bash
# Build, install, and enable service
just setup

# Remove source-installed binaries and service integration
just teardown

# Check installation status
just status
```

## Manual Installation

If you prefer to install manually without the script:

```bash
# Build release binaries
cargo build --release

# Install binaries (requires root)
sudo install -Dm755 target/release/nvprime /usr/local/bin/nvprime
sudo install -Dm755 target/release/nvprimed /usr/local/bin/nvprimed

# Install the default config if needed
if ! sudo test -e /etc/nvprime.toml; then
    sudo install -Dm644 system/nvprime.toml /etc/nvprime.toml
fi

# Install D-Bus configuration
sudo install -Dm644 system/com.github.nvprime.conf /usr/share/dbus-1/system.d/com.github.nvprime.conf

# Install the unit with the path for a manual install
sed 's|^ExecStart=/usr/bin/nvprimed$|ExecStart=/usr/local/bin/nvprimed|' system/nvprime.service |
    sudo install -Dm644 /dev/stdin /usr/lib/systemd/system/nvprime.service

# Reload systemd
sudo systemctl daemon-reload

# Enable and start service with the current binary
sudo systemctl enable nvprime.service
sudo systemctl restart nvprime.service
```

## Upgrading from `nvprime-sys`

Both programs now use `/etc/nvprime.toml`. To reuse settings from a previous `nvprime.conf`, copy them into the new system file or set `NVPRIME_CONFIG` to the desired TOML file. See the [configuration guide](configuration.md).

If `systemctl cat nvprime.service` shows an older local unit under `/etc/systemd/system/`, it overrides the Arch package's unit. Back it up and move it out of the unit search path to use the packaged unit, or update its `ExecStart` to the installed `nvprimed` path. Check local drop-ins for the old path as well. Run `sudo systemctl reenable nvprime.service` to update the enablement symlink, then `sudo systemctl restart nvprime.service`.

## Troubleshooting

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
busctl call com.github.nvprime /com/github/nvprime com.github.nvprime.Service Ping
```

## Uninstallation

For a source install made with `just setup`, run `just teardown` to stop the daemon and remove its binaries, unit, and D-Bus policy. Use `sudo pacman -R nvprime` for the Arch package.

For a manual install, disable and stop `nvprime.service` and remove the binaries, service unit, and D-Bus policy from the paths above. Reload systemd after removing the unit.

## Integration with Package Managers

### Arch Linux (PKGBUILD)

See the [Arch installation instructions](../README.md#arch-linux-x86_64). The package builds from the checkout, installs `nvprime` and `nvprimed` in `/usr/bin`, and uses `/usr/bin/nvprimed` in `nvprime.service`.

On a running system, the package stops `nvprime.service` before replacing the executable. If the service was running, it reloads systemd and starts the new daemon after extraction. Enable and start the service explicitly on a fresh install. Beginning with `0.4.0-1`, the installed pre-transaction hook can abort a later upgrade if systemd cannot stop the service.

### Debian/Ubuntu (.deb)

```bash
# debian/install
target/release/nvprime usr/bin/
target/release/nvprimed usr/bin/
system/nvprime.toml etc/
system/com.github.nvprime.conf usr/share/dbus-1/system.d/
system/nvprime.service usr/lib/systemd/system/
```

## Security Considerations

- The daemon runs as **root** (required for GPU management and process priority)
- D-Bus policy allows **any user** to send commands (by design)
- No authentication required (suitable for single-user systems)
- For multi-user systems, consider adding polkit policies for finer access control

## See Also

- [Configuration guide](configuration.md)
- [Development guide](development.md)
- [Project overview](../README.md)
