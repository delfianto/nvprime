#![allow(dead_code)]

use crate::common::config::EnvValue;
use log::debug;
use phf::{Map, phf_map};
use std::collections::HashMap;
use std::env;

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
    vars: HashMap<String, String>,
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

    pub fn with_env(mut self, key: &str, val: &str) -> Self {
        self.vars.insert(key.into(), val.into());
        self
    }

    pub fn with_bool(self, key: &str, enabled: bool) -> Self {
        self.with_env(key.into(), if enabled { "1" } else { "0" })
    }

    pub fn with_gpu_name(self, device: &str) -> Self {
        self.with_env("DXVK_FILTER_DEVICE_NAME", device)
            .with_env("VKD3D_FILTER_DEVICE_NAME", device)
    }

    pub fn with_mangohud(self, enabled: bool) -> Self {
        self.with_bool("MANGOHUD", enabled)
    }

    pub fn with_proton_log(self, enabled: bool) -> Self {
        self.with_bool("PROTON_LOG", enabled)
    }

    pub fn with_proton_ntsync(self, enabled: bool) -> Self {
        self.with_bool("PROTON_USE_NTSYNC", enabled)
    }

    pub fn with_proton_wayland(self, enabled: bool) -> Self {
        self.with_bool("PROTON_ENABLE_WAYLAND", enabled)
    }

    pub fn with_wine_dll_overrides(self, value: &str) -> Self {
        self.with_env("WINEDLLOVERRIDES", value)
    }

    /// Build the final environment map
    pub fn build(self) -> HashMap<String, String> {
        debug!(
            "Building final environment map with {} variables",
            self.vars.len()
        );
        self.vars
    }

    /// Merge global environment variables from config
    pub fn merge_global(&mut self, global: &HashMap<String, EnvValue>) {
        debug!("Merging {} global environment variables", global.len());
        for (key, value) in global {
            let value_str = value.to_string();
            debug!("  Adding global: {} = {}", key, value_str);
            self.vars.insert(key.clone(), value_str);
        }
    }

    /// Merge executable-specific environment variables
    pub fn merge_executable(&mut self, exe_vars: Option<&HashMap<String, EnvValue>>) {
        if let Some(vars) = exe_vars {
            debug!(
                "Merging {} executable-specific environment variables",
                vars.len()
            );
            for (key, value) in vars {
                let value_str = value.to_string();
                debug!("  Adding executable-specific: {} = {}", key, value_str);
                self.vars.insert(key.clone(), value_str);
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

pub fn from_hashmap(env_vars: &HashMap<String, String>) {
    debug!("Setting environment variables in parent context:");

    for (key, val) in env_vars {
        unsafe {
            env::set_var(key, val);
            debug!("  Setting Vars: {} = {}", key, val);
        }
    }
}
