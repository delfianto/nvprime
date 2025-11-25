use log::{debug, error, info};
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

const CONFIG_FILE: &str = "nvprime.conf";

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub tuning: TuningConfig,

    #[serde(default)]
    pub hooks: HooksConfig,

    #[serde(rename = "env")]
    pub environments: EnvironmentConfig,
}

#[derive(Deserialize, Debug, Default)]
pub struct TuningConfig {
    #[serde(default)]
    pub process: ProcConfig,

    #[serde(default)]
    pub nvidia: NvGpuConfig,

    #[serde(default)]
    pub ryzen: AmdZenConfig,
}

#[derive(Clone, Deserialize, Debug, Default)]
pub struct ProcConfig {
    pub enabled: Option<bool>,
    pub proc_ioprio: Option<u32>,
    pub proc_renice: Option<u32>,
    pub splitlock_hack: Option<bool>,
}

#[derive(Clone, Deserialize, Debug, Default)]
pub struct NvGpuConfig {
    pub enabled: Option<bool>,
    pub set_max: Option<bool>,
    pub power_limit: Option<u32>,
    pub device_uuid: Option<String>,
}

#[derive(Clone, Deserialize, Debug, Default)]
pub struct AmdZenConfig {
    pub enabled: Option<bool>,
    pub amd_epp: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct HooksConfig {
    pub init: Option<String>,
    pub shutdown: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct EnvironmentConfig {
    pub global: HashMap<String, EnvValue>,

    #[serde(flatten)]
    pub executables: HashMap<String, HashMap<String, EnvValue>>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum EnvValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

impl EnvValue {
    pub fn to_string(&self) -> String {
        match self {
            EnvValue::String(s) => s.clone(),
            EnvValue::Integer(i) => i.to_string(),
            EnvValue::Float(f) => f.to_string(),
            EnvValue::Boolean(b) => {
                if *b {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            }
        }
    }
}

impl Config {
    pub fn is_tuning_enabled(&self) -> bool {
        self.tuning.process.enabled.unwrap_or(false)
            || self.tuning.nvidia.enabled.unwrap_or(false)
            || self.tuning.ryzen.enabled.unwrap_or(false)
    }

    pub fn load() -> anyhow::Result<Self> {
        debug!("Locating configuration directory");
        let config_path = dirs::config_dir()
            .ok_or_else(|| {
                error!("Could not find system config directory");
                anyhow::anyhow!("Could not find config directory")
            })?
            .join(CONFIG_FILE);

        Self::load_file(config_path)
    }

    pub fn load_file(config_path: PathBuf) -> anyhow::Result<Self> {
        info!("Loading configuration from: {}", config_path.display());

        let config_str = std::fs::read_to_string(&config_path).map_err(|e| {
            error!(
                "Failed to read config file '{}': {}",
                config_path.display(),
                e
            );
            e
        })?;

        debug!("Configuration file size: {} bytes", config_str.len());

        let config: Config = toml::from_str(&config_str).map_err(|e| {
            error!("Failed to parse TOML configuration: {}", e);
            e
        })?;

        debug!("Configuration parsed successfully");
        debug!("  Global env vars: {}", config.environments.global.len());
        debug!(
            "  Executable configs: {}",
            config.environments.executables.len()
        );
        if let Some(ref init_hook) = config.hooks.init {
            debug!("  Init hook: {}", init_hook);
        }
        if let Some(ref shutdown_hook) = config.hooks.shutdown {
            debug!("  Shutdown hook: {}", shutdown_hook);
        }

        Ok(config)
    }
}
