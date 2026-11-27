[← Back to README](../README.md)

# Configuration Guide

NvPrime uses a TOML-based configuration file to manage system tuning, game-specific settings, and environment variables.

## Configuration File Location

`nvprime` and `nvprimed` both read `/etc/nvprime.toml` by default. The Arch package and `just setup` install a valid TOML file with tuning disabled. Missing files and invalid TOML are reported with the file path.

Set `NVPRIME_CONFIG` to use a different TOML file for a command.

```sh
NVPRIME_CONFIG="$HOME/.config/nvprime.toml" nvprime ./game_executable
```

To override the daemon's config, run `sudo systemctl edit nvprime.service` and add this drop-in using a root-managed file.

```ini
[Service]
Environment=NVPRIME_CONFIG=/etc/nvprime-custom.toml
```

Restart `nvprime.service` after changing its configuration. Set `gpu.gpu_tuning` in the daemon config to initialize GPU tuning, and set `gpu.gpu_uuid` there to choose the GPU. If you use a separate client config, enable GPU tuning there to request it for a launch.

## Structure

The configuration is divided into several sections:

- `[cpu]`: AMD CPU performance tuning.
- `[gpu]`: NVIDIA GPU power and identity management.
- `[sys]`: System-level process priority and hacks.
- `[launch]`: Global launch settings and environment overrides.
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
| `gpu_vlk_icd`    | string  | `""`                                      | Optional Vulkan driver manifest path, passed as `VK_DRIVER_FILES`. |
| `set_max_pwr`    | bool    | `false`                                   | Force maximum power limit.                         |
| `pwr_limit_tune` | integer | `None`                                    | Specific power limit in milliwatts (e.g., 350000). |

### System Tuning `[sys]`

Process priority and system-level hacks.

| Option                  | Type    | Default | Description                                         |
| ----------------------- | ------- | ------- | --------------------------------------------------- |
| `sys_tuning`            | bool    | `false` | Enable system tuning.                               |
| `proc_ioprio`           | integer | `4`     | IO priority (0-7, lower is higher priority).        |
| `proc_renice`           | integer | `0`     | CPU niceness (-20 to 19, lower is higher priority). |
| `splitlock_hack`        | bool    | `false` | Enable split-lock detection mitigation.             |
| `watchdog_interval_sec` | integer | `10`    | Interval for process monitoring (seconds).          |

### Game Specific Config `[game.<name>]`

Settings applied only when running a specific game executable.

**Important:** `<name>` is **NOT** an alias. It must match the game's executable name (case-insensitive, without extension).

- `Cyberpunk2077.exe` -> `[game.cyberpunk2077]`

- `dota2` -> `[game.dota2]`

| Option               | Type   | Default | Description                                 |
| -------------------- | ------ | ------- | ------------------------------------------- |
| `mangohud`           | bool   | Inherit | Enable MangoHud overlay.                    |
| `mangohud_conf`      | string | `None`  | Custom MangoHud configuration string.       |
| `proton_log`         | bool   | Inherit | Enable Proton logging (`PROTON_LOG=1`).     |
| `proton_ntsync`      | bool   | `true`  | Use NTSync on supported Proton and Linux versions. |
| `proton_wayland`     | bool   | `false` | Request Proton's native Wayland driver on supported builds. |
| `wine_dll_overrides` | string | `None`  | Set `WINEDLLOVERRIDES`.                     |

NvPrime defaults to NTSync and XWayland, including for games with settings such as `mangohud`. To enable native Wayland globally, set `wayland = true` under `[launch.proton]`. To enable it for a single game, set `proton_wayland = true` in its game section. Set `proton_ntsync = false` there if the game needs an NTSync workaround.

```toml
[game.example]
proton_ntsync = false
proton_wayland = true
```

