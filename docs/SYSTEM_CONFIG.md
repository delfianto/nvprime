[← Back to README](../README.md)

# System Configuration Files

## Required Configuration Files

### D-Bus Policy (REQUIRED)

**File:** `dbus/com.github.nvprime.conf`
**Install to:** `/usr/share/dbus-1/system.d/com.github.nvprime.conf`

```xml
<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <policy user="root">
    <allow own="com.github.nvprime"/>
  </policy>
  <policy context="default">
    <allow send_destination="com.github.nvprime"/>
  </policy>
</busconfig>
```

**Purpose:**
- Allows the daemon (running as root) to own the D-Bus name `com.github.nvprime`
- Allows any user to send messages to the daemon
- This is **essential** for D-Bus communication to work

**Without this file:**
- The daemon cannot register on the system bus
- Clients cannot communicate with the daemon
- You'll see "Failed to connect to system bus" errors

### Systemd Service (REQUIRED)

**File:** `systemd/nvprime.service`
**Install to:** `/usr/lib/systemd/system/nvprime.service`

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

**Purpose:**
- Runs the daemon as a system service
- Ensures it starts on boot
- Manages the daemon lifecycle
- Integrates with D-Bus activation

## What We DON'T Need

### Polkit Policy Files (.policy)

We **do not** need polkit because:

1. **No Interactive Elevation:** The daemon runs as root permanently via systemd
2. **No Password Prompts:** Users don't need to authenticate to use the client
3. **Simple Permission Model:** D-Bus policy handles all access control

**Example of what we don't need:**
```xml
<!-- We DON'T need this -->
<policyconfig>
  <action id="com.github.nvprime.apply-tuning">
    <message>Authentication required to apply GPU tuning</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
    </defaults>
  </action>
</policyconfig>
```

## Architecture Summary

```
┌─────────────────────────────────────────────────┐
│  nvprime (client)                               │
│  Runs as: Current user                          │
│  Permissions: None required                     │
└──────────────────┬──────────────────────────────┘
                   │
                   │ D-Bus System Bus
                   │ (controlled by dbus/com.github.nvprime.conf)
                   │
                   ▼
┌─────────────────────────────────────────────────┐
│  nvprime-sys (daemon)                           │
│  Runs as: root (via systemd)                    │
│  Permissions: Can modify GPU, set process nice  │
│  D-Bus name: com.github.nvprime                 │
└─────────────────────────────────────────────────┘
```

## Why This Approach?

### Traditional Approach (pkexec/sudo)
```
User runs command → pkexec prompt → password → elevated command
- Password prompts for every game launch
- User must be in sudoers
- No centralized state management
```

### Our D-Bus Approach
```
Daemon (always root) ← D-Bus → User client (no privileges)
- No password prompts
- Works for all users
- Centralized state management
- Automatic cleanup when games exit
```

## Installation

The `just install` command handles all configuration:

```bash
just install
```

This installs:
1. Binaries to `/usr/local/bin/`
2. D-Bus policy to `/usr/share/dbus-1/system.d/`
3. Systemd service to `/usr/lib/systemd/system/`

## Verification

After installation, verify:

```bash
# Check D-Bus policy is installed
ls -l /usr/share/dbus-1/system.d/com.github.nvprime.conf

# Check systemd service is installed
systemctl status nvprime.service

# Test D-Bus connection
just test-dbus
```

## Troubleshooting

### "Failed to claim bus name"
**Cause:** D-Bus policy file not installed
**Fix:** `sudo cp dbus/com.github.nvprime.conf /usr/share/dbus-1/system.d/`

### "Permission denied" when connecting
**Cause:** D-Bus policy doesn't allow user connections
**Fix:** Check the policy file has `<allow send_destination="com.github.nvprime"/>`

### Daemon won't start
**Cause:** Systemd service not properly configured
**Fix:** Check service file with `systemctl status nvprime.service`
