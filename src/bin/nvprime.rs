use anyhow::Result;
use log::{debug, error, info};
use primers::common::{Config, NvGpu, logging};
use primers::runner::Launcher;

fn main() -> Result<()> {
    logging::init(true)?;
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        error!("Usage: nvprime <executable> [args...]");
        std::process::exit(1);
    }

    info!("Starting nvprime");
    // debug!("Command line arguments: {:?}", args);

    // Load configuration
    // debug!("Loading configuration...");
    let config = Config::load()?;
    // info!("Configuration loaded successfully");

    // NVIDIA GPU initialization
    match NvGpu::init(config.gpu.gpu_uuid.clone()) {
        Ok(mut nv_gpu) => {
            nv_gpu.log_gpu_info()?;
        }
        Err(e) => {
            error!("Failed to initialize GPU: {:?}", e);
            std::process::exit(1);
        }
    }

    // Run init hooks
    // debug!("Running init hooks");
    // Hooks::run_init(&config, config.hooks.init.as_deref())?;

    // Get executable name
    // let exe_name = get_executable_name(&args);
    // debug!("Detected executable name: '{}'", exe_name);

    // let env_builder = EnvBuilder::new();

    // Merge global environment variables
    // debug!(
    //     "Applying global environment variables: {} vars",
    //     config.environments.global.len()
    // );
    // env_builder.merge_global(&config.environments.global);

    // Merge executable-specific environment variables
    // if let Some(exe_config) = config.environments.executables.get(&exe_name) {
    //     info!("Found executable-specific configuration for '{}'", exe_name);
    //     debug!("Applying {} executable-specific env vars", exe_config.len());
    //     env_builder.merge_executable(Some(exe_config));
    // } else {
    //     debug!("No executable-specific configuration for '{}'", exe_name);
    //     env_builder.merge_executable(None);
    // }

    debug!("Launching process...");
    // let final_env = env_builder.with_config(&config, &exe_name);
    let mut launcher = Launcher::new(args);

    // Start and wait for completion
    // debug!("Launching process");
    let exit_code = launcher.execute()?;

    // Run shutdown hooks
    // debug!("Running shutdown hooks");
    // Hooks::run_shutdown(&config, config.hooks.shutdown.as_deref())?;

    std::process::exit(exit_code);
}
