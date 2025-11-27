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
fn detect_game_info(raw_args: &[String]) -> GameInfo {
    if raw_args.is_empty() {
        return GameInfo {
            app_id: 0, // default to zero when no args
            exec: "unknown".to_string(),
            args: Vec::new(),
        };
    }

    let app_id = extract_app_id(raw_args);
    let exec = detect_executable_name(raw_args);
    let args = raw_args[1..].to_vec();

    GameInfo { app_id, exec, args }
}

fn detect_executable_name(args: &[String]) -> String {
    debug!("Detecting executable from args: {:?}", args);

    // #1 Get the .exe after "waitforexitandrun"
    for (i, arg) in args.iter().enumerate() {
        if arg == "waitforexitandrun" {
            for next_arg in &args[i + 1..] {
                if next_arg.ends_with(".exe") {
                    let name = extract_stem(next_arg);
                    debug!("Detected game '{}' via waitforexitandrun", name);
                    return name;
                }
            }
        }
    }

    // #2 any .exe, prefer last
    for arg in args.iter().rev() {
        if arg.ends_with(".exe") {
            let name = extract_stem(arg);
            debug!("Detected game '{}' via .exe scan", name);
            return name;
        }
    }

    // #3 If none worked, fallback to first arg filename
    let fallback = extract_stem(&args[0]);
    debug!("Using fallback executable name: {}", fallback);
    fallback
}

/// Extract Steam AppId if present, else default to zero
fn extract_app_id(args: &[String]) -> u32 {
    for arg in args {
        if let Some(appid_str) = arg.strip_prefix("AppId=") {
            if let Ok(appid) = appid_str.parse::<u32>() {
                debug!("Detected Steam AppId: {}", appid);
                return appid;
            }
        }
    }

    debug!("Steam AppId not detected, defaulting to 0");
    0
}

/// Helper to extract filename stem, no changes needed
fn extract_stem(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "unknown".to_string())
}
