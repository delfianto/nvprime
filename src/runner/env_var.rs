#![allow(dead_code)]

use crate::common::config::EnvValue;
use log::debug;
use std::collections::HashMap;
use std::env;

pub struct EnvBuilder {
    vars: HashMap<String, String>,
}

impl EnvBuilder {
    pub fn new() -> Self {
        debug!("Creating new environment builder");
        Self {
            vars: HashMap::new(),
        }
    }

    /// Merge global environment variables from config
    pub fn merge_global(&mut self, global: &HashMap<String, EnvValue>) {
        debug!("Merging {} global environment variables", global.len());
        for (key, value) in global {
            let value_str = value.to_string();
            debug!("  Adding global: {} = {}", key, value_str);
            self.vars.insert(key.clone(), value_str);
        }
    }

    /// Merge executable-specific environment variables
    pub fn merge_executable(&mut self, exe_vars: Option<&HashMap<String, EnvValue>>) {
        if let Some(vars) = exe_vars {
            debug!(
                "Merging {} executable-specific environment variables",
                vars.len()
            );
            for (key, value) in vars {
                let value_str = value.to_string();
                debug!("  Adding executable-specific: {} = {}", key, value_str);
                self.vars.insert(key.clone(), value_str);
            }
        } else {
            debug!("No executable-specific environment variables to merge");
        }
    }

    /// Build the final environment map
    pub fn build(self) -> HashMap<String, String> {
        debug!(
            "Building final environment map with {} variables",
            self.vars.len()
        );
        self.vars
    }
}

const EMPTY_STRING: &str = "EMPTY_STRING";
const NOT_PRESENT: &str = "NOT_PRESENT";

pub fn get_value(key: &str) -> String {
    match env::var(key) {
        Ok(val) => {
            if val.is_empty() {
                EMPTY_STRING.to_string()
            } else {
                val
            }
        }
        Err(_) => NOT_PRESENT.to_string(),
    }
}

pub fn from_strings(env_vars: &(&str, &str)) {
    unsafe {
        env::set_var(env_vars.0, env_vars.1);
        debug!("  Setting Vars: {} = {}", env_vars.0, env_vars.1);
    }
}

pub fn from_slices(env_vars: &[(&str, &str)]) {
    for (key, val) in env_vars {
        unsafe {
            env::set_var(key, val);
            debug!("  Setting Vars: {} = {}", key, val);
        }
    }
}

pub fn from_hashmap(env_vars: &HashMap<String, String>) {
    debug!("Setting environment variables in parent context:");

    for (key, val) in env_vars {
        unsafe {
            env::set_var(key, val);
            debug!("  Setting Vars: {} = {}", key, val);
        }
    }
}
