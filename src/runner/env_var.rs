use crate::common::Config;
use crate::common::config::{EnvValue, LaunchConfig, LaunchTable, NvidiaYieldMode};
use log::debug;
use std::collections::BTreeMap;

const LOG: &str = "PROTON_LOG";
const HUD: &str = "MANGOHUD";
const HUD_CFG: &str = "MANGOHUD_CONFIG";
const NTSYNC: &str = "PROTON_USE_NTSYNC";
const NO_NTSYNC: &str = "PROTON_NO_NTSYNC";
const WAYLAND: &str = "PROTON_ENABLE_WAYLAND";
const WAYLAND_ALIAS: &str = "PROTON_USE_WAYLAND";
const DXVK_GPU: &str = "DXVK_FILTER_DEVICE_NAME";
const VKD3D_GPU: &str = "VKD3D_FILTER_DEVICE_NAME";
const WINE_DLLS: &str = "WINEDLLOVERRIDES";

const ENV_DEFAULTS: [(&str, &str); 11] = [
    ("__NV_PRIME_RENDER_OFFLOAD", "1"),
    ("__GLX_VENDOR_LIBRARY_NAME", "nvidia"),
    ("__VK_LAYER_NV_optimus", "NVIDIA_only"),
    (NTSYNC, "1"),
    (WAYLAND, "1"),
    ("DXVK_LOG_LEVEL", "info"),
    ("DXVK_NVAPI_LOG_LEVEL", "info"),
    ("DXVK_NVAPI_VKREFLEX_LAYER_LOG_LEVEL", "info"),
    ("VKD3D_DEBUG", "info"),
    ("VKD3D_SHADER_DEBUG", "info"),
    ("WINEDEBUG", "-all,err+all"),
];

pub struct EnvBuilder {
    vars: BTreeMap<String, String>,
}

impl EnvBuilder {
    pub fn new() -> Self {
        Self {
            vars: BTreeMap::from(ENV_DEFAULTS.map(|(key, value)| (key.into(), value.into()))),
        }
    }
}

impl Default for EnvBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvBuilder {
    fn set_str(&mut self, key: &str, value: &str) {
        self.vars.insert(key.to_string(), value.to_string());
    }

    fn set_bool(&mut self, key: &str, enabled: bool) {
        self.set_str(key, if enabled { "1" } else { "0" });
    }

    fn set_optional_bool(&mut self, key: &str, value: Option<bool>) {
        if let Some(enabled) = value {
            self.set_bool(key, enabled);
        }
    }

    fn set_optional_str(&mut self, key: &str, value: Option<&str>) {
        if let Some(value) = value {
            self.set_str(key, value);
        }
    }

    fn set_ntsync(&mut self, enabled: bool) {
        self.set_bool(NTSYNC, enabled);
        self.set_bool(NO_NTSYNC, !enabled);
    }

    fn set_wayland(&mut self, enabled: bool) {
        self.set_bool(WAYLAND, enabled);
        self.set_bool(WAYLAND_ALIAS, enabled);
    }

