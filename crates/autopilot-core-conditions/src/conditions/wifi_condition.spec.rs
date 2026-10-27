use super::*;

#[cfg(target_os = "linux")]
#[test]
fn parses_connected_network_from_nmcli() {
    let output = "no:Other Network\nyes:Home WiFi\n";

    assert_eq!(
        get_connected_wifi_linux(output),
        Ok("Home WiFi".to_string())
    );
    assert!(get_connected_wifi_linux("no:Other Network").is_err());
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
