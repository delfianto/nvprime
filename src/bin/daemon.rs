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
    let daemon = NvPrimeDaemon::new();
}