    fn apply_experimental(&mut self, launch: &LaunchConfig) {
        let dlss = &launch.experimental.dlss;
        if let Some(preset) = &dlss.super_resolution_preset {
            self.set_str("DXVK_NVAPI_DRS_NGX_DLSS_SR_OVERRIDE", "on");
            self.set_str(
                "DXVK_NVAPI_DRS_NGX_DLSS_SR_OVERRIDE_RENDER_PRESET_SELECTION",
                preset,
            );
        }
        if let Some(preset) = &dlss.ray_reconstruction_preset {
            self.set_str("DXVK_NVAPI_DRS_NGX_DLSS_RR_OVERRIDE", "on");
            self.set_str(
                "DXVK_NVAPI_DRS_NGX_DLSS_RR_OVERRIDE_RENDER_PRESET_SELECTION",
                preset,
            );
        }
        if let Some(enabled) = dlss.streamline_override {
            self.set_str(
                "DXVK_NVAPI_DRS_SL_DLSS_OVERRIDE",
                if enabled { "on" } else { "off" },
            );
        }

        let nvidia = &launch.experimental.nvidia;
        self.set_optional_bool(
            "__GL_ExperimentalPerfStrategy",
            nvidia.experimental_perf_strategy,
        );
        self.set_optional_bool("__GL_GSYNC_ALLOWED", nvidia.gsync_allowed);
        self.set_optional_bool("__GL_VRR_ALLOWED", nvidia.vrr_allowed);
        if let Some(frames) = nvidia.max_frames_allowed {
            self.set_str("__GL_MaxFramesAllowed", &frames.to_string());
        }
        if let Some(mode) = nvidia.yield_mode {
            let value = match mode {
                NvidiaYieldMode::Nothing => "NOTHING",
                NvidiaYieldMode::Usleep => "USLEEP",
            };
            self.set_str("__GL_YIELD", value);
        }
    }

    fn apply_launch(&mut self, launch: &LaunchConfig) {
        let proton = &launch.proton;
        if let Some(enabled) = proton.ntsync {
            self.set_ntsync(enabled);
        }
        if let Some(enabled) = proton.wayland {
            self.set_wayland(enabled);
        }
        self.set_optional_bool(LOG, proton.log);
        self.set_optional_str("PROTON_LOG_DIR", proton.log_dir.as_deref());
        self.set_optional_bool("PROTON_SET_GAME_DRIVE", proton.game_drive);
        self.set_optional_str(WINE_DLLS, proton.wine_dll_overrides.as_deref());

        self.set_optional_bool(HUD, launch.mangohud.enabled);
        self.set_optional_str(HUD_CFG, launch.mangohud.config.as_deref());
        self.apply_experimental(launch);
    }

