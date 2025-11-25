use log::debug;
use primers::common::logging;
use primers::runner::env_utils;
use primers::service::privilege::Privilege;
use std::env;
use std::process;

// const LOCK_FILE: &str = "/tmp/nvprime-sys.lock";

struct NvPrimeDaemon {
    // lock_file: String,
}

impl NvPrimeDaemon {
    pub fn new() {
        println!("Daemon Initialized")
    }
}

pub fn main() {
    let _ = logging::init(true);
    NvPrimeDaemon::new();
    Privilege::try_elevate();

    if Privilege::is_root() {
        if let Ok(original_home) = env::var("ORIGINAL_HOME") {
            env_utils::from_strings(&("HOME", &original_home));
            debug!("Restored HOME to ORIGINAL_HOME: {}", original_home);
        }
    }

    debug!("HOME: {}", env_utils::get_value("HOME"));
    process::exit(0);
}
