use dialoguer::{Input, theme::ColorfulTheme};
use duct::cmd;
use serde::{Deserialize, Serialize};

use crate::{
    conditions::{Condition, ConditionScheme},
    error::AutoPilotError,
};

/// Represents a WiFi network condition that checks if a specific network is connected
#[derive(Clone)]
pub struct WifiCondition {
    /// The SSID (network name) to check for
    pub ssid: String,
}

impl WifiCondition {
    /// Create a new WiFi condition for a specific SSID
    pub fn new(ssid: String) -> Self {
        WifiCondition { ssid }
    }

    /// Create from a scheme (used for deserialization)
    pub fn from_scheme(scheme: WifiConditionScheme) -> Self {
        Self { ssid: scheme.ssid }
    }
}

impl Condition for WifiCondition {
    fn check(&self) -> bool {
        sync_condition(&self.ssid)
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }

    fn name(&self) -> &str {
        "WiFi"
    }

    fn create(&self) -> Result<ConditionScheme, AutoPilotError> {
        let ssid = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter WiFi network name (SSID):")
            .interact_text()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;

        Ok(ConditionScheme::Wifi(WifiConditionScheme { ssid }))
    }
}

/// Check if the current WiFi network matches the target SSID (synchronously)
pub fn sync_condition(target_ssid: &str) -> bool {
    #[cfg(target_os = "linux")]
    {
        // Try nmcli (NetworkManager) first - most common on Linux
        if let Ok(output) = cmd("nmcli", vec!["-t", "-f", "active,ssid", "dev", "wifi"]).read() {
            if let Ok(ssid) = get_connected_wifi_linux(&output) {
                return ssid == target_ssid;
            }
        }

        // Fallback to iwgetid if nmcli fails
        if let Ok(output) = cmd("iwgetid", vec!["-r"]).read() {
            return output.trim() == target_ssid;
        }

        false
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: use networksetup or ipconfig
        // networksetup is more reliable for SSID
        let _ssid =
            if let Ok(output) = cmd("networksetup", vec!["-getairportnetwork", "en0"]).read() {
                get_connected_wifi_macos(&output)
            } else {
                None
            };

        if let Some(ssid) = _ssid {
            if ssid == target_ssid {
                return true;
            }
        }

        // Fallback for other interfaces if en0 fails
        if let Ok(output) = cmd("/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport", vec!["-I"])
            .read()
        {
            if let Some(ssid) = get_connected_wifi_macos_airport(&output) {
                return ssid == target_ssid;
            }
        }
        false
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: use netsh
        if let Ok(output) = cmd("netsh", vec!["wlan", "show", "interfaces"]).read() {
            if let Some(ssid) = get_connected_wifi_windows(&output) {
                return ssid == target_ssid;
            }
        }

        // Fallback for Windows: try parsing netsh show current profile
        if let Ok(output) = cmd("netsh", vec!["wlan", "show", "profiles"]).read() {
            // This is less reliable but might help in some environments
            if output.contains(target_ssid) {
                return true;
            }
        }
        false
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        warn!("WiFi condition not supported on this platform");
        false
    }
}

/// Check if the current WiFi network matches the target SSID (asynchronously)
pub async fn async_condition(target_ssid: &str) -> bool {
    // For now, just call sync_condition since WiFi checks are typically fast
    // In a real async implementation, you'd use tokio::process::Command
    sync_condition(target_ssid)
}

/// Parse SSID from nmcli output (Linux with NetworkManager)
fn get_connected_wifi_linux(output: &str) -> Result<String, String> {
    for line in output.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 2 && parts[0].trim() == "yes" {
            return Ok(parts[1].trim().to_string());
        }
    }
    Err("No connected WiFi found".to_string())
}

/// Parse SSID from networksetup output (macOS)
fn get_connected_wifi_macos(output: &str) -> Option<String> {
    // Output format: "Current Wi-Fi Network: SSID_NAME"
    if let Some(pos) = output.find(": ") {
        return Some(output[pos + 2..].trim().to_string());
    }
    None
}

/// Parse SSID from airport -I output (macOS)
fn get_connected_wifi_macos_airport(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.trim().starts_with("SSID:") {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                return Some(parts[1].trim().to_string());
            }
        }
    }
    None
}

/// Parse SSID from netsh output (Windows)
fn get_connected_wifi_windows(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.contains("SSID") && line.contains(":") {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let ssid = parts[1].trim().to_string();
                if !ssid.is_empty() {
                    return Some(ssid);
                }
            }
        }
    }
    None
}

/// Scheme for deserializing WifiCondition from JSON/JSONC
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct WifiConditionScheme {
    /// The SSID (network name) to match
    #[serde(default)]
    pub ssid: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wifi_condition_creation() {
        let condition = WifiCondition::new("MyNetwork".to_string());
        assert_eq!(condition.ssid, "MyNetwork");
    }

    #[test]
    fn test_wifi_condition_from_scheme() {
        let scheme = WifiConditionScheme {
            ssid: "TestNetwork".to_string(),
        };
        let condition = WifiCondition::from_scheme(scheme);
        assert_eq!(condition.ssid, "TestNetwork");
    }
}
