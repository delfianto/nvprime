#![allow(dead_code)]

use log::{debug, error, info, warn};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Child, Command, Stdio};

use crate::common::Config;
use crate::runner::EnvBuilder;

#[derive(Debug, Clone)]
struct GameInfo {
    /// Steam AppId (0 if not detected)
    app_id: u32,

    /// Detected game executable name (e.g., "ffxvi")
    exec: String,

    /// Arguments to pass to the game executable
    args: Vec<String>,
}

pub struct Launcher {
    info: GameInfo,
    exec: String,
    args: Vec<String>,
    vars: BTreeMap<String, String>,
    child: Option<Child>,
}

impl Launcher {
    pub fn new(args: Vec<String>, config: &Config) -> Self {
        let info = detect_game_info(&args);
        let vars = EnvBuilder::new().with_config(config, &info.exec);

        debug!("Raw argument from Steam: {:?}", args);
        debug!("Initializing process launcher for game: {:?}", info);

        Launcher {
            info,
            exec: args[0].clone(),
            args: args[1..].to_vec(),
            vars,
            child: None,
        }
    }

    /// Spawns the process but does not wait for it.
    /// Returns the PID of the spawned process.
    pub fn spawn(&mut self) -> anyhow::Result<u32> {
        debug!("Running process '{}' with args: {:?}", self.exec, self.args);
        debug!("Setting environment variables from configs:");
        for (key, val) in &self.vars {
            debug!("  ENV: '{}' with '{}'", key, val);
        }

        let mut cmd = Command::new(&self.exec);
        cmd.args(&self.args)
            .envs(&self.vars)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        cmd.env("PROTON_LOG", "1");
        // cmd.env("MANGOHUD", "1");

        let child = cmd.spawn().map_err(|e| {
            error!("Failed to spawn process {}: {}", self.exec, e);
            anyhow::anyhow!(e)
        })?;

        let pid = child.id();
        info!("Spawned process '{}' with PID {}", self.exec, pid);
        self.child = Some(child);
        Ok(pid)
    }

    /// Waits for the spawned process to finish and returns its exit code.
    pub fn wait(&mut self) -> anyhow::Result<i32> {
        if let Some(child) = &mut self.child {
            debug!(
                "Waiting process '{}' with PID {} to finish",
                self.exec,
                child.id()
            );
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

/// Detect game information from raw args
fn detect_game_info(args: &[String]) -> GameInfo {
    debug!("Detecting game info from args: {:?}", args);

    // Extract Steam AppId if present
    let app_id = args
        .iter()
        .find_map(|arg| {
            arg.strip_prefix("AppId=")
                .and_then(|id_str| id_str.parse::<u32>().ok())
        })
        .unwrap_or_else(|| {
            debug!("Steam AppId not detected, defaulting to 0");
            0
        });

    // Detect executable and indices of remaining args
    let (exec, remaining_indices) = {
        // #1 look for .exe after "waitforexitandrun"
        if let Some(i) = args.iter().position(|arg| arg == "waitforexitandrun") {
            if let Some((j, exe_arg)) = args
                .iter()
                .enumerate()
                .skip(i + 1)
                .find(|(_, arg)| arg.ends_with(".exe"))
            {
                let name = extract_stem(exe_arg);
                debug!(
                    "Detected game '{}' via waitforexitandrun at index {}, remaining: {:?}",
                    name,
                    j + 1,
                    &args[j + 1..]
                );
                (name, (j + 1..args.len()).collect())
            } else {
                // No .exe found after waitforexitandrun, fall back to other methods below
                (String::new(), Vec::new())
            }
        } else {
            (String::new(), Vec::new())
        }
    };

    // If exec not found in strategy #1, try other methods
    let (exec, remaining_indices) = if exec.is_empty() {
        // #2 any .exe, prefer last
        if let Some((i_rev, exe_arg)) = args
            .iter()
            .rev()
            .enumerate()
            .find(|(_, arg)| arg.ends_with(".exe"))
        {
            let actual_index = args.len() - 1 - i_rev;
            let name = extract_stem(exe_arg);
            debug!(
                "Detected game '{}' via .exe scan at index {}, remaining: {:?}",
                name,
                actual_index,
                &args[actual_index + 1..]
            );
            (name, (actual_index + 1..args.len()).collect())
        } else {
            // #3 fallback to first arg
            let name = extract_stem(&args[0]);
            debug!(
                "Using fallback executable name '{}' at index 0, remaining: {:?}",
                name,
                &args[1..]
            );
            (name, (1..args.len()).collect())
        }
    } else {
        (exec, remaining_indices)
    };

    // Collect actual remaining argument strings from indices
    let remaining_args = remaining_indices.iter().map(|&i| args[i].clone()).collect();

    GameInfo {
        app_id,
        exec,
        args: remaining_args,
    }
}

/// Helper to extract filename stem
fn extract_stem(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "unknown".to_string())
}
