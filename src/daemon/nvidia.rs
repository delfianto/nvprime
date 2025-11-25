use crate::common::config::NvGpuConfig;
use crate::daemon::privilege::Privilege;

use log::{debug, info, warn};
use nvml_wrapper::Nvml;
use nvml_wrapper::enum_wrappers::device::Clock;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::error::NvmlError;

pub struct NvTuner {
    nvml: Nvml,
    config: NvGpuConfig,
    device_identifier: DeviceId,
}

enum DeviceId {
    Index(u32),
    Uuid(String),
}

impl NvTuner {
    /// Initialize GpuTuner only if config is enabled
    pub fn from_config(config: NvGpuConfig) -> Result<Option<Self>, NvmlError> {
        if !config.enabled.unwrap_or(false) {
            debug!("NVIDIA tuning is disabled in config");
            return Ok(None);
        }

        info!("NVIDIA tuning enabled, checking privileges");

        // Try to get privileged access for NVML
        if !Privilege::try_elevate() {
            warn!("Fail to obtain elevated privileges, NVIDIA tuning may not work properly");
        }

        debug!("Initializing NVML");
        let nvml = Nvml::init()?;

        let device_identifier = if let Some(ref uuid) = config.device_uuid {
            info!("Will use device UUID: {}", uuid);
            DeviceId::Uuid(uuid.clone())
        } else {
            debug!("Will use device index 0");
            DeviceId::Index(0)
        };

        // Verify device exists
        let device = match &device_identifier {
            DeviceId::Index(idx) => nvml.device_by_index(*idx)?,
            DeviceId::Uuid(uuid) => nvml.device_by_uuid(uuid.as_str())?,
        };

        let device_name = device.name()?;
        info!("Initialized GpuTuner for device: {}", device_name);

        Ok(Some(Self {
            nvml,
            device_identifier,
            config,
        }))
    }

    /// Get device (helper method)
    fn get_device(&self) -> Result<nvml_wrapper::Device<'_>, NvmlError> {
        match &self.device_identifier {
            DeviceId::Index(idx) => self.nvml.device_by_index(*idx),
            DeviceId::Uuid(uuid) => self.nvml.device_by_uuid(uuid.as_str()),
        }
    }

    /// Get and log GPU information
    pub fn log_gpu_info(&mut self) -> Result<&mut Self, NvmlError> {
        let device = self.get_device()?;

        let name = device.name()?;
        let brand = device.brand()?;
        let uuid = device.uuid()?;
        let memory_info = device.memory_info()?;
        let enforced_power = device.enforced_power_limit()?;

        info!("GPU: {} ({:?})", name, brand);
        info!("UUID: {}", uuid);
        info!(
            "Memory: {:.2}GB / {:.2}GB",
            memory_info.used as f64 / 1024.0 / 1024.0 / 1024.0,
            memory_info.total as f64 / 1024.0 / 1024.0 / 1024.0
        );
        info!("Power limit: {}mW", enforced_power);

        Ok(self)
    }

    /// Monitor and log GPU performance metrics
    pub fn log_gpu_stat(&mut self) -> Result<&mut Self, NvmlError> {
        let device = self.get_device()?;

        let graphics_clock = device.clock_info(Clock::Graphics)?;
        let memory_clock = device.clock_info(Clock::Memory)?;

        let utilization = device.utilization_rates()?;
        let temp = device.temperature(TemperatureSensor::Gpu)?;
        let fan_speed = device.fan_speed(0).ok();

        debug!("Performance stats:");
        debug!("  Graphics clock: {} MHz", graphics_clock);
        debug!("  Memory clock: {} MHz", memory_clock);
        debug!("  GPU utilization: {}%", utilization.gpu);
        debug!("  Memory utilization: {}%", utilization.memory);
        debug!("  Temperature: {}°C", temp);
        if let Some(speed) = fan_speed {
            debug!("  Fan speed: {}%", speed);
        }

        Ok(self)
    }

    // Apply tuning profile from stored config
    pub fn apply_tuning(&mut self) -> Result<&mut Self, NvmlError> {
        let mut device = self.get_device()?;
        let device_name = device.name()?;
        info!("Applying NVIDIA tuning to device: {}", device_name);

        // Apply gaming profile (max power limit) if set_max is enabled
        if self.config.set_max.unwrap_or(false) {
            let constraints = device.power_management_limit_constraints()?;
            debug!(
                "Power constraints: min={}mW, max={}mW",
                constraints.min_limit, constraints.max_limit
            );

            device.set_power_management_limit(constraints.max_limit)?;
            info!("Set power limit to maximum: {}mW", constraints.max_limit);
        }
        // Apply custom power limit if specified
        else if let Some(requested_limit) = self.config.power_limit {
            let constraints = device.power_management_limit_constraints()?;
            debug!(
                "Power constraints: min={}mW, max={}mW",
                constraints.min_limit, constraints.max_limit
            );

            let clamped_limit = requested_limit.clamp(constraints.min_limit, constraints.max_limit);

            if clamped_limit != requested_limit {
                warn!(
                    "Requested power limit {}mW is out of range, clamping to {}mW",
                    requested_limit, clamped_limit
                );
            }

            device.set_power_management_limit(clamped_limit)?;
            info!("Set power limit to: {}mW", clamped_limit);
        }

        // Verify and log current state
        let enforced_power = device.enforced_power_limit()?;
        let temp =
            device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)?;

        debug!("Enforced power limit: {}mW", enforced_power);
        debug!("GPU temperature: {}°C", temp);

        Ok(self)
    }

    /// Restore GPU to default settings
    pub fn restore_defaults(&mut self) -> Result<&mut Self, NvmlError> {
        let mut device = self.get_device()?;
        let device_name = device.name()?;
        info!("Restoring NVIDIA defaults for device: {}", device_name);

        let default_power = device.power_management_limit_default()?;
        device.set_power_management_limit(default_power)?;
        info!("Restored power limit to default: {}mW", default_power);

        Ok(self)
    }
}
