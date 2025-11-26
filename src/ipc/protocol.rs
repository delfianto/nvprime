use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// Commands that can be sent to the daemon
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub enum Command {
    /// Apply CPU and GPU tuning parameters for gaming
    ApplyTuning { cmd: String, pid: u32 },
    /// Reset everything to default state
    ResetTuning,
    /// Query current daemon status
    Status,
}

/// Response status codes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
#[repr(u8)]
pub enum StatusCode {
    Success = 0,
    Failure = 1,
}

impl From<StatusCode> for u8 {
    fn from(code: StatusCode) -> Self {
        code as u8
    }
}

/// Response from the daemon - tuple of (status_code, message)
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Response {
    pub status: StatusCode,
    pub message: String,
}

impl Response {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::Success,
            message: message.into(),
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::Failure,
            message: message.into(),
        }
    }

    /// Convert to tuple representation (status_code, message)
    pub fn to_tuple(&self) -> (u8, String) {
        (self.status.into(), self.message.clone())
    }
}

/// Request wrapper with protocol version for future compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub version: u8,
    pub command: Command,
}

impl Request {
    pub const CURRENT_VERSION: u8 = 1;

    pub fn new(command: Command) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            command,
        }
    }
}
