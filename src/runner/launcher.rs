use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};

pub struct Launcher {
    cmnd: String,
    args: Vec<String>,
    vars: HashMap<String, String>,
    child: Option<Child>,
}

impl Launcher {
    pub fn new(cmd: String, args: Vec<String>) -> Self {
        debug!("Initializing process launcher for executable: {}", cmd);
        Launcher {
            cmnd: cmd,
            args,
            vars: HashMap::new(),
            child: None,
        }
    }

    pub fn with_env(mut self, env_vars: HashMap<String, String>) -> Self {
        debug!(
            "Adding environment variables to process, count: {}",
            env_vars.len()
        );
        self.vars = env_vars;
        self
    }

    /// Spawns the process but does not wait for it.
    /// Returns the PID of the spawned process.
    pub fn spawn(&mut self) -> anyhow::Result<u32> {
        debug!("Setting environment variables in parent context:");
        for (key, value) in &self.vars {
            unsafe {
                std::env::set_var(key, value);
                debug!("  Setting Vars: {} = {}", key, value);
            }
        }

        debug!("Running process '{}' with args: {:?}", self.cmnd, self.args);

        let mut cmd = Command::new(&self.cmnd);
        cmd.args(&self.args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        // Environment variables for the child process
        for (key, value) in &self.vars {
            cmd.env(key, value);
        }

        let child = cmd.spawn().map_err(|e| {
            error!("Failed to spawn process {}: {}", self.cmnd, e);
            anyhow::anyhow!(e)
        })?;

        let pid = child.id();
        info!("Spawned process '{}' with PID {}", self.cmnd, pid);
        self.child = Some(child);
        Ok(pid)
    }

    /// Waits for the spawned process to finish and returns its exit code.
    pub fn wait(&mut self) -> anyhow::Result<i32> {
        if let Some(child) = &mut self.child {
            debug!("Waiting process '{}' with PID {}", self.cmnd, child.id());
            let status = child.wait().map_err(|e| {
                error!("Failed waiting on process PID {}: {}", child.id(), e);
                anyhow::anyhow!(e)
            })?;

            let exit_code = status.code().unwrap_or(-1);
            if status.success() {
                info!(
                    "Process PID {} completed successfully with exit code {}",
                    child.id(),
                    exit_code
                );
            } else {
                warn!(
                    "Process PID {} exited with non-zero code {}",
                    child.id(),
                    exit_code
                );
            }
            Ok(exit_code)
        } else {
            Err(anyhow::anyhow!("No running process to wait for"))
        }
    }

    /// Combined spawn and wait function for convenience.
    pub fn execute(&mut self) -> anyhow::Result<i32> {
        self.spawn()?;
        self.wait()
    }
}
