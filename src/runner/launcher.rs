use log::{debug, error, info, warn};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Child, Command, Stdio};

use crate::common::Config;
use crate::runner::EnvBuilder;

pub struct Launcher {
    exec: String,
    args: Vec<String>,
    vars: BTreeMap<String, String>,
    child: Option<Child>,
}

impl Launcher {
    pub fn new(args: Vec<String>, config: &Config) -> Self {
        let game_exec = detect_game_exec(&args);
        let vars = EnvBuilder::new().with_config(config, &game_exec);

        debug!("Raw argument from Steam: {:?}", args);
        debug!("Detected game executable: {}", game_exec);

        Launcher {
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

        let child = Command::new(&self.exec)
            .args(&self.args)
            .envs(&self.vars)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| {
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

fn detect_game_exec(args: &[String]) -> String {
    debug!("Detecting game executable from args");

    if let Some(i) = args.iter().position(|arg| arg == "waitforexitandrun")
        && let Some((_, exe_arg)) = args
            .iter()
            .enumerate()
            .skip(i + 1)
            .find(|(_, arg)| arg.ends_with(".exe"))
    {
        let name = extract_stem(exe_arg);
        debug!("Detected game '{}' via waitforexitandrun", name);
        return name;
    }

    if let Some((_, exe_arg)) = args
        .iter()
        .enumerate()
        .rev()
        .find(|(_, arg)| arg.ends_with(".exe"))
    {
        let name = extract_stem(exe_arg);
        debug!("Detected game '{}' via .exe scan", name);
        return name;
    }

    let name = extract_stem(&args[0]);
    debug!("Using fallback executable name '{}'", name);
    name
}

fn extract_stem(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_stem_simple() {
        assert_eq!(extract_stem("game.exe"), "game");
        assert_eq!(extract_stem("MyGame.EXE"), "mygame");
    }

    #[test]
    fn test_extract_stem_with_path() {
        assert_eq!(extract_stem("/path/to/game.exe"), "game");
        #[cfg(windows)]
        assert_eq!(extract_stem("C:\\Games\\MyGame.exe"), "mygame");
        #[cfg(not(windows))]
        assert_eq!(extract_stem("C:\\Games\\MyGame.exe"), "c:\\games\\mygame");
    }

    #[test]
    fn test_extract_stem_no_extension() {
        assert_eq!(extract_stem("game"), "game");
        assert_eq!(extract_stem("/path/to/launcher"), "launcher");
    }

    #[test]
    fn test_extract_stem_multiple_dots() {
        assert_eq!(extract_stem("game.version.1.2.exe"), "game.version.1.2");
    }

    #[test]
    fn test_detect_game_exec_waitforexitandrun() {
        let args = vec![
            "proton".to_string(),
            "waitforexitandrun".to_string(),
            "steam.exe".to_string(),
            "game.exe".to_string(),
            "arg1".to_string(),
        ];

        assert_eq!(detect_game_exec(&args), "steam");
    }

    #[test]
    fn test_detect_game_exec_first_exe() {
        let args = vec![
            "launcher.exe".to_string(),
            "arg1".to_string(),
            "arg2".to_string(),
        ];

        assert_eq!(detect_game_exec(&args), "launcher");
    }

    #[test]
    fn test_detect_game_exec_last_exe() {
        let args = vec![
            "proton".to_string(),
            "run".to_string(),
            "launcher.exe".to_string(),
            "game.exe".to_string(),
        ];

        assert_eq!(detect_game_exec(&args), "game");
    }

    #[test]
    fn test_detect_game_exec_no_exe() {
        let args = vec!["launcher".to_string(), "arg1".to_string()];

        assert_eq!(detect_game_exec(&args), "launcher");
    }

    #[test]
    fn test_detect_game_exec_complex_steam_args() {
        let args = vec![
            "/path/to/proton".to_string(),
            "waitforexitandrun".to_string(),
            "/path/steam.exe".to_string(),
            "-applaunch".to_string(),
            "AppId=12345".to_string(),
            "/game/FinalFantasy.exe".to_string(),
            "-windowed".to_string(),
        ];

        assert_eq!(detect_game_exec(&args), "steam");
    }

    #[test]
    fn test_detect_game_exec_only_final_exe() {
        let args = vec![
            "launcher".to_string(),
            "/game/FinalFantasy.exe".to_string(),
            "-windowed".to_string(),
        ];

        assert_eq!(detect_game_exec(&args), "finalfantasy");
    }

    fn create_test_config() -> Config {
        Config {
            cpu: Default::default(),
            gpu: Default::default(),
            sys: Default::default(),
            env: Default::default(),
            game: Default::default(),
            hook: Default::default(),
        }
    }

    #[test]
    fn test_launcher_new() {
        let args = vec![
            "game.exe".to_string(),
            "arg1".to_string(),
            "arg2".to_string(),
        ];
        let config = create_test_config();

        let launcher = Launcher::new(args.clone(), &config);

        assert_eq!(launcher.exec, "game.exe");
        assert_eq!(launcher.args, vec!["arg1".to_string(), "arg2".to_string()]);
        assert!(!launcher.vars.is_empty());
        assert!(launcher.child.is_none());
    }

    #[test]
    fn test_launcher_new_single_arg() {
        let args = vec!["game.exe".to_string()];
        let config = create_test_config();

        let launcher = Launcher::new(args, &config);

        assert_eq!(launcher.exec, "game.exe");
        assert!(launcher.args.is_empty());
    }

    #[test]
    fn test_launcher_wait_without_spawn() {
        let args = vec!["test".to_string()];
        let config = create_test_config();
        let mut launcher = Launcher::new(args, &config);

        let result = launcher.wait();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No running process")
        );
    }
}
