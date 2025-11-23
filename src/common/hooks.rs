use std::process::Command;
use anyhow::Result;

pub struct HookRunner;

impl HookRunner {
    pub fn run_init(hook: Option<&str>) -> Result<()> {
        if let Some(cmd) = hook {
            Self::execute_hook(cmd, "init")?;
        }
        Ok(())
    }

    pub fn run_shutdown(hook: Option<&str>) -> Result<()> {
        if let Some(cmd) = hook {
            Self::execute_hook(cmd, "shutdown")?;
        }
        Ok(())
    }

    fn execute_hook(cmd: &str, hook_type: &str) -> Result<()> {
        let status = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()?;

        if !status.success() {
            eprintln!("Warning: {} hook failed: {}", hook_type, cmd);
        }

        Ok(())
    }
}
