use std::collections::BTreeSet;

use crate::cross_platform::get::get_supported_editors;
use crate::form::FormError;

pub fn values(provider: &str) -> Result<Vec<String>, FormError> {
    match provider {
        "editors" => Ok(unique(get_supported_editors())),
        "processes" => Ok(processes()),
        "disks" => Ok(disks()),
        "screens" => Ok(screens()),
        "wifi.networks" => Ok(wifi_networks()),
        "bluetooth.devices" => Ok(bluetooth_devices()),
        other => Err(FormError::Unsupported(format!(
            "unknown form provider '{other}'"
        ))),
    }
}

fn unique(values: impl IntoIterator<Item = impl Into<String>>) -> Vec<String> {
    values
        .into_iter()
        .map(Into::into)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(target_os = "linux")]
fn processes() -> Vec<String> {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };

    unique(entries.flatten().filter_map(|entry| {
        let pid = entry.file_name().to_str()?.parse::<u32>().ok()?;
        let comm = std::fs::read_to_string(format!("/proc/{pid}/comm")).ok()?;
        Some(comm.trim_end().to_string())
    }))
}

#[cfg(not(target_os = "linux"))]
fn processes() -> Vec<String> {
    use sysinfo::{ProcessRefreshKind, RefreshKind, System};

    let system = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing()),
    );

    unique(
        system
            .processes()
            .values()
            .map(|process| process.name().to_string_lossy().into_owned()),
    )
}

fn disks() -> Vec<String> {
    use sysinfo::Disks;

    let mut values = Vec::new();
    for disk in Disks::new_with_refreshed_list().iter() {
        values.push(disk.mount_point().to_string_lossy().into_owned());
        values.push(disk.name().to_string_lossy().into_owned());
    }

    unique(values)
}

fn screens() -> Vec<String> {
    use display_info::DisplayInfo;

    let Ok(displays) = DisplayInfo::all() else {
        return Vec::new();
    };

    unique(displays.into_iter().map(|display| display.name))
}

#[cfg(target_os = "linux")]
fn wifi_networks() -> Vec<String> {
    let Ok(output) = duct_sh::sh_dangerous("nmcli -t -f SSID dev wifi").read() else {
        return Vec::new();
    };

    unique(
        output
            .lines()
            .map(|line| line.replace("\\:", ":"))
            .map(|line| line.trim().to_string()),
    )
}

#[cfg(target_os = "macos")]
fn wifi_networks() -> Vec<String> {
    for interface in ["en0", "en1"] {
        let command = format!("networksetup -listpreferredwirelessnetworks {interface}");
        let Ok(output) = duct_sh::sh_dangerous(command).read() else {
            continue;
        };

        let values = unique(output.lines().filter(|line| {
            !line.trim().is_empty() && !line.trim_start().starts_with("Preferred networks")
        }));
        if !values.is_empty() {
            return values;
        }
    }

    Vec::new()
}

#[cfg(target_os = "windows")]
fn wifi_networks() -> Vec<String> {
    let Ok(output) = duct_sh::sh_dangerous("netsh wlan show networks mode=bssid").read() else {
        return Vec::new();
    };

    unique(output.lines().filter_map(|line| {
        let line = line.trim();
        let (key, value) = line.split_once(':')?;
        key.trim()
            .strip_prefix("SSID")
            .filter(|suffix| suffix.trim().is_empty())
            .map(|_| value.trim().to_string())
    }))
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn wifi_networks() -> Vec<String> {
    Vec::new()
}

#[cfg(target_os = "linux")]
fn bluetooth_devices() -> Vec<String> {
    let Ok(output) = duct_sh::sh_dangerous("bluetoothctl devices").read() else {
        return Vec::new();
    };

    unique(output.lines().filter_map(|line| {
        let mut parts = line.splitn(3, ' ');
        let key = parts.next()?;
        if key != "Device" {
            return None;
        }
        let address = parts.next()?;
        let name = parts.next().unwrap_or_default().trim();
        Some(if name.is_empty() { address } else { name }.to_string())
    }))
}

#[cfg(target_os = "macos")]
fn bluetooth_devices() -> Vec<String> {
    let Ok(output) = duct_sh::sh_dangerous("blueutil --paired").read() else {
        return Vec::new();
    };

    unique(output.lines().filter_map(|line| {
        let (_, rest) = line.split_once("name: ")?;
        Some(rest.trim().trim_matches('"').to_string())
    }))
}

#[cfg(target_os = "windows")]
fn bluetooth_devices() -> Vec<String> {
    let Ok(output) = duct_sh::sh_dangerous(
        "powershell -NoProfile -Command \"Get-PnpDevice -Class Bluetooth | Select-Object -ExpandProperty FriendlyName\"",
    )
    .read()
    else {
        return Vec::new();
    };

    unique(output.lines())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn bluetooth_devices() -> Vec<String> {
    Vec::new()
}

#[cfg(test)]
#[path = "providers.spec.rs"]
mod spec;
