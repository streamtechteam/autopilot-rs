use dialoguer::{Confirm, Input, theme::ColorfulTheme};
use duct::cmd;
use serde::{Deserialize, Serialize};

use crate::{
    conditions::{Condition, ConditionScheme},
    error::AutoPilotError,
};

/// Represents a Bluetooth device condition that checks if a specific device is connected
#[derive(Clone)]
pub struct BluetoothCondition {
    /// The Bluetooth device address or name to check for
    pub device: String,
    /// Check by MAC address (true) or device name (false)
    pub match_by_mac: bool,
}

impl BluetoothCondition {
    /// Create a new Bluetooth condition for a specific device name
    pub fn new(device: String) -> Self {
        BluetoothCondition {
            device,
            match_by_mac: false,
        }
    }

    /// Create a new Bluetooth condition matching by MAC address
    pub fn with_mac(mac_address: String) -> Self {
        BluetoothCondition {
            device: mac_address,
            match_by_mac: true,
        }
    }

    /// Create from a scheme (used for deserialization)
    pub fn from_scheme(scheme: BluetoothConditionScheme) -> Self {
        Self {
            device: scheme.device,
            match_by_mac: scheme.match_by_mac.unwrap_or(false),
        }
    }
}

impl Condition for BluetoothCondition {
    fn check(&self) -> bool {
        sync_condition(&self.device, self.match_by_mac)
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }

    fn name(&self) -> &str {
        "Bluetooth"
    }

    fn create(&self) -> Result<ConditionScheme, AutoPilotError> {
        let device = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter device name :")
            .interact_text()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;

        let match_by_mac = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Match by MAC address?")
            .interact_opt()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;

        Ok(ConditionScheme::Bluetooth(BluetoothConditionScheme {
            device,
            match_by_mac,
        }))
    }
}

/// Check if a Bluetooth device is connected (synchronously)
pub fn sync_condition(device: &str, match_by_mac: bool) -> bool {
    #[cfg(target_os = "linux")]
    {
        // Try bluetoothctl first (most common)
        if let Ok(output) = cmd("bluetoothctl", vec!["devices", "Connected"]).read() {
            return check_bluetooth_linux(&output, device, match_by_mac);
        }

        // Fallback to hcitool if bluetoothctl fails
        if let Ok(output) = cmd("hcitool", vec!["con"]).read() {
            return output.contains(device);
        }

        false
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: Use system_profiler
        let connected =
            if let Ok(output) = cmd("system_profiler", vec!["SPBluetoothDataType"]).read() {
                output.contains(device)
            } else {
                false
            };

        if connected {
            return true;
        }

        // Fallback to defaults read
        if let Ok(output) = cmd(
            "defaults",
            vec!["read", "/Library/Preferences/com.apple.Bluetooth.plist"],
        )
        .read()
        {
            return output.contains(device);
        }
        false
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: Use PowerShell Get-CimInstance
        let ps_cmd = format!(
            "Get-CimInstance -Class Win32_PnPDevice | Where-Object {{$_.Name -like '*{}*' -and $_.Status -eq 'OK'}}",
            device
        );
        if let Ok(output) = cmd("powershell", vec!["-NoProfile", "-Command", &ps_cmd]).read() {
            return !output.is_empty();
        }
        false
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        warn!("Bluetooth condition not supported on this platform");
        false
    }
}

/// Check if a Bluetooth device is connected (asynchronously)
pub async fn async_condition(device: &str, match_by_mac: bool) -> bool {
    // For now, just call sync_condition since Bluetooth checks are typically fast
    // In a real async implementation, you'd use tokio::process::Command
    sync_condition(device, match_by_mac)
}

/// Parse Bluetooth devices from bluetoothctl output (Linux)
fn check_bluetooth_linux(output: &str, target_device: &str, match_by_mac: bool) -> bool {
    for line in output.lines() {
        let line = line.trim();

        if match_by_mac {
            // MAC address format: XX:XX:XX:XX:XX:XX
            if line.contains(target_device) {
                return true;
            }
        } else {
            // Device name matching - typically appears after MAC address
            if line.contains(target_device) {
                return true;
            }
        }
    }
    false
}

/// Scheme for deserializing BluetoothCondition from JSON/JSONC
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct BluetoothConditionScheme {
    /// The Bluetooth device address or name to match
    #[serde(default)]
    pub device: String,
    /// Match by MAC address (true) or device name (false). Defaults to false.
    #[serde(default)]
    pub match_by_mac: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bluetooth_condition_creation() {
        let condition = BluetoothCondition::new("MyHeadphones".to_string());
        assert_eq!(condition.device, "MyHeadphones");
        assert!(!condition.match_by_mac);
    }

    #[test]
    fn test_bluetooth_condition_with_mac() {
        let condition = BluetoothCondition::with_mac("AA:BB:CC:DD:EE:FF".to_string());
        assert_eq!(condition.device, "AA:BB:CC:DD:EE:FF");
        assert!(condition.match_by_mac);
    }

    #[test]
    fn test_bluetooth_condition_from_scheme() {
        let scheme = BluetoothConditionScheme {
            device: "TestDevice".to_string(),
            match_by_mac: Some(true),
        };
        let condition = BluetoothCondition::from_scheme(scheme);
        assert_eq!(condition.device, "TestDevice");
        assert!(condition.match_by_mac);
    }
}
