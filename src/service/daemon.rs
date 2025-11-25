use log::debug;
use std::path::PathBuf;

// const LOCK_FILE: &str = "/tmp/nvprime-sys.lock";

pub struct NvPrimeDaemon {
    config_path: PathBuf,
}

impl NvPrimeDaemon {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    pub fn run(&self) {
        debug!("Validated config file: {:?}", self.config_path);
    }
}
