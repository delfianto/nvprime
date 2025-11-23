use crate::common::config::EnvValue;
use log::debug;
use std::collections::HashMap;

pub struct EnvironmentBuilder {
    vars: HashMap<String, String>,
}

impl EnvironmentBuilder {
    pub fn new() -> Self {
        debug!("Creating new EnvironmentBuilder");
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