    fn apply_env<'a>(
        &mut self,
        vars: impl IntoIterator<Item = (&'a String, &'a EnvValue)>,
        source: &str,
    ) {
        for (key, value) in vars {
            debug!("Applying {key} from {source}");
            self.set_str(key, &value.to_string());
        }
    }

    pub fn with_config(mut self, config: &Config, exe_name: &str) -> BTreeMap<String, String> {
        debug!("Initializing environment values for game: {exe_name}");

        if let Some(device) = &config.gpu.gpu_name {
            self = self.with_gpu_name(device);
        }
        if !config.gpu.gpu_vlk_icd.is_empty() {
            self.set_str("VK_DRIVER_FILES", &config.gpu.gpu_vlk_icd);
        }
        let legacy_launch_env = match &config.launch {
            Some(LaunchTable::Settings(launch)) => {
                debug!("Applying [launch] settings");
                self.apply_launch(launch);
                self.apply_env(&launch.env, "[launch.env]");
                None
            }
            Some(LaunchTable::ExecutableEnv(env)) if exe_name == "launch" => Some(env),
            _ => None,
        };

        let game = config.game.get(exe_name);
        if let Some(game) = game {
            debug!("Applying [game.{exe_name}] settings");
            self.set_optional_bool(HUD, game.mangohud);
            self.set_optional_bool(LOG, game.proton_log);
            if let Some(enabled) = game.proton_ntsync {
                self.set_ntsync(enabled);
            }
            if let Some(enabled) = game.proton_wayland {
                self.set_wayland(enabled);
            }
            self.set_optional_str(HUD_CFG, game.mangohud_conf.as_deref());
            self.set_optional_str(WINE_DLLS, game.wine_dll_overrides.as_deref());
            self.apply_launch(&game.launch);
        }

        if let Some(env) = config.env.get(exe_name) {
            self.apply_env(env, &format!("[{exe_name}]"));
        }
        if let Some(env) = legacy_launch_env {
            self.apply_env(env, "[launch] executable group");
        }
        if let Some(game) = game {
            self.apply_env(&game.launch.env, &format!("[game.{exe_name}.launch.env]"));
        }

        self.build()
    }

    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.set_str(key, value);
        self
    }

    pub fn with_bool(mut self, key: &str, enabled: bool) -> Self {
        self.set_bool(key, enabled);
        self
    }

    pub fn with_gpu_name(self, device: &str) -> Self {
        self.with_env(DXVK_GPU, device).with_env(VKD3D_GPU, device)
    }

    pub fn with_mangohud(self, enabled: bool) -> Self {
        self.with_bool(HUD, enabled)
    }

    pub fn with_log(self, enabled: bool) -> Self {
        self.with_bool(LOG, enabled)
    }

    pub fn with_ntsync(mut self, enabled: bool) -> Self {
        self.set_ntsync(enabled);
        self
    }

    pub fn with_wayland(mut self, enabled: bool) -> Self {
        self.set_wayland(enabled);
        self
    }

    pub fn with_dll_overrides(self, value: &str) -> Self {
        self.with_env(WINE_DLLS, value)
    }

    pub fn build(self) -> BTreeMap<String, String> {
        debug!(
            "Building final environment map with {} variables",
            self.vars.len()
        );
        self.vars
    }

    pub fn merge_global(&mut self, global: &BTreeMap<String, EnvValue>) {
        self.apply_env(global, "global settings");
    }

    pub fn merge_executable(&mut self, exe_vars: Option<&BTreeMap<String, EnvValue>>) {
        if let Some(vars) = exe_vars {
            self.apply_env(vars, "executable settings");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::config::Config;

    #[test]
    fn test_env_builder_new() {
        let builder = EnvBuilder::new();
        assert!(!builder.vars.is_empty());
        assert_eq!(
            builder.vars.get("__NV_PRIME_RENDER_OFFLOAD"),
            Some(&"1".to_string())
        );
        assert_eq!(
            builder.vars.get("__GLX_VENDOR_LIBRARY_NAME"),
            Some(&"nvidia".to_string())
        );
    }

    #[test]
    fn test_env_builder_with_env() {
        let builder = EnvBuilder::new().with_env("TEST_VAR", "test_value");
        let vars = builder.build();
        assert_eq!(vars.get("TEST_VAR"), Some(&"test_value".to_string()));
    }

    #[test]
    fn test_env_builder_with_bool() {
        let builder = EnvBuilder::new()
            .with_bool("ENABLED_FLAG", true)
            .with_bool("DISABLED_FLAG", false);
        let vars = builder.build();
        assert_eq!(vars.get("ENABLED_FLAG"), Some(&"1".to_string()));
        assert_eq!(vars.get("DISABLED_FLAG"), Some(&"0".to_string()));
    }

    #[test]
    fn test_env_builder_with_gpu_name() {
        let builder = EnvBuilder::new().with_gpu_name("NVIDIA RTX 4090");
        let vars = builder.build();
        assert_eq!(vars.get(DXVK_GPU), Some(&"NVIDIA RTX 4090".to_string()));
        assert_eq!(vars.get(VKD3D_GPU), Some(&"NVIDIA RTX 4090".to_string()));
    }

    #[test]
    fn test_env_builder_with_mangohud() {
        let builder = EnvBuilder::new().with_mangohud(true);
        let vars = builder.build();
        assert_eq!(vars.get(HUD), Some(&"1".to_string()));
    }

    #[test]
    fn test_env_builder_with_log() {
        let builder = EnvBuilder::new().with_log(true);
        let vars = builder.build();
        assert_eq!(vars.get(LOG), Some(&"1".to_string()));
    }

    #[test]
    fn test_env_builder_with_ntsync() {
        let builder = EnvBuilder::new().with_ntsync(true);
        let vars = builder.build();
        assert_eq!(vars.get(NTSYNC), Some(&"1".to_string()));
        assert_eq!(vars.get(NO_NTSYNC), Some(&"0".to_string()));
    }

    #[test]
    fn test_env_builder_with_wayland() {
        let builder = EnvBuilder::new().with_wayland(true);
        let vars = builder.build();
        assert_eq!(vars.get(WAYLAND), Some(&"1".to_string()));
        assert_eq!(vars.get(WAYLAND_ALIAS), Some(&"1".to_string()));
    }

    #[test]
    fn test_env_builder_with_dll_overrides() {
        let builder = EnvBuilder::new().with_dll_overrides("dinput8=n,b");
        let vars = builder.build();
        assert_eq!(vars.get(WINE_DLLS), Some(&"dinput8=n,b".to_string()));
    }

    #[test]
    fn test_env_builder_merge_global() {
        let mut builder = EnvBuilder::new();
        let mut global = BTreeMap::new();
        global.insert(
            "GLOBAL_VAR".to_string(),
            EnvValue::String("global_value".to_string()),
        );
        global.insert("GLOBAL_INT".to_string(), EnvValue::Integer(42));

        builder.merge_global(&global);
        let vars = builder.build();

        assert_eq!(vars.get("GLOBAL_VAR"), Some(&"global_value".to_string()));
        assert_eq!(vars.get("GLOBAL_INT"), Some(&"42".to_string()));
    }

    #[test]
    fn test_env_builder_with_config_minimal() {
        let config = Config::default();

        let vars = EnvBuilder::new().with_config(&config, "testgame");
        assert!(!vars.is_empty());
        assert_eq!(
            vars.get("__NV_PRIME_RENDER_OFFLOAD"),
            Some(&"1".to_string())
        );
        assert_eq!(vars.get(NTSYNC).map(String::as_str), Some("1"));
        assert_eq!(vars.get(WAYLAND).map(String::as_str), Some("1"));
    }

    #[test]
    fn test_env_builder_with_config_gpu_name() {
        let mut config = Config::default();
        config.gpu.gpu_name = Some("Test GPU".to_string());

        let vars = EnvBuilder::new().with_config(&config, "testgame");
        assert_eq!(vars.get(DXVK_GPU), Some(&"Test GPU".to_string()));
        assert_eq!(vars.get(VKD3D_GPU), Some(&"Test GPU".to_string()));
    }

    #[test]
    fn test_env_builder_keeps_proton_defaults_with_mangohud() {
        let config: Config = toml::from_str("[game.testgame]\nmangohud = true").unwrap();

        let vars = EnvBuilder::new().with_config(&config, "testgame");
        assert_eq!(vars.get(HUD).map(String::as_str), Some("1"));
        assert_eq!(vars.get(NTSYNC).map(String::as_str), Some("1"));
        assert_eq!(vars.get(WAYLAND).map(String::as_str), Some("1"));
        assert!(!vars.contains_key(NO_NTSYNC));
        assert!(!vars.contains_key(WAYLAND_ALIAS));
    }

    #[test]
    fn test_env_builder_with_config_game_specific() {
        let config: Config = toml::from_str(
            r#"
[game.testgame]
mangohud = true
mangohud_conf = "fps_only=1"
proton_log = true
proton_ntsync = false
proton_wayland = false
wine_dll_overrides = "dinput8=n,b"
"#,
        )
        .unwrap();

        let vars = EnvBuilder::new().with_config(&config, "testgame");
        assert_eq!(vars.get(HUD), Some(&"1".to_string()));
        assert_eq!(vars.get(HUD_CFG), Some(&"fps_only=1".to_string()));
        assert_eq!(vars.get(LOG), Some(&"1".to_string()));
        assert_eq!(vars.get(NTSYNC), Some(&"0".to_string()));
        assert_eq!(vars.get(NO_NTSYNC), Some(&"1".to_string()));
        assert_eq!(vars.get(WAYLAND), Some(&"0".to_string()));
        assert_eq!(vars.get(WAYLAND_ALIAS), Some(&"0".to_string()));
        assert_eq!(vars.get(WINE_DLLS), Some(&"dinput8=n,b".to_string()));
    }

    #[test]
    fn test_logging_defaults_preserve_proton_log_inheritance() {
        let vars = EnvBuilder::new().with_config(&Config::default(), "testgame");
        assert!(!vars.contains_key(LOG));
        assert_eq!(vars["DXVK_LOG_LEVEL"], "info");
        assert_eq!(vars["DXVK_NVAPI_LOG_LEVEL"], "info");
        assert_eq!(vars["DXVK_NVAPI_VKREFLEX_LAYER_LOG_LEVEL"], "info");
        assert_eq!(vars["VKD3D_DEBUG"], "info");
        assert_eq!(vars["VKD3D_SHADER_DEBUG"], "info");
        assert_eq!(vars["WINEDEBUG"], "-all,err+all");
    }

    #[test]
    fn test_launch_config_scope_and_override_order() {
        let config: Config = toml::from_str(
            r#"
[gpu]
gpu_vlk_icd = "/custom/driver.json"

[launch.proton]
log = false
wayland = true

[launch.experimental.dlss]
super_resolution_preset = "RENDER_PRESET_K"
streamline_override = true

[launch.env]
PROTON_LOG = true

[game.testgame]
mangohud = true

[game.testgame.launch.proton]
log = false
wayland = false

[game.testgame.launch.mangohud]
enabled = false

[game.testgame.launch.experimental.nvidia]
max_frames_allowed = 2
yield_mode = "usleep"

[testgame]
MANGOHUD = true
PROTON_LOG_DIR = "/tmp"

[game.testgame.launch.env]
MANGOHUD = false
VKD3D_SWAPCHAIN_PRESENT_MODE = "FIFO_LATEST_READY"
WINEDEBUG = "-all,+seh"
"#,
        )
        .unwrap();

        let global = EnvBuilder::new().with_config(&config, "othergame");
        assert_eq!(global[LOG], "1");
        assert_eq!(global[WAYLAND], "1");
        assert!(!global.contains_key(HUD));
        assert_eq!(global["VK_DRIVER_FILES"], "/custom/driver.json");

        let game = EnvBuilder::new().with_config(&config, "testgame");
        assert_eq!(game[LOG], "0");
        assert_eq!(game[WAYLAND], "0");
        assert_eq!(game[WAYLAND_ALIAS], "0");
        assert_eq!(game[HUD], "0");
        assert_eq!(game["PROTON_LOG_DIR"], "/tmp");
        assert_eq!(game["DXVK_NVAPI_DRS_NGX_DLSS_SR_OVERRIDE"], "on");
        assert_eq!(
            game["DXVK_NVAPI_DRS_NGX_DLSS_SR_OVERRIDE_RENDER_PRESET_SELECTION"],
            "RENDER_PRESET_K"
        );
        assert_eq!(game["DXVK_NVAPI_DRS_SL_DLSS_OVERRIDE"], "on");
        assert!(!game.contains_key("DXVK_NVAPI_DRS_NGX_DLSS_RR_OVERRIDE"));
        assert_eq!(game["__GL_MaxFramesAllowed"], "2");
        assert_eq!(game["__GL_YIELD"], "USLEEP");
        assert_eq!(game["VKD3D_SWAPCHAIN_PRESENT_MODE"], "FIFO_LATEST_READY");
        assert_eq!(game["WINEDEBUG"], "-all,+seh");
    }

    #[test]
    fn test_launch_executable_keeps_legacy_environment_group() {
        let config: Config = toml::from_str("[launch]\nMANGOHUD = true").unwrap();
        assert_eq!(EnvBuilder::new().with_config(&config, "launch")[HUD], "1");
        assert!(
            !EnvBuilder::new()
                .with_config(&config, "othergame")
                .contains_key(HUD)
        );
    }
}
