use std::collections::HashMap;
use std::process::{Command, Stdio};

pub struct ProcessLauncher {
    executable: String,
    args: Vec<String>,
    env_vars: HashMap<String, String>,
}

impl ProcessLauncher {
    pub fn new(executable: String, args: Vec<String>) -> Self {
        Self {
            executable,
            args,
            env_vars: HashMap::new(),
        }
    }

    pub fn with_env(mut self, env_vars: HashMap<String, String>) -> Self {
        self.env_vars = env_vars;
        self
    }

    pub fn execute(self) -> anyhow::Result<i32> {
        eprintln!("=== DEBUG: Executing process ===");
        eprintln!("Executable: {}", self.executable);
        eprintln!("Args: {:?}", self.args);
        eprintln!("Setting {} env vars", self.env_vars.len());

        // CRITICAL FIX: Set env vars in OUR OWN process first
        // This way ALL child processes (including grandchildren) inherit them
        for (key, value) in &self.env_vars {
            unsafe {
                std::env::set_var(key, value);
            }

            eprintln!("  Set in parent: {}={}", key, value);
        }

        let mut cmd = Command::new(&self.executable);
        cmd.args(&self.args);

        // No need for cmd.envs() anymore - child inherits from parent

        eprintln!("About to spawn process...");
        eprintln!("================================");

        let status = cmd
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        Ok(status.code().unwrap_or(1))
    }
}
