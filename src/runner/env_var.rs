#![allow(dead_code)]

use crate::common::Config;
use crate::common::config::EnvValue;
use log::debug;
use phf::{Map, phf_map};
use std::collections::BTreeMap;
use std::env;

const HUD: &str = "MANGOHUD";
const LOG: &str = "PROTON_LOG";
const NTSYNC: &str = "PROTON_USE_NTSYNC";
const WAYLAND: &str = "PROTON_ENABLE_WAYLAND";
const DXVK_GPU: &str = "DXVK_FILTER_DEVICE_NAME";
const VKD3D_GPU: &str = "VKD3D_FILTER_DEVICE_NAME";
const WINE_DLLS: &str = "WINEDLLOVERRIDES";

/// Default values for environment variables
static ENV_DEFAULTS: Map<&'static str, &'static str> = phf_map! {
    // Various flags for proton and mangohud
    "MANGOHUD" => "0",
    "PROTON_LOG" => "0",
    "PROTON_USE_NTSYNC" => "0",
    "PROTON_ENABLE_WAYLAND" => "0",

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

const EMPTY_STRING: &str = "EMPTY_STRING";
const NOT_PRESENT: &str = "NOT_PRESENT";

pub fn get_value(key: &str) -> String {
    match env::var(key) {
        Ok(val) => {
            if val.is_empty() {
                EMPTY_STRING.to_string()
            } else {
                val
            }
        }
        Err(_) => NOT_PRESENT.to_string(),
    }
}

pub fn from_strings(env_vars: &(&str, &str)) {
    unsafe {
        env::set_var(env_vars.0, env_vars.1);
        debug!("  Setting Vars: {} = {}", env_vars.0, env_vars.1);
    }
}

pub fn from_slices(env_vars: &[(&str, &str)]) {
    for (key, val) in env_vars {
        unsafe {
            env::set_var(key, val);
            debug!("  Setting Vars: {} = {}", key, val);
        }
    }
}

pub fn from_collection(env_vars: &BTreeMap<String, String>) {
    debug!("Setting environment variables in parent context:");

    for (key, val) in env_vars {
        unsafe {
            env::set_var(key, val);
            debug!("  Setting Vars: {} = {}", key, val);
        }
    }
}
