use duct::cmd;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::conditions::Condition;

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
}

/// Check if the current WiFi network matches the target SSID (synchronously)
pub fn sync_condition(target_ssid: &str) -> bool {
    #[cfg(target_os = "linux")]
    {
        // Try nmcli (NetworkManager) first - most common on Linux
        if let Ok(output) = cmd("nmcli", vec!["-t", "-f", "active,ssid", "dev", "wifi"]).read() {
            let connected = connected_wifi_linux(&output);
            if !connected.is_empty() {
                return connected.iter().any(|ssid| ssid == target_ssid);
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

        if let Some(ssid) = _ssid
            && ssid == target_ssid
        {
            return true;
        }

        // Fallback for other interfaces if en0 fails
        if let Ok(output) = cmd("/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport", vec!["-I"])
            .read()
            && let Some(ssid) = get_connected_wifi_macos_airport(&output)
        {
            return ssid == target_ssid;
        }
        false
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: use netsh. Only the interfaces view reports the current
        // connection; `netsh wlan show profiles` lists saved networks and
        // must not be used as a connected check.
        if let Ok(output) = cmd("netsh", vec!["wlan", "show", "interfaces"]).read()
            && let Some(ssid) = get_connected_wifi_windows(&output)
        {
            return ssid == target_ssid;
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

/// Parse SSIDs of active networks from nmcli output (Linux with NetworkManager).
///
/// `nmcli -t` escapes `:` and `\` with a backslash, so field splitting must
/// honor escapes. Multiple active interfaces are all returned.
#[cfg(target_os = "linux")]
fn connected_wifi_linux(output: &str) -> Vec<String> {
    fn split_fields(line: &str) -> Vec<String> {
        let mut fields = Vec::new();
        let mut current = String::new();
        let mut escaped = false;

        for character in line.chars() {
            if escaped {
                current.push(character);
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == ':' {
                fields.push(std::mem::take(&mut current));
            } else {
                current.push(character);
            }
        }

        if escaped {
            current.push('\\');
        }
        fields.push(current);
        fields
    }

    output
        .lines()
        .filter_map(|line| {
            let fields = split_fields(line);
            if fields.len() >= 2 && fields[0].trim() == "yes" {
                let ssid = fields[1].trim();
                if !ssid.is_empty() {
                    return Some(ssid.to_string());
                }
            }
            None
        })
        .collect()
}

/// Parse SSID from networksetup output (macOS)
#[cfg(target_os = "macos")]
fn get_connected_wifi_macos(output: &str) -> Option<String> {
    // Output format: "Current Wi-Fi Network: SSID_NAME"
    if let Some(pos) = output.find(": ") {
        return Some(output[pos + 2..].trim().to_string());
    }
    None
}

/// Parse SSID from airport -I output (macOS)
#[cfg(target_os = "macos")]
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
///
/// Only the exact `SSID` field is accepted. `BSSID` is a different value and
/// must not be mistaken for the network name.
#[cfg(any(target_os = "windows", test))]
fn get_connected_wifi_windows(output: &str) -> Option<String> {
    for line in output.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case("SSID") {
            let ssid = value.trim();
            if !ssid.is_empty() {
                return Some(ssid.to_string());
            }
        }
    }
    None
}

/// Scheme for deserializing WifiCondition from JSON/JSONC
#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "WiFi condition")]
pub struct WifiConditionScheme {
    /// The SSID (network name) to match
    #[serde(default)]
    #[schemars(
        title = "WiFi network",
        extend(
            "x-autopilot" = {
                "required": true,
                "widget": "dynamic-select",
                "provider": "wifi.networks",
                "allow_custom": true
            }
        )
    )]
    pub ssid: String,
}

#[cfg(test)]
#[path = "wifi_condition.spec.rs"]
mod spec;
