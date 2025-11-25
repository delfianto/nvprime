use anyhow::Result;
use log::{debug, info, warn};
use nix::libc;
use std::env;
use std::os::unix::process::CommandExt;
use std::process::Command;

/// Set priority of a PID
pub fn set_priority(pid: u32, priority: i32) -> Result<()> {
    // This converts positive to negative
    // E.g. priority 10 means renice it to -10
    let nice_value = -priority.abs();

    let status = Command::new("renice")
        .arg(nice_value.to_string())
        .arg("-p")
        .arg(pid.to_string())
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to set process priority");
    }

    Ok(())
}

/// Check if we're running as root
pub fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

/// Try to elevate the privilege and handle any failure
pub fn try_elevate() -> bool {
    if is_root() {
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

    match run_pkexec() {
        Ok(_) => unreachable!("exec() should replace the process"),
        Err(e) => {
            warn!("Failed to elevate privileges: {}", e);
            false
        }
    }
}

/// Re-execute the current binary with pkexec
fn run_pkexec() -> std::io::Result<()> {
    let exe = env::current_exe()?;
    let args: Vec<String> = env::args().skip(1).collect();

    info!("Re-executing with pkexec for elevated privileges");
    debug!("Current exe: {:?}", exe);
    debug!("Args: {:?}", args);

    // Execute pkexec to elevate privilege
    let error = Command::new("pkexec").arg(&exe).args(&args).exec();

    // *If* we get here, exec has failed
    Err(error)
}
