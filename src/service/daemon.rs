use log::debug;
use std::path::PathBuf;

pub struct NvPrimeDaemon {
    config_path: PathBuf,
}

impl NvPrimeDaemon {
    pub fn new(config_path: PathBuf) -> Self {
        println!("Daemon Initialized with config: {:?}", config_path);
        Self { config_path }
    }

    pub fn run(&self) {
        // Your daemon logic here using self.config_path
        debug!("Using config file: {:?}", self.config_path);
    }
}
