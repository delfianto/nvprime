use crate::common::Config;
use crate::common::config::EnvValue;
use log::debug;
use phf::{Map, phf_map};
use std::collections::BTreeMap;

const LOG: &str = "PROTON_LOG";
const HUD: &str = "MANGOHUD";
const HUD_CFG: &str = "MANGOHUD_CONFIG";
const NTSYNC: &str = "PROTON_USE_NTSYNC";
const WAYLAND: &str = "PROTON_ENABLE_WAYLAND";
const DXVK_GPU: &str = "DXVK_FILTER_DEVICE_NAME";
const VKD3D_GPU: &str = "VKD3D_FILTER_DEVICE_NAME";
const WINE_DLLS: &str = "WINEDLLOVERRIDES";

/// Default values for environment variables
static ENV_DEFAULTS: Map<&'static str, &'static str> = phf_map! {
    // MangoHud settings
    "MANGOHUD" => "0",
    "MANGOHUD_CONFIG" => "preset=1",

    // Proton logging flags
    "PROTON_LOG" => "0",
    "DXVK_LOG_LEVEL" => "info",
    "DXVK_NVAPI_LOG_LEVEL" => "info",
    "DXVK_NVAPI_VKREFLEX_LAYER_LOG_LEVEL" => "info",
    "VKD3D_DEBUG" => "info",
    "VKD3D_SHADER_DEBUG" => "info",
    "WINEDEBUG" => "+err,+warn,-all",

    // Proton tuneables
    "PROTON_USE_NTSYNC" => "0",
    "PROTON_ENABLE_WAYLAND" => "0",
    "PROTON_SET_GAME_DRIVE" => "1",

    // NVIDIA DLSS settings from dxvk-nvapi
    "DXVK_NVAPI_SET_NGX_DEBUG_OPTIONS" => "DLSSIndicator=0,DLSSGIndicator=0",

    "DXVK_NVAPI_DRS_NGX_DLSS_RR_OVERRIDE" => "on",
    "DXVK_NVAPI_DRS_NGX_DLSS_RR_OVERRIDE_RENDER_PRESET_SELECTION" => "render_preset_latest",

    "DXVK_NVAPI_DRS_NGX_DLSS_SR_OVERRIDE" => "on",
    "DXVK_NVAPI_DRS_NGX_DLSS_SR_OVERRIDE_RENDER_PRESET_SELECTION" => "render_preset_latest",

    // The actual flag for PRIME rendering offload
    "__NV_PRIME_RENDER_OFFLOAD" => "1",
    "__GLX_VENDOR_LIBRARY_NAME" => "nvidia",
    "__VK_LAYER_NV_optimus" => "NVIDIA_only",
    "VK_ICD_FILENAMES" => "/usr/share/vulkan/icd.d/nvidia_icd.json",

    // Tells the driver to prioritize performance over power saving,
    // suppose to help the case where the GPU is not boosting under
    // some game menu thus making the UI laggy
    "__GL_ExperimentalPerfStrategy" => "1",

    // Prevents the desktop compositor (GNOME/KDE) from double-syncing frames
    "__GL_GSYNC_ALLOWED" => "1",
    "__GL_MaxFramesAllowed" => "1",
    "__GL_VRR_ALLOWED" => "1",
    "__GL_YIELD" => "USLEEP",
};

pub struct EnvBuilder {
    vars: BTreeMap<String, String>,
}

