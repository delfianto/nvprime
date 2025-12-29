# nvprime

**nvprime** is a simple tool for tuning your gaming session without the hassle of endlessly editing Steam launch arguments or juggling multiple scripts. It handles things like GPU power limits, CPU energy profiles, and process priorities so you don't have to.

## Features

- **Automated Tuning:** Automatically applies high-performance power profiles to AMD CPUs and NVIDIA GPUs when a game starts.
- **Process Priority:** Increases process priority (renice) and I/O priority for the game process.
- **Environment Management:** Easily manages per-game environment variables (Proton, MangoHud, Wayland overrides).
- **Daemon-Client Architecture:** Securely performs privileged operations (like power management) via a D-Bus daemon, eliminating the need for `sudo` on every launch.
- **Hooks:** Run custom scripts on game start and exit.

## Documentation

- **[Installation Guide](docs/INSTALLATION.md)**: System requirements and installation steps.
- **[Configuration Guide](docs/CONFIGURATION.md)**: Detailed reference for `nvprime.conf` options.
- **[Development & Usage](docs/DEVELOPMENT.md)**: How to build, test, and use the development tools.
- **[System Integration](docs/SYSTEM_CONFIG.md)**: Details on Systemd and D-Bus integration.

## Quick Start

### Installation

```bash
# Install binaries and system services
just install

# Enable the daemon
just install-service
```

### Usage

To apply tuning to a game, simply prefix the command with `nvprime`.

**For Steam Games:**
Set the game's Launch Options to:
```bash
nvprime %command%
```
This is identical to how `gamemoderun` works. `nvprime` will automatically detect the game executable, apply the correct configuration (looking for `[game.executablename]`), and inject necessary environment variables.

**For non-Steam games:**
```bash
nvprime ./game_executable
```

**Note:** This tool is primarily tested with Steam games. Non-Steam games are currently untested.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
