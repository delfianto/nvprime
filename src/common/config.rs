#![allow(dead_code)]
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub common: CommonConfig,

    #[serde(default)]
    pub hooks: HooksConfig,

    #[serde(rename = "env")]
    pub environments: EnvironmentConfig,
}

#[derive(Deserialize, Debug, Default)]
pub struct CommonConfig {
    pub amd_epp: Option<String>,
    pub gpu_id: Option<String>,
    pub priority: Option<i32>,
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
    pub fn load() -> anyhow::Result<Self> {
        let config_path = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("prime-rs.conf");

        let config_str = std::fs::read_to_string(config_path)?;
        Ok(toml::from_str(&config_str)?)
    }
}
