use primers::common::logging;
use primers::service::privilege::Privilege;
use std::process;

const LOCK_FILE: &str = "/tmp/nvprime-sys.lock";

struct NvPrimeDaemon {
    lock_file: String,
}

impl NvPrimeDaemon {
    pub fn new() {
        println!("Daemon Initialized")
    }
    pub fn hello() {
        println!("Daemon Initialized")
    }
}

pub fn main() {
    logging::init(true);
    let daemon = NvPrimeDaemon::new();
    Privilege::try_elevate();
    process::exit(0);
}
