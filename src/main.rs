mod common;
mod runner;
mod system;

use anyhow::Result;
use common::{config::Config, hooks::HookRunner};
use runner::{env::EnvironmentBuilder, process::ProcessLauncher};
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: prime-rs <executable> [args...]");
        std::process::exit(1);
    }

    // Debug: Show what we're loading
    eprintln!("=== DEBUG: Loading config ===");
    let config = Config::load()?;
    eprintln!("Config loaded successfully");

    HookRunner::run_init(config.hooks.init.as_deref())?;

    // Get executable name - handle Option properly
    let exe_name = get_executable_name(&args[1]);
    eprintln!("DEBUG: Extracted exe name: '{}'", exe_name);

    let mut env_builder = EnvironmentBuilder::new();

    eprintln!("DEBUG: Global env vars from config:");
    for (key, value) in &config.environments.global {
        eprintln!("  {} = {:?}", key, value);
    }

    env_builder.merge_global(&config.environments.global);

    if let Some(exe_config) = config.environments.executables.get(&exe_name) {
        eprintln!("DEBUG: Found exe-specific config for '{}':", exe_name);
        for (key, value) in exe_config {
            eprintln!("  {} = {:?}", key, value);
        }
        env_builder.merge_executable(Some(exe_config));
    } else {
        eprintln!("DEBUG: No exe-specific config for '{}'", exe_name);
        env_builder.merge_executable(None);
    }

    let final_env = env_builder.build();
    eprintln!("DEBUG: Final env vars to set:");
    for (key, value) in &final_env {
        eprintln!("  {} = {}", key, value);
    }
    eprintln!("=========================");

    let launcher = ProcessLauncher::new(args[1].clone(), args[2..].to_vec()).with_env(final_env);

    let exit_code = launcher.execute()?;

    HookRunner::run_shutdown(config.hooks.shutdown.as_deref())?;

    std::process::exit(exit_code);
}

// Return String, not Option<String>
fn get_executable_name(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| path.to_string())
}
