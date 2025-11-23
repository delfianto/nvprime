use anyhow::Result;
use log::{debug, error, info, warn};
use std::process::Command;

pub struct HookRunner;

impl HookRunner {
    pub fn run_init(hook: Option<&str>) -> Result<()> {
        if let Some(cmd) = hook {
            info!("Executing init hook");
            Self::execute_hook(cmd, "init")?;
        } else {
            debug!("No init hook configured");
        }
        Ok(())
    }

    pub fn run_shutdown(hook: Option<&str>) -> Result<()> {
        if let Some(cmd) = hook {
            info!("Executing shutdown hook");
            Self::execute_hook(cmd, "shutdown")?;
        } else {
            debug!("No shutdown hook configured");
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
