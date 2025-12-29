use crate::common::{
    config::{CpuTune, GpuTune, SysTune},
    nvgpu::NvGpu,
};
use crate::service::ryzen::RyzenEPPManager;
use anyhow::{Context, Result};
use log::{debug, error, info};
use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct DaemonState {
    pub gpu: Option<NvGpu>,
    pub active_pids: HashSet<u32>,
    pub baseline_power_limit: Option<u32>,
    pub baseline_epp: Option<String>,
}

impl DaemonState {
    pub fn new() -> Self {
        Self {
            gpu: None,
            active_pids: HashSet::new(),
            baseline_power_limit: None,
            baseline_epp: None,
        }
    }
}

impl Default for DaemonState {
    fn default() -> Self {
        Self::new()
    }
}

impl DaemonState {
    pub fn init_gpu(&mut self, gpu_uuid: Option<String>) -> Result<()> {
        info!("Initializing GPU");
        let mut gpu = NvGpu::init(gpu_uuid).context("Failed to initialize NVML")?;

        gpu.log_gpu_info().context("Failed to get GPU info")?;

        let device = gpu.get_device().context("Failed to get GPU device")?;
        self.baseline_power_limit = Some(
            device
                .power_management_limit_default()
                .context("Failed to get default power limit")?,
        );

        self.gpu = Some(gpu);
        Ok(())
    }

    pub fn apply_cpu_tuning(&mut self, cpu_config: &CpuTune) -> Result<()> {
        if !cpu_config.enabled {
            debug!("CPU tuning disabled, skipping");
            return Ok(());
        }

        // Save the baseline EPP if not already saved (from config)
        if self.baseline_epp.is_none() {
            self.baseline_epp = Some(cpu_config.amd_epp_base.clone());
        }

        RyzenEPPManager::set_epp(&cpu_config.amd_epp_tune)?;
        info!("Applied CPU tuning: {}", cpu_config.amd_epp_tune);
        Ok(())
    }

    pub fn apply_gpu_tuning(&mut self, gpu_config: &GpuTune) -> Result<()> {
        if !gpu_config.enabled {
            debug!("GPU tuning disabled, skipping");
            return Ok(());
        }

        let gpu = self.gpu.as_mut().context("GPU not initialized")?;

        gpu.set_power_limit(gpu_config.pwr_limit_tune, Some(gpu_config.set_max_pwr))
            .context("Failed to set power limit")?;

        info!("Applied GPU tuning");
        Ok(())
    }

    pub fn apply_process_priority(&self, pid: u32, sys_config: &SysTune) -> Result<()> {
        if !sys_config.enabled {
            debug!("System tuning disabled, skipping");
            return Ok(());
        }

        if sys_config.proc_renice != 0 {
            unsafe {
                let result = libc::setpriority(libc::PRIO_PROCESS, pid, sys_config.proc_renice);

                if result != 0 {
                    anyhow::bail!("setpriority failed with code {}", result);
                }
            }

            info!("Set process {} priority to {}", pid, sys_config.proc_renice);
        }

        Ok(())
    }

    pub fn restore_gpu_defaults(&mut self) -> Result<()> {
        if let Some(gpu) = self.gpu.as_mut() {
            gpu.restore_defaults()
                .context("Failed to restore GPU defaults")?;
            info!("Restored GPU to default settings");
        }
        Ok(())
    }

    pub fn restore_cpu_defaults(&mut self) -> Result<()> {
        if let Some(base_epp) = &self.baseline_epp {
            RyzenEPPManager::set_epp(base_epp)?;
            info!("Restored CPU EPP to default: {}", base_epp);
        }
        Ok(())
    }

    pub fn add_active_pid(&mut self, pid: u32) {
        self.active_pids.insert(pid);
    }

    pub fn remove_active_pid(&mut self, pid: u32) {
        self.active_pids.remove(&pid);
    }

    pub fn is_pid_alive(pid: u32) -> bool {
        Path::new(&format!("/proc/{}", pid)).exists()
    }
}

