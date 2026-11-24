use anyhow::Context;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    path::PathBuf,
};

const DEFAULT_CONFIG_PATH: &str = "/etc/nvprime.toml";

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub cpu: CpuTune,

    #[serde(default)]
    pub gpu: GpuTune,

    #[serde(default)]
    pub sys: SysTune,

    pub launch: Option<LaunchTable>,

    #[serde(flatten)]
    pub env: HashMap<String, HashMap<String, EnvValue>>,

    #[serde(default)]
    pub game: HashMap<String, GameConfig>,

    #[serde(default)]
    pub hook: HooksConfig,
}

/// Config section for AMD Zen EPP tuning
#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct CpuTune {
    /// Flag for tuning status
    #[serde(rename = "cpu_tuning")]
    pub enabled: bool,

    /// Power profile when gaming
    pub amd_epp_tune: String,

    /// Default (baseline) power profile
    pub amd_epp_base: String,
}

/// Default state for AMD Zen EPP tuning
impl Default for CpuTune {
    fn default() -> Self {
        Self {
            enabled: false,
            amd_epp_tune: "performance".to_string(),
            amd_epp_base: "balance_performance".to_string(),
        }
    }
}

/// Config section for NVIDIA GPU and any related tuning flag
#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct GpuTune {
    /// Flag to enable power tuning
    #[serde(rename = "gpu_tuning")]
    pub enabled: bool,

    /// Vulkan GPU name, this will be used to set the
    /// DXVK_FILTER_DEVICE_NAME and VKD3D_FILTER_DEVICE_NAME
    pub gpu_name: Option<String>,

    /// NVIDIA GPU uuid, get it from `nvidia-smi -L`
    pub gpu_uuid: Option<String>,

    /// Optional Vulkan driver manifest passed to the Vulkan loader.
    pub gpu_vlk_icd: String,

    /// Set the GPU power limit to highest
    pub set_max_pwr: bool,

    /// Set custom power limit for the GPU
    pub pwr_limit_tune: Option<u32>,
}

