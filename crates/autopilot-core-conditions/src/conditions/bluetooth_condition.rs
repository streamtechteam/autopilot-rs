#[cfg(any(target_os = "linux", target_os = "macos"))]
use duct::cmd;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::conditions::Condition;

/// PowerShell query for a connected device.
///
/// The device name is supplied via `AUTOPILOT_BLUETOOTH_DEVICE`, never
/// interpolated into this script.
#[cfg(target_os = "windows")]
const WINDOWS_BLUETOOTH_SCRIPT: &str = r#"Get-CimInstance -Class Win32_PnPDevice | Where-Object { $_.Name -and $_.Name.IndexOf($env:AUTOPILOT_BLUETOOTH_DEVICE, [System.StringComparison]::OrdinalIgnoreCase) -ge 0 -and $_.Status -eq 'OK' }"#;

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
}

/// Check if a Bluetooth device is connected (synchronously)
pub fn sync_condition(device: &str, _match_by_mac: bool) -> bool {
    #[cfg(target_os = "linux")]
    {
        // Try bluetoothctl first (most common)
        if let Ok(output) = cmd("bluetoothctl", vec!["devices", "Connected"]).read() {
            return check_bluetooth_linux(&output, device, _match_by_mac);
        }

        // Fallback to hcitool if bluetoothctl fails
        if let Ok(output) = cmd("hcitool", vec!["con"]).read() {
            return output.contains(device);
        }

        false
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: only report devices whose own `Connected` field is `Yes`.
        // The defaults plist also lists paired-but-disconnected devices.
        if let Ok(output) = cmd("system_profiler", vec!["SPBluetoothDataType"]).read() {
            return check_bluetooth_macos(&output, device, _match_by_mac);
        }
        false
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: pass the device through the environment so PowerShell
        // treats it as data instead of interpolating it into the script.
        if let Ok(output) = duct::cmd(
            "powershell",
            [
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                WINDOWS_BLUETOOTH_SCRIPT,
            ],
        )
        .env("AUTOPILOT_BLUETOOTH_DEVICE", device)
        .read()
        {
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
#[cfg(target_os = "linux")]
fn check_bluetooth_linux(output: &str, target_device: &str, match_by_mac: bool) -> bool {
    for line in output.lines() {
        let Some(rest) = line.trim().strip_prefix("Device ") else {
            continue;
        };
        let Some((mac, name)) = rest.split_once(' ') else {
            continue;
        };

        let matched = if match_by_mac {
            mac.eq_ignore_ascii_case(target_device)
        } else {
            name.trim().eq_ignore_ascii_case(target_device)
        };

        if matched {
            return true;
        }
    }
    false
}

/// Parse macOS `system_profiler SPBluetoothDataType` output.
#[cfg(any(target_os = "macos", test))]
fn check_bluetooth_macos(output: &str, target_device: &str, match_by_mac: bool) -> bool {
    let mut name: Option<String> = None;
    let mut address: Option<String> = None;

    for line in output.lines() {
        let trimmed = line.trim();
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();

        if value.is_empty() {
            // A device header, or a section header followed by the device header.
            name = Some(key.to_string());
            address = None;
            continue;
        }

        if key.eq_ignore_ascii_case("Address") {
            address = Some(value.to_string());
        } else if key.eq_ignore_ascii_case("Connected") {
            let matched = if match_by_mac {
                address
                    .as_deref()
                    .is_some_and(|address| address.eq_ignore_ascii_case(target_device))
            } else {
                name.as_deref()
                    .is_some_and(|name| name.eq_ignore_ascii_case(target_device))
            };

            if matched && value.eq_ignore_ascii_case("Yes") {
                return true;
            }

            name = None;
            address = None;
        }
    }

    false
}

/// Scheme for deserializing BluetoothCondition from JSON/JSONC
#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "Bluetooth condition")]
pub struct BluetoothConditionScheme {
    /// The Bluetooth device address or name to match
    #[serde(default)]
    #[schemars(
        title = "Device",
        extend(
            "x-autopilot" = {
                "required": true,
                "widget": "dynamic-select",
                "provider": "bluetooth.devices",
                "allow_custom": true
            }
        )
    )]
    pub device: String,
    /// Match by MAC address (true) or device name (false). Defaults to false.
    #[serde(default)]
    #[schemars(title = "Match by MAC address")]
    #[schemars(extend("default" = false))]
    pub match_by_mac: Option<bool>,
}

#[cfg(test)]
#[path = "bluetooth_condition.spec.rs"]
mod spec;
