#![allow(dead_code)]
use anyhow::Result;
use std::process::Command;

pub struct PriorityManager;

impl PriorityManager {
    pub fn set_priority(pid: u32, priority: i32) -> Result<()> {
        // Convert positive to negative as per your config spec
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
}