impl EnvBuilder {
    pub fn new() -> Self {
        debug!("Creating new environment builder");
        Self {
            vars: ENV_DEFAULTS
                .entries()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    fn set_str(&mut self, key: &str, val: &str) {
        self.vars.insert(key.to_string(), val.to_string());
    }

    fn set_bool(&mut self, key: &str, enabled: bool) {
        self.set_str(key, if enabled { "1" } else { "0" })
    }

    pub fn with_config(mut self, config: &Config, exe_name: &String) -> BTreeMap<String, String> {
        debug!("Initializing environment values for game: {}", exe_name);

        // `config.gpu.gpu_name` is an `Option<String>` and since `String`
        // does not implement `Copy` we need to explicitly use reference
        // when performing pattern matching.
        if let Some(device) = &config.gpu.gpu_name {
            let slice = device.as_str();
            self.set_str(DXVK_GPU, slice);
            self.set_str(VKD3D_GPU, slice);
        }

        // `config.game` is a `HashMap`, the `get` function will return
        // `Option<&T> which already a reference itself, thus we do not
        // need to access config through its reference.
        if let Some(game) = config.game.get(exe_name) {
            self.set_bool(HUD, game.mangohud);
            self.set_bool(LOG, game.proton_log);
            self.set_bool(NTSYNC, game.proton_ntsync);
            self.set_bool(WAYLAND, game.proton_wayland);

            if let Some(hud_cfg) = &game.mangohud_conf {
                self.set_str(HUD_CFG, hud_cfg);
            }

            if let Some(dll_overrides) = &game.wine_dll_overrides {
                self.set_str(WINE_DLLS, dll_overrides);
            }
        }

        if let Some(env) = config.env.get(exe_name) {
            for (key, val) in env {
                self.vars.insert(key.to_string(), val.to_string());
            }
        }

        self.build()
    }

    pub fn with_env(mut self, key: &str, val: &str) -> Self {
        self.set_str(key, val);
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

    pub fn with_ntsync(self, enabled: bool) -> Self {
        self.with_bool(NTSYNC, enabled)
    }

    pub fn with_wayland(self, enabled: bool) -> Self {
        self.with_bool(WAYLAND, enabled)
    }

    pub fn with_dll_overrides(self, value: &str) -> Self {
        self.with_env(WINE_DLLS, value)
    }

    /// Build the final environment map
    pub fn build(self) -> BTreeMap<String, String> {
        debug!(
            "Building final environment map with {} variables",
            self.vars.len()
        );
        self.vars
    }

    /// Merge global environment variables from config
    pub fn merge_global(&mut self, global: &BTreeMap<String, EnvValue>) {
        debug!("Merging {} global environment variables", global.len());
        for (key, value) in global {
            let value_str = value.to_string();
            debug!("  Adding global: {} = {}", key, value_str);
            self.vars.insert(key.clone(), value_str);
        }
    }

    /// Merge executable-specific environment variables
    pub fn merge_executable(&mut self, exe_vars: Option<&BTreeMap<String, EnvValue>>) {
        if let Some(vars) = exe_vars {
            debug!(
                "Merging {} executable-specific environment variables",
                vars.len()
            );
            for (key, val) in vars {
                let str = val.to_string();
                debug!("  Adding executable-specific: {} = {}", key, str);
                self.vars.insert(key.clone(), str);
            }
        } else {
            debug!("No executable-specific environment variables to merge");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::config::{Config, GameConfig, GpuTune};

    #[test]
    fn test_env_builder_new() {
        let builder = EnvBuilder::new();
        assert!(!builder.vars.is_empty());
        assert_eq!(builder.vars.get("__NV_PRIME_RENDER_OFFLOAD"), Some(&"1".to_string()));
        assert_eq!(builder.vars.get("__GLX_VENDOR_LIBRARY_NAME"), Some(&"nvidia".to_string()));
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
    }

    #[test]
    fn test_env_builder_with_wayland() {
        let builder = EnvBuilder::new().with_wayland(true);
        let vars = builder.build();
        assert_eq!(vars.get(WAYLAND), Some(&"1".to_string()));
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
        global.insert("GLOBAL_VAR".to_string(), EnvValue::String("global_value".to_string()));
        global.insert("GLOBAL_INT".to_string(), EnvValue::Integer(42));

        builder.merge_global(&global);
        let vars = builder.build();

        assert_eq!(vars.get("GLOBAL_VAR"), Some(&"global_value".to_string()));
        assert_eq!(vars.get("GLOBAL_INT"), Some(&"42".to_string()));
    }

    #[test]
    fn test_env_builder_with_config_minimal() {
        let config = Config {
            cpu: Default::default(),
            gpu: GpuTune::default(),
            sys: Default::default(),
            env: Default::default(),
            game: Default::default(),
            hook: Default::default(),
        };

        let vars = EnvBuilder::new().with_config(&config, &"testgame".to_string());
        assert!(!vars.is_empty());
        assert_eq!(vars.get("__NV_PRIME_RENDER_OFFLOAD"), Some(&"1".to_string()));
    }

    #[test]
    fn test_env_builder_with_config_gpu_name() {
        let mut config = Config {
            cpu: Default::default(),
            gpu: GpuTune::default(),
            sys: Default::default(),
            env: Default::default(),
            game: Default::default(),
            hook: Default::default(),
        };
        config.gpu.gpu_name = Some("Test GPU".to_string());

        let vars = EnvBuilder::new().with_config(&config, &"testgame".to_string());
        assert_eq!(vars.get(DXVK_GPU), Some(&"Test GPU".to_string()));
        assert_eq!(vars.get(VKD3D_GPU), Some(&"Test GPU".to_string()));
    }

    #[test]
    fn test_env_builder_with_config_game_specific() {
        let mut config = Config {
            cpu: Default::default(),
            gpu: GpuTune::default(),
            sys: Default::default(),
            env: Default::default(),
            game: Default::default(),
            hook: Default::default(),
        };

        let game_config = GameConfig {
            mangohud: true,
            mangohud_conf: Some("fps_only=1".to_string()),
            proton_log: true,
            proton_ntsync: true,
            proton_wayland: false,
            wine_dll_overrides: Some("dinput8=n,b".to_string()),
        };
        config.game.insert("testgame".to_string(), game_config);

        let vars = EnvBuilder::new().with_config(&config, &"testgame".to_string());
        assert_eq!(vars.get(HUD), Some(&"1".to_string()));
        assert_eq!(vars.get(HUD_CFG), Some(&"fps_only=1".to_string()));
        assert_eq!(vars.get(LOG), Some(&"1".to_string()));
        assert_eq!(vars.get(NTSYNC), Some(&"1".to_string()));
        assert_eq!(vars.get(WAYLAND), Some(&"0".to_string()));
        assert_eq!(vars.get(WINE_DLLS), Some(&"dinput8=n,b".to_string()));
    }

    #[test]
    fn test_env_defaults_contains_required_vars() {
        let builder = EnvBuilder::new();
        assert!(builder.vars.contains_key("__NV_PRIME_RENDER_OFFLOAD"));
        assert!(builder.vars.contains_key("__GLX_VENDOR_LIBRARY_NAME"));
        assert!(builder.vars.contains_key("VK_ICD_FILENAMES"));
        assert!(builder.vars.contains_key("MANGOHUD"));
        assert!(builder.vars.contains_key("PROTON_LOG"));
    }
}
