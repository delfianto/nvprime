use anyhow::Result;
use log::{debug, error, info, warn};
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// Valid AMD EPP profiles
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EppProfile {
    Performance,
    BalancePerformance,
    Default,
    BalancePower,
    Power,
}

impl FromStr for EppProfile {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "performance" => Ok(EppProfile::Performance),
            "balance_performance" => Ok(EppProfile::BalancePerformance),
            "default" => Ok(EppProfile::Default),
            "balance_power" => Ok(EppProfile::BalancePower),
            "power" => Ok(EppProfile::Power),
            _ => Err(()),
        }
    }
}

impl EppProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            EppProfile::Performance => "performance",
            EppProfile::BalancePerformance => "balance_performance",
            EppProfile::Default => "default",
            EppProfile::BalancePower => "balance_power",
            EppProfile::Power => "power",
        }
    }
}

pub struct RyzenEPPManager;

impl RyzenEPPManager {
    /// Applies the requested EPP profile to all detected CPU cores.
    /// If the profile is invalid, it logs an error and ignores the request.
    pub fn set_epp(mode: &str) -> Result<()> {
        let profile = match EppProfile::from_str(mode) {
            Ok(p) => p,
            Err(_) => {
                error!("Invalid EPP profile requested: '{}'. Ignoring.", mode);
                return Ok(());
            }
        };

        let profile_str = profile.as_str();
        info!("Applying AMD EPP profile: {}", profile_str);

        let cpu_dir = Path::new("/sys/devices/system/cpu");
        if !cpu_dir.exists() {
            warn!("CPU directory not found (is this Linux?). Skipping EPP tuning.");
            return Ok(());
        }

        let entries = match fs::read_dir(cpu_dir) {
            Ok(entries) => entries,
            Err(e) => {
                error!("Failed to read CPU directory: {}", e);
                return Ok(());
            }
        };

        let mut success_count = 0;
        let mut fail_count = 0;

        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                // Check if it's cpu0, cpu1, etc.
                if file_name.starts_with("cpu")
                    && file_name["cpu".len()..].chars().all(|c| c.is_ascii_digit())
                {
                    let epp_path = path.join("cpufreq/energy_performance_preference");
                    if epp_path.exists() {
                        if let Err(e) = fs::write(&epp_path, profile_str) {
                            debug!("Failed to write EPP to {}: {}", epp_path.display(), e);
                            fail_count += 1;
                        } else {
                            success_count += 1;
                        }
                    }
                }
            }
        }

        if success_count > 0 {
            info!("Successfully applied EPP to {} cores", success_count);
        } else if fail_count > 0 {
            warn!("Failed to apply EPP to any core (permission denied or unsupported hardware?)");
        } else {
            debug!("No EPP control files found. Not an AMD CPU or `amd_pstate` driver not loaded?");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epp_parsing() {
        assert_eq!(
            EppProfile::from_str("performance"),
            Ok(EppProfile::Performance)
        );
        assert_eq!(
            EppProfile::from_str("balance_performance"),
            Ok(EppProfile::BalancePerformance)
        );
        assert_eq!(EppProfile::from_str("default"), Ok(EppProfile::Default));
        assert_eq!(
            EppProfile::from_str("balance_power"),
            Ok(EppProfile::BalancePower)
        );
        assert_eq!(EppProfile::from_str("power"), Ok(EppProfile::Power));

        // Test case insensitivity
        assert_eq!(
            EppProfile::from_str("PERFORMANCE"),
            Ok(EppProfile::Performance)
        );

        // Test invalid
        assert_eq!(EppProfile::from_str("invalid_mode"), Err(()));
    }

    #[test]
    fn test_epp_as_str() {
        assert_eq!(EppProfile::Performance.as_str(), "performance");
        assert_eq!(
            EppProfile::BalancePerformance.as_str(),
            "balance_performance"
        );
    }
}
