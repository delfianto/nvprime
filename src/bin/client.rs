use anyhow::Result;
use log::{debug, error, info};
use primers::common::{Config, NvGpu, logging};
use primers::runner::{EnvBuilder, Hooks, Launcher};
use std::{path::Path, process};

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

    // NVIDIA GPU initialization
    let uuid = config.tuning.nvidia.device_uuid.as_deref().unwrap_or("");
    match NvGpu::init(uuid.to_string()) {
        Ok(mut nv_gpu) => {
            nv_gpu.log_gpu_info()?;
        }
        Err(e) => {
            error!("Failed to initialize GPU: {:?}", e);
            std::process::exit(1);
        }
    }

    // Run init hooks
    debug!("Running init hooks");
    Hooks::run_init(&config, config.hooks.init.as_deref())?;

    // Get executable name
    let exe_name = get_executable_name(&args[1]);
    debug!("Extracted executable name: '{}'", exe_name);

    let mut env_builder = EnvBuilder::new();

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
    let mut launcher = Launcher::new(args[1].clone(), args[2..].to_vec()).with_env(final_env);

    // Start and wait for completion
    debug!("Launching process");
    let exit_code = launcher.execute()?;

    // Run shutdown hooks
    debug!("Running shutdown hooks");
    Hooks::run_shutdown(&config, config.hooks.shutdown.as_deref())?;

    process::exit(exit_code);
}

fn get_executable_name(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| path.to_string())
}
