use crate::common::config::{CpuTune, GpuTune, SysTune};
use crate::service::daemon::{DaemonState, start_pid_watchdog};
use log::{error, info};
use std::sync::{Arc, Mutex};
use zbus::{interface, proxy};

pub struct NvPrimeService {
    pub state: Arc<Mutex<DaemonState>>,
}

impl NvPrimeService {
    pub fn new(state: Arc<Mutex<DaemonState>>) -> Self {
        Self { state }
    }
}

#[interface(name = "com.github.nvprime.Service")]
impl NvPrimeService {
    async fn apply_tuning(&mut self, pid: u32, config_json: String) -> zbus::fdo::Result<()> {
        info!("Received tuning request for PID {}", pid);

        let config: TuningConfig = serde_json::from_str(&config_json)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Invalid config JSON: {}", e)))?;

        {
            let mut state = self.state.lock().unwrap();

            if let Err(e) = state.apply_cpu_tuning(&config.cpu) {
                error!("Failed to apply CPU tuning: {}", e);
                // We don't return error here, just log it, as CPU tuning is optional/best-effort
            }

            if let Err(e) = state.apply_gpu_tuning(&config.gpu) {
                error!("Failed to apply GPU tuning: {}", e);
                return Err(zbus::fdo::Error::Failed(format!(
                    "GPU tuning failed: {}",
                    e
                )));
            }

            if let Err(e) = state.apply_process_priority(pid, &config.sys) {
                error!("Failed to apply process priority: {}", e);
                return Err(zbus::fdo::Error::Failed(format!(
                    "Process priority failed: {}",
                    e
                )));
            }

            state.add_active_pid(pid);
        }

        start_pid_watchdog(
            Arc::clone(&self.state),
            pid,
            config.sys.watchdog_interval_sec,
        )
        .await;

        info!("Applied tuning for PID {}", pid);
        Ok(())
    }

    async fn reset_tuning(&mut self) -> zbus::fdo::Result<()> {
        info!("Resetting tuning");
        let mut state = self.state.lock().unwrap();

        let mut success = true;

        if let Err(e) = state.restore_gpu_defaults() {
            error!("Failed to restore GPU defaults: {}", e);
            success = false;
        }

        if let Err(e) = state.restore_cpu_defaults() {
            error!("Failed to restore CPU defaults: {}", e);
            success = false;
        }

        state.active_pids.clear();
        info!("Tuning reset complete");

        if !success {
            return Err(zbus::fdo::Error::Failed(
                "Failed to fully reset tuning".to_string(),
            ));
        }

        Ok(())
    }

    async fn ping(&self) -> String {
        "pong".to_string()
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct TuningConfig {
    pub cpu: CpuTune,
    pub gpu: GpuTune,
    pub sys: SysTune,
}

#[proxy(
    interface = "com.github.nvprime.Service",
    default_service = "com.github.nvprime",
    default_path = "/com/github/nvprime"
)]
pub trait NvPrimeClient {
    async fn apply_tuning(&self, pid: u32, config_json: String) -> zbus::Result<()>;
    async fn reset_tuning(&self) -> zbus::Result<()>;
    async fn ping(&self) -> zbus::Result<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tuning_config_serialization() {
        let cpu = CpuTune {
            enabled: true,
            amd_epp_tune: "performance".to_string(),
            amd_epp_base: "balance".to_string(),
        };

        let gpu = GpuTune {
            enabled: true,
            gpu_name: Some("Test GPU".to_string()),
            gpu_uuid: Some("GPU-123".to_string()),
            gpu_vlk_icd: "/test.json".to_string(),
            set_max_pwr: true,
            pwr_limit_tune: Some(350000),
        };

        let sys = SysTune {
            enabled: true,
            proc_ioprio: 2,
            proc_renice: -5,
            splitlock_hack: true,
            watchdog_interval_sec: 10,
        };

        let config_json = serde_json::json!({
            "cpu": cpu,
            "gpu": gpu,
            "sys": sys,
        });

        let json_str = serde_json::to_string(&config_json).unwrap();
        assert!(!json_str.is_empty());

        let parsed: TuningConfig = serde_json::from_str(&json_str).unwrap();
        assert!(parsed.cpu.enabled);
        assert_eq!(parsed.cpu.amd_epp_tune, "performance");
        assert!(parsed.gpu.enabled);
        assert_eq!(parsed.gpu.gpu_name, Some("Test GPU".to_string()));
        assert!(parsed.sys.enabled);
        assert_eq!(parsed.sys.proc_renice, -5);
    }

    #[test]
    fn test_tuning_config_deserialization_minimal() {
        let json_str = r#"{"cpu": {"cpu_tuning": false}, "gpu": {"gpu_tuning": false}, "sys": {"sys_tuning": false}}"#;
        let parsed: TuningConfig = serde_json::from_str(json_str).unwrap();

        assert!(!parsed.cpu.enabled);
        assert!(!parsed.gpu.enabled);
        assert!(!parsed.sys.enabled);
    }

    #[test]
    fn test_nvprime_service_new() {
        let state = Arc::new(Mutex::new(DaemonState::new()));
        let service = NvPrimeService::new(Arc::clone(&state));

        let state_lock = service.state.lock().unwrap();
        assert!(state_lock.gpu.is_none());
        assert!(state_lock.active_pids.is_empty());
    }

    #[test]
    fn test_tuning_config_round_trip() {
        let original = TuningConfig {
            cpu: CpuTune::default(),
            gpu: GpuTune {
                enabled: true,
                gpu_name: Some("RTX 4090".to_string()),
                gpu_uuid: None,
                gpu_vlk_icd: "/nvidia.json".to_string(),
                set_max_pwr: false,
                pwr_limit_tune: Some(400000),
            },
            sys: SysTune {
                enabled: true,
                proc_ioprio: 1,
                proc_renice: -10,
                splitlock_hack: false,
                watchdog_interval_sec: 15,
            },
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: TuningConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.gpu.enabled, original.gpu.enabled);
        assert_eq!(deserialized.gpu.gpu_name, original.gpu.gpu_name);
        assert_eq!(deserialized.gpu.pwr_limit_tune, original.gpu.pwr_limit_tune);
        assert_eq!(deserialized.sys.proc_renice, original.sys.proc_renice);
    }
}
