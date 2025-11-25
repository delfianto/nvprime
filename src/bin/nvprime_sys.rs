use clap::Parser;
use log::debug;
use primers::common::logging;
use primers::runner::env_utils;
use primers::service::{NvPrimeDaemon, process};
use std::env;
use std::path::PathBuf;

// const LOCK_FILE: &str = "/tmp/nvprime-sys.lock";

#[derive(Parser, Debug)]
#[command(name = "nvprime-daemon")]
#[command(about = "NvPrime system daemon", long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, value_name = "FILE", value_parser = validate_config_file)]
    config: PathBuf,
}

/// Validates that the config file exists and is parseable
fn validate_config_file(path: &str) -> Result<PathBuf, String> {
    let path_buf = PathBuf::from(path);

    // Check if file exists
    if !path_buf.exists() {
        return Err(format!("Config file does not exist: {}", path));
    }

    // Check if it's a file (not a directory)
    if !path_buf.is_file() {
        return Err(format!("Path is not a file: {}", path));
    }

    // Try to parse the config file to ensure it's valid
    // Replace this with your actual config parsing logic
    match std::fs::read_to_string(&path_buf) {
        Ok(contents) => {
            // Add your config parsing validation here
            // For example, if using toml:
            // match toml::from_str::<YourConfigStruct>(&contents) {
            //     Ok(_) => Ok(path_buf),
            //     Err(e) => Err(format!("Failed to parse config file: {}", e)),
            // }

            // Placeholder validation - replace with actual parsing
            if contents.is_empty() {
                Err(format!("Config file is empty: {}", path))
            } else {
                Ok(path_buf)
            }
        }
        Err(e) => Err(format!("Failed to read config file: {}", e)),
    }
}

fn main() {
    let _ = logging::init(true);
    let args = Args::parse();
    let daemon = NvPrimeDaemon::new(args.config);

    process::try_elevate();

    if process::is_root() {
        if let Ok(original_home) = env::var("ORIGINAL_HOME") {
            env_utils::from_strings(&("HOME", &original_home));
            debug!("Restored HOME to ORIGINAL_HOME: {}", original_home);
        }

        debug!("HOME: {}", env_utils::get_value("HOME"));

        daemon.run();

        std::process::exit(0);
    }
}
