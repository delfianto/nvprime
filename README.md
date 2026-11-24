# nvprime

**nvprime** is a simple tool for tuning your gaming session without the hassle of endlessly editing Steam launch arguments or juggling multiple scripts. It handles things like GPU power limits, CPU energy profiles, and process priorities so you don't have to.

## Features

- **Automated Tuning:** Automatically applies high-performance power profiles to AMD CPUs and NVIDIA GPUs when a game starts.
- **Process Priority:** Increases process priority (renice) and I/O priority for the game process.
- **Environment Management:** Easily manages per-game environment variables (Proton, MangoHud, Wayland overrides).
- **Daemon-Client Architecture:** Securely performs privileged operations (like power management) via a D-Bus daemon, eliminating the need for `sudo` on every launch.
- **Hooks:** Run custom scripts on game start and exit.

## Documentation

- **[Installation Guide](docs/installation.md)** covers system requirements and installation.
- **[Configuration Guide](docs/configuration.md)** covers `nvprime.toml` options.
- **[Development & Usage](docs/development.md)** covers building, testing, and using the development tools.

## Quick Start

### Arch Linux (x86_64)

The package installs `nvprime` and `nvprimed` in `/usr/bin`, along with the systemd unit, D-Bus policy, and `/etc/nvprime.toml`. Pacman preserves local edits to the configuration on upgrades.

Build and install the package from the repository checkout.

```sh
git clone https://github.com/delfianto/nvprime.git
cd nvprime/arch
makepkg -si
sudo systemctl enable nvprime.service
sudo systemctl restart nvprime.service
```

If you previously installed a local systemd unit, review it before switching to the package. See the [migration steps](docs/installation.md#upgrading-from-nvprime-sys).

### Build from source

Source builds require Rust 1.100.0 or later.

```bash
# Install binaries, system services, and start the daemon
just setup
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

## Here Be Dragons

By using this software, you agree that if your GPU turns into a smoke machine, your CPU decides to cook an egg, or your computer achieves sentience and tries to take over the world, it is **not my fault**. This tool messes with power limits and hardware knobs; if you push your hardware beyond its limits, it might push back (by melting). You have been warned. Good luck, have fun, and may your framerates be high and your temperatures low.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
