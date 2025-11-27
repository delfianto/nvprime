use anyhow::Result;
use log::{debug, error, info};
use nvprime::common::{Config, NvGpu, logging};
use nvprime::runner::Launcher;

fn main() -> Result<()> {
    logging::init(true)?;

    // Remove 1st argument as this is the nvprime itself
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        error!("Usage: nvprime <executable> [args...]");
        std::process::exit(1);
    }

    info!("Starting nvprime");
    let config = Config::load()?;

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

    debug!("Launching process...");
    let mut launcher = Launcher::new(args, &config);

    // Start and wait for completion
    let exit_code = launcher.execute()?;

    // Run shutdown hooks
    // debug!("Running shutdown hooks");
    // Hooks::run_shutdown(&config, config.hooks.shutdown.as_deref())?;

    std::process::exit(exit_code);
}
