pub mod config;
pub mod nvidia;
pub mod protocol; // IPC protocol definitions

// Re-export commonly used types
pub use config::{Config, NvGpuConfig};
pub use nvidia::NvidiaDevice;
pub use protocol::{DaemonRequest, DaemonResponse};
