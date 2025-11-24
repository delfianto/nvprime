use log::{debug, info, warn};
use nix::libc;
use std::env;
use std::os::unix::process::CommandExt;
use std::process::Command;

pub struct Privilege;

impl Privilege {
    /// Try to elevate, but continue if it fails (graceful degradation)
    pub fn try_elevate() -> bool {
        if Self::is_root() {
            debug!("Already running with elevated privileges");
            return true;
        }

        if env::var("PRIME_RS_ELEVATED").is_ok() {
            debug!("Already attempted elevation, continuing without privileges");
            return false;
        }

        unsafe {
            env::set_var("PRIME_RS_ELEVATED", "1");
        }

        match Self::run_pkexec() {
            Ok(_) => unreachable!("exec() should replace the process"),
            Err(e) => {
                warn!("Failed to elevate privileges: {}", e);
                false
            }
        }
    }

    /// Check if we're running as root or with sufficient privileges
    fn is_root() -> bool {
        unsafe { libc::geteuid() == 0 }
    }

    /// Re-execute the current binary with pkexec
    fn run_pkexec() -> std::io::Result<()> {
        let exe = env::current_exe()?;
        let args: Vec<String> = env::args().skip(1).collect();

        info!("Re-executing with pkexec for elevated privileges");
        debug!("Current exe: {:?}", exe);
        debug!("Args: {:?}", args);

        // Run the process using polkit
        // Use exec() to replace current process with pkexec
        let error = Command::new("pkexec").arg(&exe).args(&args).exec();

        // If we get here, exec has failed
        Err(error)
    }
}