/// Default state for NVIDIA GPU tuning
impl Default for GpuTune {
    fn default() -> Self {
        Self {
            enabled: false,
            gpu_name: None,
            gpu_uuid: None,
            gpu_vlk_icd: String::new(),
            set_max_pwr: false,
            pwr_limit_tune: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct SysTune {
    /// Enable or disable system-level tuning
    #[serde(rename = "sys_tuning")]
    pub enabled: bool,

    /// IO priority level for processes (0-7, lower is higher priority)
    /// Uses ionice best-effort class where 0 is highest, 7 is lowest
    /// Default: 4 (middle priority)
    pub proc_ioprio: i32,

    /// Nice value adjustment for process CPU priority (-20 to 19)
    /// Negative values increase priority (root only), positive values decrease it
    /// Default: 0 (no adjustment)
    pub proc_renice: i32,

    /// Enable split-lock detection mitigation hack
    /// Helps prevent performance degradation from split-lock abuse by game engine
    pub splitlock_hack: bool,

    /// Interval in seconds for the daemon to poll process status
    /// Default: 10 seconds
    pub watchdog_interval_sec: u64,
}

impl Default for SysTune {
    fn default() -> Self {
        Self {
            enabled: false,
            proc_ioprio: 4,
            proc_renice: 0,
            splitlock_hack: false,
            watchdog_interval_sec: 10,
        }
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct HooksConfig {
    pub init: Option<String>,
    pub shutdown: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct GameConfig {
    pub mangohud: Option<bool>,
    pub mangohud_conf: Option<String>,
    pub proton_log: Option<bool>,
    pub proton_ntsync: Option<bool>,
    pub proton_wayland: Option<bool>,
    pub wine_dll_overrides: Option<String>,
    pub launch: LaunchConfig,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum LaunchTable {
    Settings(LaunchConfig),
    // Older configs can use [launch] as the environment group for launch.exe.
    ExecutableEnv(BTreeMap<String, EnvValue>),
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct LaunchConfig {
    pub proton: ProtonLaunchConfig,
    pub mangohud: MangoHudLaunchConfig,
    pub experimental: ExperimentalLaunchConfig,
    pub env: BTreeMap<String, EnvValue>,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct ProtonLaunchConfig {
    pub ntsync: Option<bool>,
    pub wayland: Option<bool>,
    pub log: Option<bool>,
    pub log_dir: Option<String>,
    pub game_drive: Option<bool>,
    pub wine_dll_overrides: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct MangoHudLaunchConfig {
    pub enabled: Option<bool>,
    pub config: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct ExperimentalLaunchConfig {
    pub dlss: DlssLaunchConfig,
    pub nvidia: NvidiaLaunchConfig,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct DlssLaunchConfig {
    pub super_resolution_preset: Option<String>,
    pub ray_reconstruction_preset: Option<String>,
    pub streamline_override: Option<bool>,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct NvidiaLaunchConfig {
    pub experimental_perf_strategy: Option<bool>,
    pub gsync_allowed: Option<bool>,
    pub vrr_allowed: Option<bool>,
    pub max_frames_allowed: Option<u32>,
    pub yield_mode: Option<NvidiaYieldMode>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum NvidiaYieldMode {
    Nothing,
    Usleep,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum EnvValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

impl fmt::Display for EnvValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvValue::String(s) => write!(f, "{}", s),
            EnvValue::Integer(i) => write!(f, "{}", i),
            EnvValue::Float(fl) => write!(f, "{}", fl),
            EnvValue::Boolean(b) => write!(f, "{}", if *b { "1" } else { "0" }),
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = std::env::var_os("NVPRIME_CONFIG")
            .map_or_else(|| PathBuf::from(DEFAULT_CONFIG_PATH), PathBuf::from);

        Self::load_file(config_path)
    }

    pub fn load_file(config_path: PathBuf) -> anyhow::Result<Self> {
        info!("Loading configuration from: {}", config_path.display());

        let config_str = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file '{}'", config_path.display()))?;

        debug!("Configuration file size: {} bytes", config_str.len());

        let config: Config = toml::from_str(&config_str).with_context(|| {
            format!("Invalid TOML configuration in '{}'", config_path.display())
        })?;

        debug!("Configuration parsed successfully");
        debug!("  Executable configs: {}", config.env.len());
        if let Some(ref init_hook) = config.hook.init {
            debug!("  Init hook: {}", init_hook);
        }
        if let Some(ref shutdown_hook) = config.hook.shutdown {
            debug!("  Shutdown hook: {}", shutdown_hook);
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_cpu_tune_defaults() {
        let cpu = CpuTune::default();
        assert!(!cpu.enabled);
        assert_eq!(cpu.amd_epp_tune, "performance");
        assert_eq!(cpu.amd_epp_base, "balance_performance");
    }

    #[test]
    fn test_gpu_tune_defaults() {
        let gpu = GpuTune::default();
        assert!(!gpu.enabled);
        assert!(gpu.gpu_name.is_none());
        assert!(gpu.gpu_uuid.is_none());
        assert!(gpu.gpu_vlk_icd.is_empty());
        assert!(!gpu.set_max_pwr);
        assert!(gpu.pwr_limit_tune.is_none());
    }

    #[test]
    fn test_sys_tune_defaults() {
        let sys = SysTune::default();
        assert!(!sys.enabled);
        assert_eq!(sys.proc_ioprio, 4);
        assert_eq!(sys.proc_renice, 0);
        assert!(!sys.splitlock_hack);
    }

    #[test]
    fn test_game_config_defaults() {
        let game = GameConfig::default();
        assert!(game.mangohud.is_none());
        assert!(game.mangohud_conf.is_none());
        assert!(game.proton_log.is_none());
        assert!(game.proton_ntsync.is_none());
        assert!(game.proton_wayland.is_none());
        assert!(game.wine_dll_overrides.is_none());
    }

    #[test]
    fn test_env_value_to_string() {
        assert_eq!(EnvValue::String("test".to_string()).to_string(), "test");
        assert_eq!(EnvValue::Integer(42).to_string(), "42");
        assert_eq!(EnvValue::Float(12.5).to_string(), "12.5");
        assert_eq!(EnvValue::Boolean(true).to_string(), "1");
        assert_eq!(EnvValue::Boolean(false).to_string(), "0");
    }

    #[test]
    fn test_minimal_config_parsing() {
        let toml_content = r#""#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert!(!config.cpu.enabled);
        assert!(!config.gpu.enabled);
        assert!(!config.sys.enabled);
    }

    #[test]
    fn test_packaged_config_parses_with_tuning_disabled() {
        let config: Config = toml::from_str(include_str!("../../system/nvprime.toml")).unwrap();

        assert!(!config.cpu.enabled);
        assert!(!config.gpu.enabled);
        assert!(!config.sys.enabled);
    }

    #[test]
    fn test_full_config_parsing() {
        let toml_content = r#"
[cpu]
cpu_tuning = true
amd_epp_tune = "performance"
amd_epp_base = "balance_performance"

[gpu]
gpu_tuning = true
gpu_name = "NVIDIA GeForce RTX 4090"
gpu_uuid = "GPU-12345678"
gpu_vlk_icd = "/custom/nvidia_icd.json"
set_max_pwr = true
pwr_limit_tune = 450000

[sys]
sys_tuning = true
proc_ioprio = 2
proc_renice = -5
splitlock_hack = true

[hook]
init = "echo 'Starting game'"
shutdown = "echo 'Game ended'"

[game.testgame]
mangohud = true
mangohud_conf = "fps_only=1"
proton_log = true
proton_ntsync = true
proton_wayland = false
wine_dll_overrides = "dinput8=n,b"
        "#;

        let config: Config = toml::from_str(toml_content).unwrap();

        assert!(config.cpu.enabled);
        assert_eq!(config.cpu.amd_epp_tune, "performance");

        assert!(config.gpu.enabled);
        assert_eq!(
            config.gpu.gpu_name,
            Some("NVIDIA GeForce RTX 4090".to_string())
        );
        assert_eq!(config.gpu.gpu_uuid, Some("GPU-12345678".to_string()));
        assert!(config.gpu.set_max_pwr);
        assert_eq!(config.gpu.pwr_limit_tune, Some(450000));

        assert!(config.sys.enabled);
        assert_eq!(config.sys.proc_ioprio, 2);
        assert_eq!(config.sys.proc_renice, -5);
        assert!(config.sys.splitlock_hack);

        assert_eq!(config.hook.init, Some("echo 'Starting game'".to_string()));
        assert_eq!(config.hook.shutdown, Some("echo 'Game ended'".to_string()));

        let game = config.game.get("testgame").unwrap();
        assert_eq!(game.mangohud, Some(true));
        assert_eq!(game.mangohud_conf, Some("fps_only=1".to_string()));
        assert_eq!(game.proton_log, Some(true));
    }

    #[test]
    fn test_config_load_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
[gpu]
gpu_tuning = true
gpu_name = "Test GPU"
            "#
        )
        .unwrap();

        let config = Config::load_file(temp_file.path().to_path_buf()).unwrap();
        assert!(config.gpu.enabled);
        assert_eq!(config.gpu.gpu_name, Some("Test GPU".to_string()));
    }

    #[test]
    fn test_config_load_file_nonexistent() {
        let result = Config::load_file(PathBuf::from("/nonexistent/config.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_config_load_file_invalid_toml() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid toml [[[").unwrap();

        let result = Config::load_file(temp_file.path().to_path_buf());
        assert!(result.is_err());
    }

    #[test]
    fn test_config_serialization() {
        let gpu = GpuTune {
            enabled: true,
            gpu_name: Some("Test".to_string()),
            gpu_uuid: None,
            gpu_vlk_icd: "/test.json".to_string(),
            set_max_pwr: true,
            pwr_limit_tune: Some(400000),
        };

        let json = serde_json::to_string(&gpu).unwrap();
        let deserialized: GpuTune = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.enabled, gpu.enabled);
        assert_eq!(deserialized.gpu_name, gpu.gpu_name);
        assert_eq!(deserialized.set_max_pwr, gpu.set_max_pwr);
    }
}
