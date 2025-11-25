use crate::common::NvGpu;

pub struct NvTuner {
    gpu: NvGpu,
}

enum DeviceId {
    Index(u32),
    Uuid(String),
}

impl NvTuner {}
