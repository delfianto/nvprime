use log::{debug, error, info, warn};
use nvml_wrapper::Nvml;
use nvml_wrapper::enum_wrappers::device::Clock;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::error::NvmlError;

pub struct NvGpu {
    nvml: Nvml,
    gpu_id: GpuId,
}

enum GpuId {
    Index(u32),
    Uuid(String),
}

impl NvGpu {
    /// Initialize NVIDIA GPU support
    pub fn init(uuid: String) -> Result<Self, NvmlError> {
        debug!("Starting NVML initialization");
        let nvml = Nvml::init().map_err(|e| {
            error!("FATAL: NVML initialization failed: {}", e);
            error!("PRIME rendering unavailable. Game will run at ~3 FPS on iGPU.");
            e
        })?;

        let device_identifier = if !uuid.is_empty() {
            GpuId::Uuid(uuid.clone())
        } else {
            debug!("Will use device index 0");
            GpuId::Index(0)
        };

        let device = match &device_identifier {
            GpuId::Uuid(uuid) => nvml.device_by_uuid(uuid.as_str())?,
            GpuId::Index(idx) => nvml.device_by_index(*idx)?,
        };

        let device_name = device.name()?;
        info!("Initialized NVML for {}", device_name);

        Ok(Self {
            nvml,
            gpu_id: device_identifier,
        })
    }

    /// Get device (helper method)
    pub fn get_device(&self) -> Result<nvml_wrapper::Device<'_>, NvmlError> {
        match &self.gpu_id {
            GpuId::Index(idx) => self.nvml.device_by_index(*idx),
            GpuId::Uuid(uuid) => self.nvml.device_by_uuid(uuid.as_str()),
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

        let gpu_clk = device.clock_info(Clock::Graphics)?;
        let mem_clk = device.clock_info(Clock::Memory)?;

        let gpu_load = device.utilization_rates()?;
        let gpu_temp = device.temperature(TemperatureSensor::Gpu)?;
        let fan_speed = device.fan_speed(0).ok();

        debug!("Performance stats:");
        debug!("  Graphics clock: {} MHz", gpu_clk);
        debug!("  Memory clock: {} MHz", mem_clk);
        debug!("  GPU utilization: {}%", gpu_load.gpu);
        debug!("  Memory utilization: {}%", gpu_load.memory);
        debug!("  Temperature: {}°C", gpu_temp);

        if let Some(speed) = fan_speed {
            debug!("  Fan speed: {}%", speed);
        }

        Ok(self)
    }

    /// Set the GPU power limit, need superuser access
    pub fn set_power_limit(
        &mut self,
        power_limit: Option<u32>,
        set_max_pwr: Option<bool>,
    ) -> Result<&mut Self, NvmlError> {
        let mut device = self.get_device()?;
        let device_name = device.name()?;

        info!("Setting NVIDIA power limit for: {}", device_name);
        let pm = device.power_management_limit_constraints()?;

        debug!(
            "Power constraints: min={}mW, max={}mW",
            pm.min_limit, pm.max_limit
        );

        // Apply gaming profile (max power limit) if set_max_pwr is true
        if set_max_pwr.unwrap_or(false) {
            device.set_power_management_limit(pm.max_limit)?;
            info!("Set power limit to maximum: {}mW", pm.max_limit);
        } else if let Some(requested_limit) = power_limit {
            // Apply custom power limit if specified
            let clamped_limit = requested_limit.clamp(pm.min_limit, pm.max_limit);

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
        let temp = device.temperature(TemperatureSensor::Gpu)?;

        debug!("Enforced power limit: {}mW", enforced_power);
        debug!("GPU temperature: {}°C", temp);

        Ok(self)
    }

    /// Restore GPU to default settings, need superuser access
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
