use clap::Parser;
use nvprime::common::{Config, logging};
use nvprime::service::{NvPrimeDaemon, process};
use std::path::PathBuf;

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

    // Check if it's a file
    if !path_buf.is_file() {
        return Err(format!("Path is not a file: {}", path));
    }

    // Try to parse the config file to ensure it's valid
    match std::fs::read_to_string(&path_buf) {
        Ok(contents) => {
            if contents.is_empty() {
                Err(format!("Config file is empty: {}", path))
            } else {
                let _ = Config::load_file(path_buf.clone());
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
    daemon.run();
}
