[← Back to README](../README.md)

# Configuration Guide

NvPrime uses a TOML-based configuration file to manage system tuning, game-specific settings, and environment variables.

## Configuration File Location

The configuration file is expected to be at:
`~/.config/nvprime/nvprime.conf`

## Structure

The configuration is divided into several sections:

- `[cpu]`: AMD CPU performance tuning.
- `[gpu]`: NVIDIA GPU power and identity management.
- `[sys]`: System-level process priority and hacks.
- `[game.<name>]`: Per-game overrides and settings.
- `[hook]`: Custom scripts to run at start/stop.
- `[<custom_env_group>]`: Groups of environment variables to apply.

### CPU Tuning `[cpu]`

Controls AMD Zen Energy Performance Preference (EPP).

| Option         | Type   | Default                 | Description                         |
| -------------- | ------ | ----------------------- | ----------------------------------- |
| `cpu_tuning`   | bool   | `false`                 | Enable CPU tuning.                  |
| `amd_epp_tune` | string | `"performance"`         | EPP hint to apply when game starts. |
| `amd_epp_base` | string | `"balance_performance"` | EPP hint to restore when game ends. |

### GPU Tuning `[gpu]`

Controls NVIDIA GPU settings. Requires the daemon to be running.

| Option           | Type    | Default                                   | Description                                        |
| ---------------- | ------- | ----------------------------------------- | -------------------------------------------------- |
| `gpu_tuning`     | bool    | `false`                                   | Enable GPU tuning.                                 |
| `gpu_name`       | string  | `None`                                    | Vulkan device name (used for filtering).           |
| `gpu_uuid`       | string  | `None`                                    | GPU UUID (from `nvidia-smi -L`).                   |
| `gpu_vlk_icd`    | string  | `/usr/share/vulkan/icd.d/nvidia_icd.json` | Path to Vulkan ICD.                                |
| `set_max_pwr`    | bool    | `false`                                   | Force maximum power limit.                         |
| `pwr_limit_tune` | integer | `None`                                    | Specific power limit in milliwatts (e.g., 350000). |

### System Tuning `[sys]`

Process priority and system-level hacks.

| Option           | Type    | Default | Description                                         |
| ---------------- | ------- | ------- | --------------------------------------------------- |
| `sys_tuning`     | bool    | `false` | Enable system tuning.                               |
| `proc_ioprio`    | integer | `4`     | IO priority (0-7, lower is higher priority).        |
| `proc_renice`    | integer | `0`     | CPU niceness (-20 to 19, lower is higher priority). |
| `splitlock_hack` | bool    | `false` | Enable split-lock detection mitigation.             |

### Game Specific Config `[game.<name>]`

Settings applied only when running a specific game executable.

**Important:** `<name>` is **NOT** an alias. It must match the game's executable name (case-insensitive, without extension).

- `Cyberpunk2077.exe` -> `[game.cyberpunk2077]`

- `dota2` -> `[game.dota2]`

| Option               | Type   | Default | Description                                 |
| -------------------- | ------ | ------- | ------------------------------------------- |
| `mangohud`           | bool   | `false` | Enable MangoHud overlay.                    |
| `mangohud_conf`      | string | `None`  | Custom MangoHud configuration string.       |
| `proton_log`         | bool   | `false` | Enable Proton logging (`PROTON_LOG=1`).     |
| `proton_ntsync`      | bool   | `false` | Enable `PROTON_USE_WINE_ESYNC=1` / NT sync. |
| `proton_wayland`     | bool   | `false` | Enable Wayland driver for Proton.           |
| `wine_dll_overrides` | string | `None`  | Set `WINEDLLOVERRIDES`.                     |

### Hooks `[hook]`

Shell commands to execute before starting and after finishing the game.

| Option     | Type   | Default | Description                       |
| ---------- | ------ | ------- | --------------------------------- |
| `init`     | string | `None`  | Command to run before game start. |
| `shutdown` | string | `None`  | Command to run after game exit.   |

### Environment Groups

Any other top-level section is treated as a group of environment variables.
These are applied when the section name is passed as an argument or matched.

## Annotated Configuration Example

```toml
[cpu]
cpu_tuning = true                           # Enable CPU tuning
amd_epp_tune = "performance"                # EPP hint when game starts
amd_epp_base = "balanced_performance"       # EPP hint when game ends

[gpu]
gpu_tuning = true                           # Enable GPU tuning
gpu_name = "NVIDIA GeForce RTX 4080"        # Vulkan device name, this is for PRIME offload
gpu_uuid = "GPU-7e...2b"                    # GPU UUID from nvidia-smi -L
set_max_pwr = true                          # Force max power limit
pwr_limit_tune = 350                        # Or set specific limit (Watts)

[sys]
sys_tuning = true                           # Enable system tuning
proc_ioprio = 10                            # I/O priority (0-7, lower=higher)
proc_renice = 10                            # Nice value (-20 to 19, lower=higher)
splitlock_hack = true                       # Split-lock mitigation

[game.ffxvi]                                # Run with: nvprime run ffxvi
mangohud = true                             # Enable MangoHud
proton_ntsync = true                        # Enable Proton NT Sync
wine_dll_overrides = "dinput8=n,b"          # Set WINEDLLOVERRIDES, example for widescreen hack
```
