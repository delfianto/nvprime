use anyhow::Result;
use log::{debug, error, info};
use primers::common::config::Config;
use primers::common::logging;
use primers::runner::env::EnvironmentBuilder;
use primers::runner::hooks::HookRunner;
use primers::runner::process::ProcessLauncher;
use std::path::Path;
use std::process;

fn main() -> Result<()> {
    logging::init(true)?;
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        error!("Usage: nvprime <executable> [args...]");
        process::exit(1);
    }

    info!("Starting nvprime");
    debug!("Command line arguments: {:?}", args);

    // Load configuration
    debug!("Loading configuration");
    let config = Config::load()?;
    info!("Configuration loaded successfully");

    // Run init hooks
    debug!("Running init hooks");
    HookRunner::run_init(&config, config.hooks.init.as_deref())?;

    // Get executable name
    let exe_name = get_executable_name(&args[1]);
    debug!("Extracted executable name: '{}'", exe_name);

    let mut env_builder = EnvironmentBuilder::new();

    // Merge global environment variables
    debug!(
        "Applying global environment variables: {} vars",
        config.environments.global.len()
    );
    env_builder.merge_global(&config.environments.global);

    // Merge executable-specific environment variables
    if let Some(exe_config) = config.environments.executables.get(&exe_name) {
        info!("Found executable-specific configuration for '{}'", exe_name);
        debug!("Applying {} executable-specific env vars", exe_config.len());
        env_builder.merge_executable(Some(exe_config));
    } else {
        debug!("No executable-specific configuration for '{}'", exe_name);
        env_builder.merge_executable(None);
    }

    let final_env = env_builder.build();
    info!("Built final environment with {} variables", final_env.len());

    let mut launcher =
        ProcessLauncher::new(args[1].clone(), args[2..].to_vec()).with_env(final_env);

    debug!("Launching process");
    let child_pid = launcher.spawn()?;
    info!("Process spawned with PID: {}", child_pid);

    // Wait for process to complete
    let exit_code = launcher.wait()?;
    info!("Process exited with code: {}", exit_code);

    // Run shutdown hooks
    debug!("Running shutdown hooks");
    HookRunner::run_shutdown(&config, config.hooks.shutdown.as_deref())?;

    process::exit(exit_code);
}

fn get_executable_name(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| path.to_string())
}

fn is_tuning_enabled(config: &Config) -> bool {
    config.tuning.process.enabled.unwrap_or(false)
        || config.tuning.nvidia.enabled.unwrap_or(false)
        || config.tuning.ryzen.enabled.unwrap_or(false)
}