Proton 11 uses NTSync automatically when supported. NvPrime sets `PROTON_USE_NTSYNC=1` by default for older builds that require it. An explicit `proton_ntsync = false` sets `PROTON_NO_NTSYNC=1` for newer builds and `PROTON_USE_NTSYNC=0` for older builds. When the setting is omitted, an inherited `PROTON_NO_NTSYNC` value is still honored by Proton. [Proton-CachyOS NTSync change](https://github.com/CachyOS/proton-cachyos/blob/cachyos_main/CHANGELOG.md#version-110-20260428)

An explicit Wayland setting sets both `PROTON_ENABLE_WAYLAND` and its `PROTON_USE_WAYLAND` alias. Native Wayland requires a Proton build with the driver and a Wayland session. [Proton-CachyOS Wayland options](https://github.com/CachyOS/proton-cachyos#proton-cachyos-config-options)

### Hooks `[hook]`

Shell commands to execute before starting and after finishing the game.

| Option     | Type   | Default | Description                       |
| ---------- | ------ | ------- | --------------------------------- |
| `init`     | string | `None`  | Command to run before game start. |
| `shutdown` | string | `None`  | Command to run after game exit.   |

### Built-in Launch Environment

NvPrime sets these launch defaults. Vulkan uses normal driver discovery, and `[gpu].gpu_name` still selects a GPU for DXVK and VKD3D-Proton. In the `v0.3.0` release, `PROTON_USE_NTSYNC` and `PROTON_ENABLE_WAYLAND` defaulted to `0`.

| Variable | Built-in value |
| --- | --- |
| `PROTON_USE_NTSYNC` | `1` |
| `PROTON_ENABLE_WAYLAND`, `PROTON_USE_WAYLAND` | `0` |
| `__NV_PRIME_RENDER_OFFLOAD` | `1` |
| `__GLX_VENDOR_LIBRARY_NAME` | `nvidia` |
| `__VK_LAYER_NV_optimus` | `NVIDIA_only` |
| `DXVK_LOG_LEVEL`, `DXVK_NVAPI_LOG_LEVEL`, `DXVK_NVAPI_VKREFLEX_LAYER_LOG_LEVEL` | `info` |
| `VKD3D_DEBUG`, `VKD3D_SHADER_DEBUG` | `info` |
| `WINEDEBUG` | `-all,err+all` |

The Wine filter keeps errors and suppresses `fixme`, warning, and trace messages. Enable Proton's log file with `PROTON_LOG=1 nvprime %command%` in Steam or `log = true` in `[launch.proton]` or `[game.<name>.launch.proton]`. To add warnings for a specific Wine channel, set `WINEDEBUG` in `[launch.env]` or `[game.<name>.launch.env]` to a filter such as `"-all,err+all,warn+dll"`. [Wine manual](https://man.archlinux.org/man/wine.1.en#WINEDEBUG)

### Launch Settings `[launch]`

Set global preferences under `[launch]` and use the same tables under `[game.<name>.launch]` for a specific executable. Omitted settings inherit the next available value. For optional features such as MangoHud, DLSS and Proton log file creation, an omitted setting can come from the shell, Proton's `user_settings.py`, or the application's default. An explicit `false` sends the feature's disabled value.

| Table | Options |
| --- | --- |
| `launch.proton` | `ntsync`, `wayland`, `log`, and `game_drive` accept booleans. `log_dir` and `wine_dll_overrides` accept strings. |
| `launch.mangohud` | `enabled` accepts a boolean. `config` accepts a MangoHud configuration string. |
| `launch.experimental.dlss` | `super_resolution_preset` and `ray_reconstruction_preset` accept NVAPI preset strings such as `"RENDER_PRESET_K"`, `"RENDER_PRESET_F"`, or `"render_preset_latest"`. Each also turns on its matching DLL/snippet override. `streamline_override` accepts a boolean. |
| `launch.experimental.nvidia` | `experimental_perf_strategy`, `gsync_allowed`, and `vrr_allowed` accept booleans. `max_frames_allowed` accepts an unsigned integer. `yield_mode` accepts `"nothing"` or `"usleep"`. |
| `launch.env` | Raw variables for options such as `VKD3D_CONFIG`, `VKD3D_SWAPCHAIN_PRESENT_MODE`, or `PROTON_DLSS_UPGRADE`. A boolean is sent as `1` or `0`. For NVAPI's `on` and `off` values, use strings. |

```toml
[launch.proton]
log_dir = "/tmp"

[game.example.launch.mangohud]
enabled = true

[game.example.launch.proton]
wayland = true

[game.example.launch.experimental.dlss]
ray_reconstruction_preset = "RENDER_PRESET_F"
streamline_override = true

[game.example.launch.env]
VKD3D_SWAPCHAIN_PRESENT_MODE = "FIFO_LATEST_READY"
```

For a key set in several places, the last matching source wins. NvPrime applies the built-in and GPU settings, global typed settings, `[launch.env]`, legacy game fields, typed game settings, the legacy executable environment table, and finally `[game.<name>.launch.env]`. These values override matching shell keys. Proton-CachyOS then fills missing keys from `user_settings.py`. [Launch environment source](../src/runner/env_var.rs)

### Environment Groups

A top-level table named after the detected executable stem remains supported. New configurations can use `[game.<name>.launch.env]`. This example sets a raw override when Steam invokes `REDprelauncher.exe`.

```toml
[game.redprelauncher.launch.env]
DXVK_NVAPI_DRS_NGX_DLSS_SR_OVERRIDE = "on"
DXVK_NVAPI_DRS_NGX_DLSS_SR_OVERRIDE_RENDER_PRESET_SELECTION = "RENDER_PRESET_K"
```

For Proton launches, NvPrime looks for the first `.exe` argument after `waitforexitandrun`. Otherwise it uses the last `.exe` argument or the launch command if there is no `.exe` argument. It lowercases the name and drops the extension for both `[game.<name>]` and environment group lookups.

## Proton launch flag audit

Checked on 2026-11-24 against Proton-CachyOS `11.0-20261015-slr`, its bundled DXVK-NVAPI `0f26995` and VKD3D-Proton `0bb64c5`, and NVIDIA driver `620.73.04`. The Vulkan checks used an RTX 4080 with the display connected to an Arc A310. The audit covers the earlier shared Proton settings and the NvPrime `v0.3.0` defaults that motivated the cleanup. [Proton-CachyOS release notes](https://github.com/CachyOS/proton-cachyos/blob/cachyos_main/CHANGELOG.md#version-110-20261015)

### DLSS Super Resolution and Ray Reconstruction presets

`DXVK_NVAPI_DRS_NGX_DLSS_RR_OVERRIDE_RENDER_PRESET_SELECTION` supplies a driver-profile value for the **Ray Reconstruction** model. It can affect a game that uses Ray Reconstruction, such as The Witcher 3 Remastered. `DXVK_NVAPI_DRS_NGX_DLSS_RR_OVERRIDE=on` enables the separate Ray Reconstruction DLL/snippet override. The equivalent Super Resolution variables have `SR` in place of `RR`. Model presets, in-game quality modes, and DLL updates are separate controls. [DXVK-NVAPI driver settings](https://github.com/jp7677/dxvk-nvapi/wiki/Passing-driver-settings#override-dlss-rr-preset) · [NVIDIA's Proton DLSS guide](https://docs.nvidia.com/datacenter/tesla/driver-installation-guide/620/gaming.html#dlss-settings-override-steam-play-proton) · [The Witcher 3 Remastered patch notes](https://www.thewitcher.com/no/en/news/52041/see-whats-new-in-the-witcher-3-wild-hunt-remastered)

- **Super Resolution `RENDER_PRESET_K`** pins the first-generation DLSS 4 transformer model. In January 2025, K was NVIDIA's “Latest” Super Resolution choice, so pinning it made sense for a consistent DLSS 4 model across games. For DLSS 4.5, NVIDIA's recommended Super Resolution policy uses M for Performance, L for Ultra Performance, and K for the other modes. [DLSS 4 override history](https://nvidia.custhelp.com/app/answers/detail/a_id/5620) · [DLSS 4.5 Super Resolution](https://www.nvidia.com/en-ph/geforce/news/nvidia-app-dlss-4-5-dynamic-multi-frame-generation-available-now/)
- **Ray Reconstruction `RENDER_PRESET_K`** is listed as a recognized NVAPI value, and the Ray Reconstruction SDK says that K falls back to default behavior. NVIDIA recommends **Preset F** or its recommended policy for supported DLSS 4.5 Ray Reconstruction titles. DXVK-NVAPI documents `render_preset_latest` as the recommended policy for DLSS 4.5. The actual model used still depends on the game's DLSS integration and available DLLs. [NVAPI setting values](https://docs.nvidia.com/nvapi/NvApiDriverSettings_8h.html) · [Ray Reconstruction SDK presets](https://github.com/NVIDIA/DLSS/blob/main/include/nvsdk_ngx_defs_dlssd.h) · [DLSS 4.5 Ray Reconstruction](https://www.nvidia.com/en-us/geforce/news/gamescom-2026-dlss-4-5-ray-reconstruction-release-announcements-trailers/)

Two precedence rules apply. Proton-CachyOS uses `user_settings.py` to fill missing keys, so the hard-coded SR preset in NvPrime `v0.3.0` replaced the file's `RENDER_PRESET_K`. Separately, DXVK-NVAPI gives dedicated variables priority over matching entries in `DXVK_NVAPI_DRS_SETTINGS`. The checked Proton-CachyOS build seeds that combined variable after a successful `PROTON_DLSS_UPGRADE`, with SR and RR presets set to `default`. An explicit NvPrime preset still takes precedence over that combined setting. [NvPrime environment precedence](#launch-settings-launch) · [DXVK-NVAPI precedence](https://github.com/jp7677/dxvk-nvapi/wiki/Passing-driver-settings)

The Witcher 3 Remastered ships NVIDIA Streamline. For an intentional preset override in a Streamline title, NVIDIA documents `DXVK_NVAPI_DRS_SL_DLSS_OVERRIDE=on` on supported DXVK-NVAPI builds. The DLSS indicator can confirm the model and DLL the game uses at runtime. Its setting persists in the Wine prefix. [NVIDIA Streamline overrides and DLSS indicator](https://docs.nvidia.com/datacenter/tesla/driver-installation-guide/620/gaming.html#dlss-settings-override-steam-play-proton)

### Proton settings file

| Setting | Audit |
| --- | --- |
| `MANGOHUD=1` | Valid Vulkan overlay switch. For an NvPrime launch, use the per-game `mangohud` option if that is where the overlay is configured. [MangoHud instructions](https://github.com/flightlessmango/MangoHud#normal-usage) |
| `PROTON_LOG=0` | Redundant for normal Proton launches. Enable logging on demand. The `v0.3.0` NvPrime default took precedence over a value in `user_settings.py`. [Proton runtime options](https://github.com/ValveSoftware/Proton#runtime-config-options) |
| `PROTON_LOG_DIR=/tmp` | Valid location preference for Proton logs when logging is enabled. |
| `PROTON_SET_GAME_DRIVE=1` | Maps the game's Steam library to `S:`. The checked CachyOS build enables this for most titles and has exceptions. A global `1` can override those exceptions. |
| `WINEDLLOVERRIDES=d3dcompiler_47=n,b;dxgi=n,b` | Still valid. Wine tries native DLLs first and then built-ins. Scope this to a game with a mod or DLL that needs that load order, such as a ReShade installation that uses `dxgi.dll`. [Wine DLL override reference](https://man.archlinux.org/man/wine.1.en#WINEDLLOVERRIDES) |
| `VKD3D_CONFIG=descriptor_heap` | Still an active **opt-in** on the checked build and upstream v3.1. It enables the descriptor-heap path when supported. The RTX 4080 advertises the extension. Use it as a per-game performance test. [Bundled VKD3D-Proton extension gate](https://github.com/HansKristian-Work/vkd3d-proton/blob/0bb64c56c8f0dcc11ecc0f6c56e00e4f7bd8d506/libs/vkd3d/device.c#L144) · [v3.1 release notes](https://github.com/HansKristian-Work/vkd3d-proton/releases/tag/v3.1) |
| `VKD3D_SWAPCHAIN_PRESENT_MODE=FIFO_LATEST_READY` | Still active. The checked RTX 4080 exposes it on Wayland and Xwayland. VKD3D-Proton forces the mode when the surface supports it and falls back to normal selection otherwise. Use it as a per-game frame-pacing preference. [Bundled swapchain selection](https://github.com/HansKristian-Work/vkd3d-proton/blob/0bb64c56c8f0dcc11ecc0f6c56e00e4f7bd8d506/libs/vkd3d/swapchain.c#L2137) · [Vulkan present mode reference](https://docs.vulkan.org/refpages/latest/refpages/source/VkPresentModeKHR.html) |
| `PROTON_DLSS_UPGRADE=1` | Still active in CachyOS. It selects a replacement DLSS DLL version. Use it for titles that benefit from the replacement. R620's NGX updater is another option on Proton builds that support it. [CachyOS options](https://github.com/CachyOS/proton-cachyos#proton-cachyos-config-options) · [NVIDIA R620 updater](https://docs.nvidia.com/datacenter/tesla/driver-installation-guide/620/gaming.html#dlss-update-steam-play-proton) |
| `DXVK_NVAPI_DRS_NGX_DLSS_SR_OVERRIDE=on` | Still supported. The CachyOS DLSS upgrade already supplies the equivalent combined setting after a successful DLL setup. An explicit NvPrime SR preset also sets the dedicated variable. |
| `DXVK_NVAPI_DRS_NGX_DLSS_SR_OVERRIDE_RENDER_PRESET_SELECTION=RENDER_PRESET_K` | A valid per-game Super Resolution preference. Use `super_resolution_preset` where a deliberate K pin matters. |
| `PROTON_ENABLE_WAYLAND=1` | An active option in CachyOS. Keep it for direct Proton launches if native Wayland is desired. NvPrime defaults to XWayland and supports opting in to native Wayland globally or per game. [CachyOS Wayland options](https://github.com/CachyOS/proton-cachyos#proton-cachyos-config-options) |
| `PROTON_HIDE_NVIDIA_GPU=0` | Generally redundant. The checked build applies an NVIDIA-hiding workaround to specific titles such as The Crew Motorfest. A global `0` can undo that workaround. |
| `PROTON_NO_WM_DECORATION=0` | Generally redundant when using the normal window-decoration policy. Keep a per-game override for a specific window-management issue. [CachyOS decoration options](https://github.com/CachyOS/proton-cachyos#proton-cachyos-config-options) |
| `PROTON_NVIDIA_LIBS=1` | Still supported as a compatibility option. It adds CUDA, NVENC, NVML, and OptiX libraries in CachyOS. Use it for titles that need one of those libraries, such as hardware PhysX games. Ordinary PRIME offload and DLSS via DXVK-NVAPI already use their own paths. [CachyOS NVIDIA library options](https://github.com/CachyOS/proton-cachyos#proton-cachyos-config-options) · [NVIDIA Vulkan and DLSS guide](https://docs.nvidia.com/datacenter/tesla/driver-installation-guide/620/gaming.html) |

### Earlier NvPrime defaults

| NvPrime variables | Audit |
| --- | --- |
| `__NV_PRIME_RENDER_OFFLOAD`, `__GLX_VENDOR_LIBRARY_NAME`, `__VK_LAYER_NV_optimus` | Keep for offload. They request the NVIDIA offload path, select NVIDIA for GLX, and prefer NVIDIA devices in Vulkan. `gpu_name` sets `DXVK_FILTER_DEVICE_NAME` and `VKD3D_FILTER_DEVICE_NAME` to pick a specific card when more than one NVIDIA GPU is visible. [NVIDIA PRIME guide](https://download.nvidia.com/XFree86/Linux-x86_64/620.73.04/README/primerenderoffload.html) · [DXVK filtering](https://github.com/doitsujin/dxvk#device-filtering) · [VKD3D-Proton environment](https://github.com/HansKristian-Work/vkd3d-proton#environment-variables) |
| `VK_ICD_FILENAMES` | Deprecated by the Vulkan Loader. Normal discovery with PRIME enabled enumerated the RTX 4080 first on the checked machine. Keep an explicit manifest override as an opt-in where needed and use `VK_DRIVER_FILES` on current loaders. [Khronos driver discovery](https://github.com/KhronosGroup/Vulkan-Loader/blob/main/docs/LoaderDriverInterface.md#overriding-the-default-driver-discovery) |
| `MANGOHUD`, `MANGOHUD_CONFIG` | Active overlay settings. Make the overlay and `preset=1` preferences explicit so the launch can otherwise inherit the user's settings. [MangoHud configuration](https://github.com/flightlessmango/MangoHud#hud-configuration) |
| `PROTON_LOG`, `DXVK_LOG_LEVEL`, `DXVK_NVAPI_LOG_LEVEL`, `DXVK_NVAPI_VKREFLEX_LAYER_LOG_LEVEL`, `VKD3D_DEBUG`, `VKD3D_SHADER_DEBUG`, `WINEDEBUG` | Proton log file creation is opt-in. NvPrime supplies Wine, DXVK, NVAPI, and VKD3D verbosity defaults as listed above. [Proton runtime options](https://github.com/ValveSoftware/Proton#runtime-config-options) · [DXVK-NVAPI logging](https://github.com/jp7677/dxvk-nvapi#logging) · [VKD3D-Proton environment](https://github.com/HansKristian-Work/vkd3d-proton#environment-variables) |
| `PROTON_USE_NTSYNC`, `PROTON_ENABLE_WAYLAND` | The NTSync enable flag is legacy compatibility. Proton 11 auto-enables NTSync where supported and uses `PROTON_NO_NTSYNC` for opt-outs. Request Wayland on builds that support it and keep both per-game workarounds. [CachyOS NTSync change](https://github.com/CachyOS/proton-cachyos/blob/cachyos_main/CHANGELOG.md#version-110-20260428) |
| `PROTON_SET_GAME_DRIVE` | Treat the `S:` mapping as an explicit preference. Current CachyOS already enables it for most titles. |
| `DXVK_NVAPI_SET_NGX_DEBUG_OPTIONS` | A debugging control that writes NGX registry values in the Wine prefix. The old default wrote values that hid the indicators. Use an explicit environment override to change the prefix's preference. [DXVK-NVAPI registry implementation](https://github.com/jp7677/dxvk-nvapi/blob/0f26995d8126617d4d99f62c9571715fc7765da1/src/util/util_ngx_debug.h) |
| `DXVK_NVAPI_DRS_NGX_DLSS_SR_OVERRIDE`, `DXVK_NVAPI_DRS_NGX_DLSS_RR_OVERRIDE`, `DXVK_NVAPI_DRS_NGX_DLSS_SR_OVERRIDE_RENDER_PRESET_SELECTION`, `DXVK_NVAPI_DRS_NGX_DLSS_RR_OVERRIDE_RENDER_PRESET_SELECTION` | Supported. Make them per-game opt-ins so games and Proton can select their own models by default. [DXVK-NVAPI driver settings](https://github.com/jp7677/dxvk-nvapi/wiki/Passing-driver-settings) |
| `__GL_ExperimentalPerfStrategy`, `__GL_MaxFramesAllowed` | The R620 public OpenGL guide gives no contract for these knobs. They are available as opt-ins for testing a repeatable workload. [NVIDIA R620 OpenGL variables](https://download.nvidia.com/XFree86/Linux-x86_64/620.73.04/README/openglenvvariables.html) |
| `__GL_GSYNC_ALLOWED`, `__GL_VRR_ALLOWED` | Driver-side VRR controls. The R620 guide documents `__GL_VRR_ALLOWED` for compatible X screens and displays. For a Wayland desktop scanned out by the Arc, use the compositor's display policy. [NVIDIA R620 OpenGL variables](https://download.nvidia.com/XFree86/Linux-x86_64/620.73.04/README/openglenvvariables.html) |
| `__GL_YIELD=USLEEP` | Still documented as an NVIDIA OpenGL scheduling workaround. Scope it to a reproduced OpenGL issue. [NVIDIA R620 OpenGL variables](https://download.nvidia.com/XFree86/Linux-x86_64/620.73.04/README/openglenvvariables.html) |

## Annotated Configuration Example

```toml
[cpu]
cpu_tuning = true                           # Enable CPU tuning
amd_epp_tune = "performance"                # EPP hint when game starts
amd_epp_base = "balance_performance"        # EPP hint when game ends

[gpu]
gpu_tuning = true                           # Enable GPU tuning
gpu_name = "NVIDIA GeForce RTX 4080"        # Vulkan device name, this is for PRIME offload
gpu_uuid = "GPU-7e...2b"                    # GPU UUID from nvidia-smi -L
set_max_pwr = false                         # Set true to force the maximum
pwr_limit_tune = 350000                     # Custom power limit in milliwatts

[sys]
sys_tuning = true                           # Enable system tuning
proc_ioprio = 4                             # I/O priority (0-7, lower=higher)
proc_renice = 10                            # Nice value (-20 to 19, lower=higher)
splitlock_hack = true                       # Split-lock mitigation
watchdog_interval_sec = 10                  # Polling interval for cleanup (seconds)

[game.ffxvi]                                # Matches ffxvi.exe
mangohud = true                             # Enable MangoHud
wine_dll_overrides = "dinput8=n,b"          # Set WINEDLLOVERRIDES, example for widescreen hack
```
