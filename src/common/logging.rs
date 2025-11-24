use anyhow::Result;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

/// Initialize the logging system with pretty formatting
pub fn init(verbose: bool) -> Result<()> {
    let level = if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    Builder::new()
        .filter_level(level)
        .format(format_log)
        .try_init()?;

    Ok(())
}

/// Shared log formatter function that can be used in production and tests
fn format_log(buf: &mut env_logger::fmt::Formatter, record: &log::Record) -> std::io::Result<()> {
    // Extract just the module name (last component after ::)
    let target = record.target();
    let module = target.split("::").last().unwrap_or(target);

    // Format level with color and padding
    let level_str = match record.level() {
        log::Level::Error => "\x1b[31mERROR\x1b[0m", // Red
        log::Level::Warn => "\x1b[33mWARN \x1b[0m",  // Yellow
        log::Level::Info => "\x1b[32mINFO \x1b[0m",  // Green
        log::Level::Debug => "\x1b[36mDEBUG\x1b[0m", // Cyan
        log::Level::Trace => "\x1b[35mTRACE\x1b[0m", // Magenta
    };

    // Get current time in simple format
    let time = chrono::Local::now().format("%H:%M:%S");

    // Write formatted log with consistent padding
    // Module name padded to 8 characters, right-aligned
    writeln!(
        buf,
        "{} {} [{:>8}] {}",
        time,
        level_str,
        module,
        record.args()
    )
}