pub async fn start_pid_watchdog(state: Arc<Mutex<DaemonState>>, pid: u32, interval_sec: u64) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(interval_sec)).await;

            if !DaemonState::is_pid_alive(pid) {
                info!("Process {} terminated, cleaning up", pid);

                let mut state = state.lock().unwrap();
                state.remove_active_pid(pid);

                if state.active_pids.is_empty() {
                    if let Err(e) = state.restore_gpu_defaults() {
                        error!("Failed to restore GPU defaults: {}", e);
                    }
                    if let Err(e) = state.restore_cpu_defaults() {
                        error!("Failed to restore CPU defaults: {}", e);
                    }
                }
                break;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_state_new() {
        let state = DaemonState::new();
        assert!(state.gpu.is_none());
        assert!(state.active_pids.is_empty());
        assert!(state.baseline_power_limit.is_none());
        assert!(state.baseline_epp.is_none());
    }

    #[test]
    fn test_daemon_state_add_remove_pid() {
        let mut state = DaemonState::new();

        state.add_active_pid(1234);
        assert!(state.active_pids.contains(&1234));
        assert_eq!(state.active_pids.len(), 1);

        state.add_active_pid(5678);
        assert_eq!(state.active_pids.len(), 2);

        state.remove_active_pid(1234);
        assert!(!state.active_pids.contains(&1234));
        assert_eq!(state.active_pids.len(), 1);
    }

    #[test]
    fn test_daemon_state_duplicate_pid() {
        let mut state = DaemonState::new();

        state.add_active_pid(1234);
        state.add_active_pid(1234);
        assert_eq!(state.active_pids.len(), 1);
    }

    #[test]
    fn test_is_pid_alive_current_process() {
        let current_pid = std::process::id();
        assert!(DaemonState::is_pid_alive(current_pid));
    }

    #[test]
    fn test_is_pid_alive_nonexistent() {
        assert!(!DaemonState::is_pid_alive(999999));
    }

    #[test]
    fn test_apply_gpu_tuning_disabled() {
        let mut state = DaemonState::new();
        let gpu_config = GpuTune {
            enabled: false,
            gpu_name: None,
            gpu_uuid: None,
            gpu_vlk_icd: String::new(),
            set_max_pwr: false,
            pwr_limit_tune: None,
        };

        let result = state.apply_gpu_tuning(&gpu_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_gpu_tuning_no_gpu_initialized() {
        let mut state = DaemonState::new();
        let gpu_config = GpuTune {
            enabled: true,
            gpu_name: None,
            gpu_uuid: None,
            gpu_vlk_icd: String::new(),
            set_max_pwr: true,
            pwr_limit_tune: Some(300000),
        };

        let result = state.apply_gpu_tuning(&gpu_config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("GPU not initialized")
        );
    }

    #[test]
    fn test_apply_process_priority_disabled() {
        let state = DaemonState::new();
        let sys_config = SysTune {
            enabled: false,
            proc_ioprio: 4,
            proc_renice: 0,
            splitlock_hack: false,
            watchdog_interval_sec: 10,
        };

        let result = state.apply_process_priority(std::process::id(), &sys_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_process_priority_zero_renice() {
        let state = DaemonState::new();
        let sys_config = SysTune {
            enabled: true,
            proc_ioprio: 4,
            proc_renice: 0,
            splitlock_hack: false,
            watchdog_interval_sec: 10,
        };

        let result = state.apply_process_priority(std::process::id(), &sys_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_restore_gpu_defaults_no_gpu() {
        let mut state = DaemonState::new();
        let result = state.restore_gpu_defaults();
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_cpu_tuning_disabled() {
        let mut state = DaemonState::new();
        let cpu_config = CpuTune {
            enabled: false,
            amd_epp_tune: "performance".to_string(),
            amd_epp_base: "balance_performance".to_string(),
        };

        let result = state.apply_cpu_tuning(&cpu_config);
        assert!(result.is_ok());
        assert!(state.baseline_epp.is_none());
    }

    #[test]
    fn test_apply_cpu_tuning_enabled() {
        let mut state = DaemonState::new();
        let cpu_config = CpuTune {
            enabled: true,
            amd_epp_tune: "performance".to_string(),
            amd_epp_base: "balance_performance".to_string(),
        };

        // Note: This calls the real RyzenEPPManager, but since we are mocking/ignoring
        // errors in RyzenEPPManager (it returns Ok if sysfs not found), this should pass.
        // However, we can verify baseline_epp is set.
        let result = state.apply_cpu_tuning(&cpu_config);
        assert!(result.is_ok());
        assert_eq!(state.baseline_epp, Some("balance_performance".to_string()));
    }

    #[test]
    fn test_restore_cpu_defaults_none() {
        let mut state = DaemonState::new();
        let result = state.restore_cpu_defaults();
        assert!(result.is_ok());
    }
}
