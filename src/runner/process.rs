use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::process::{Command, Stdio};

pub struct ProcessLauncher {
    executable: String,
    args: Vec<String>,
    env_vars: HashMap<String, String>,
}

impl ProcessLauncher {
    pub fn new(executable: String, args: Vec<String>) -> Self {
        debug!("Creating ProcessLauncher for: {}", executable);
        Self {
            executable,
            args,
            env_vars: HashMap::new(),
        }
    }

    pub fn with_env(mut self, env_vars: HashMap<String, String>) -> Self {
        debug!("Adding {} environment variables to process", env_vars.len());
        self.env_vars = env_vars;
        self
    }

    pub fn execute(self) -> anyhow::Result<i32> {
        info!("Executing: {} {:?}", self.executable, self.args);
        debug!("Setting {} environment variables", self.env_vars.len());

        // Set env vars in parent process so child processes inherit them
        for (key, value) in &self.env_vars {
            debug!("  Setting env: {}={}", key, value);
            unsafe {
                std::env::set_var(key, value);
            }
        }

        let mut cmd = Command::new(&self.executable);
        cmd.args(&self.args);

        debug!("Spawning process with inherited stdio");

        let status = cmd
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| {
                error!("Failed to spawn process '{}': {}", self.executable, e);
                e
            })?;

        let exit_code = status.code().unwrap_or(1);

        if status.success() {
            info!(
                "Process completed successfully with exit code: {}",
                exit_code
            );
        } else {
            warn!("Process exited with non-zero code: {}", exit_code);
        }

        Ok(exit_code)
    }
}
