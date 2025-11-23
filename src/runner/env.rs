use crate::common::config::EnvValue;
use std::collections::HashMap;

pub struct EnvironmentBuilder {
    vars: HashMap<String, String>,
}

impl EnvironmentBuilder {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    /// Merge global environment variables from config
    pub fn merge_global(&mut self, global: &HashMap<String, EnvValue>) {
        for (key, value) in global {
            self.vars.insert(key.clone(), value.to_string());
        }
    }

    /// Merge executable-specific environment variables
    pub fn merge_executable(&mut self, exe_vars: Option<&HashMap<String, EnvValue>>) {
        if let Some(vars) = exe_vars {
            for (key, value) in vars {
                self.vars.insert(key.clone(), value.to_string());
            }
        }
    }

    /// Build the final environment map
    pub fn build(self) -> HashMap<String, String> {
        self.vars
    }
}
