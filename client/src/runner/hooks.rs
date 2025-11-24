use crate::common::config::Config;
use crate::system::nvidia::NvTuner;
use anyhow::Result;
use log::{debug, error, info, warn};
use std::process::Command;

pub struct HookRunner;

impl HookRunner {
    pub fn run_init(config: &Config, hook: Option<&str>) -> Result<()> {
        // Apply NVIDIA tuning first (before user hooks)
        if let Err(e) = Self::apply_nvidia_tuning(config) {
            error!("Failed to apply NVIDIA tuning: {}", e);
            // Continue execution even if NVIDIA tuning fails
        }

        // Execute user-defined init hook
        if let Some(cmd) = hook {
            info!("Executing init hook");
            Self::execute_hook(cmd, "init")?;
        } else {
            debug!("No init hook configured");
        }
        Ok(())
    }

    pub fn run_shutdown(config: &Config, hook: Option<&str>) -> Result<()> {
        // Restore NVIDIA settings first (before user hooks)
        if let Err(e) = Self::restore_nvidia_tuning(config) {
            error!("Failed to restore NVIDIA settings: {}", e);
            // Continue execution even if restore fails
        }

        // Execute user-defined shutdown hook
        if let Some(cmd) = hook {
            info!("Executing shutdown hook");
            Self::execute_hook(cmd, "shutdown")?;
        } else {
            debug!("No shutdown hook configured");
        }
        Ok(())
    }

    fn apply_nvidia_tuning(config: &Config) -> Result<()> {
        debug!("Checking NVIDIA tuning configuration");

        match NvTuner::new_from_config(config.tuning.nvidia.clone()) {
            Ok(Some(mut tuner)) => {
                tuner
                    .log_gpu_info()?
                    .log_gpu_stat()?
                    .apply_tuning()
                    .map_err(|e| anyhow::anyhow!("NVML error: {}", e))?;
                info!("NVIDIA tuning applied successfully");
            }
            Ok(None) => {
                debug!("NVIDIA tuning disabled in config");
            }
            Err(e) => {
                warn!(
                    "Could not initialize NVML (NVIDIA GPU may not be available): {}",
                    e
                );
            }
        }

        Ok(())
    }

    fn restore_nvidia_tuning(config: &Config) -> Result<()> {
        debug!("Checking NVIDIA tuning configuration for restore");

        match NvTuner::new_from_config(config.tuning.nvidia.clone()) {
            Ok(Some(mut tuner)) => {
                tuner
                    .restore_defaults()
                    .map_err(|e| anyhow::anyhow!("NVML error: {}", e))?;
                info!("NVIDIA settings restored successfully");
            }
            Ok(None) => {
                debug!("NVIDIA tuning disabled in config, nothing to restore");
            }
            Err(e) => {
                warn!(
                    "Could not initialize NVML (NVIDIA GPU may not be available): {}",
                    e
                );
            }
        }

        Ok(())
    }

    fn execute_hook(cmd: &str, hook_type: &str) -> Result<()> {
        debug!("Running {} hook: {}", hook_type, cmd);

        let status = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()
            .map_err(|e| {
                error!("Failed to execute {} hook: {}", hook_type, e);
                e
            })?;

        if status.success() {
            info!("{} hook completed successfully", hook_type);
        } else {
            warn!(
                "{} hook failed with exit code: {:?}",
                hook_type,
                status.code()
            );
        }

        Ok(())
    }
}
