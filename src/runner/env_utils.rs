use log::debug;
use std::collections::HashMap;
use std::env;

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
