use anyhow::{Context, Result};
use log::{error, info};
use nvprime::common::{Config, NvPrimeClientProxy, logging};
use nvprime::runner::Launcher;
use zbus::Connection;

#[tokio::main]
async fn main() -> Result<()> {
    logging::init(true)?;

    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        error!("Usage: nvprime <executable> [args...]");
        std::process::exit(1);
    }

    info!("Starting nvprime");
    let config = Config::load()?;

    let conn = Connection::system()
        .await
        .context("Failed to connect to system bus")?;

    let proxy = NvPrimeClientProxy::new(&conn)
        .await
        .context("Failed to create D-Bus proxy")?;

    let tuning_config = serde_json::json!({
        "cpu": config.cpu,
        "gpu": config.gpu,
        "sys": config.sys,
    });

    let config_json =
        serde_json::to_string(&tuning_config).context("Failed to serialize config")?;

    let pid = std::process::id();

    proxy
        .apply_tuning(pid, config_json)
        .await
        .context("Failed to apply tuning")?;

    info!("Applied tuning configuration");

    let mut launcher = Launcher::new(args, &config);
    let exit_code = launcher.execute()?;

    if let Err(e) = proxy.reset_tuning().await {
        error!("Failed to reset tuning: {}", e);
    }

    std::process::exit(exit_code);
}
