use anyhow::{Context, Result};
use log::info;
use nvprime::common::{ipc::NvPrimeService, logging, Config};
use nvprime::service::DaemonState;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<()> {
    logging::init(true).context("Failed to initialize logging")?;

    info!("Starting nvprime system daemon");

    let config = Config::load().context("Failed to load configuration")?;

    let state = Arc::new(Mutex::new(DaemonState::new()));

    if config.gpu.enabled {
        let mut state_lock = state.lock().unwrap();
        state_lock
            .init_gpu(config.gpu.gpu_uuid.clone())
            .context("Failed to initialize GPU")?;
    }

    let service = NvPrimeService::new(Arc::clone(&state));

    let _conn = zbus::connection::Builder::system()?
        .name("com.github.nvprime")?
        .serve_at("/com/github/nvprime", service)?
        .build()
        .await?;

    info!("D-Bus service started on system bus");
    info!("Waiting for requests...");

    std::future::pending::<()>().await;

    Ok(())
}
