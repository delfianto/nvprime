use anyhow::{Context, Result};
use log::{error, info};
use nvprime::common::{Config, ipc::NvPrimeService, logging};
use nvprime::service::DaemonState;
use std::sync::{Arc, Mutex};
use tokio::signal::unix::{SignalKind, signal};

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

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    tokio::select! {
        _ = sigterm.recv() => info!("Received SIGTERM, shutting down"),
        _ = sigint.recv() => info!("Received SIGINT, shutting down"),
    }

    info!("Restoring system defaults...");
    let mut state_lock = state.lock().unwrap();

    if let Err(e) = state_lock.restore_gpu_defaults() {
        error!("Failed to restore GPU defaults: {}", e);
    }

    if let Err(e) = state_lock.restore_cpu_defaults() {
        error!("Failed to restore CPU defaults: {}", e);
    }

    info!("Shutdown complete");

    Ok(())
}
