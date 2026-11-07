use super::*;

#[cfg(target_os = "linux")]
#[test]
fn parses_connected_networks_from_nmcli() {
    let output = "no:Other Network\nyes:Home WiFi\n";

    assert_eq!(connected_wifi_linux(output), vec!["Home WiFi".to_string()]);
    assert!(connected_wifi_linux("no:Other Network").is_empty());
}

#[cfg(target_os = "linux")]
#[test]
fn nmcli_parser_handles_escaped_colons_and_multiple_active_interfaces() {
    let output = "yes:Home\\:WiFi\nyes:Office\\:2\nno:Other\n";

    assert_eq!(
        connected_wifi_linux(output),
        vec!["Home:WiFi".to_string(), "Office:2".to_string()]
    );
}

#[test]
fn condition_keeps_ssid_and_reports_name() {
    let condition = WifiCondition::from_scheme(WifiConditionScheme {
        ssid: "Home WiFi".to_string(),
    });

    assert_eq!(condition.ssid, "Home WiFi");
    assert_eq!(condition.name(), "WiFi");
    assert_eq!(condition.clone_box().name(), "WiFi");
    assert!(!sync_condition(
        "autopilot-spec-network-that-is-not-connected"
    ));
}

#[cfg(any(target_os = "windows", test))]
#[test]
fn windows_parser_reads_the_connected_ssid() {
    let output = "\r\n    Name                   : Wi-Fi\r\n    State                  : connected\r\n    SSID                   : Home WiFi\r\n    BSSID                  : aa:bb:cc:dd:ee:ff\r\n";

    assert_eq!(
        get_connected_wifi_windows(output),
        Some("Home WiFi".to_string())
    );
}

#[cfg(any(target_os = "windows", test))]
#[test]
fn windows_parser_ignores_saved_profiles_and_bssid() {
    let profiles = "Profiles on interface Wi-Fi:\r\n    All User Profile     : Home WiFi\r\n";
    let bssid_only = "    BSSID                  : aa:bb:cc:dd:ee:ff\r\n";

    assert_eq!(get_connected_wifi_windows(profiles), None);
    assert_eq!(get_connected_wifi_windows(bssid_only), None);
}
