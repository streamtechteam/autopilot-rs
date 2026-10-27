use super::*;

#[test]
fn constructors_choose_name_or_mac_matching() {
    let by_name = BluetoothCondition::new("Headphones".to_string());
    let by_mac = BluetoothCondition::with_mac("AA:BB:CC:DD:EE:FF".to_string());
    let defaulted = BluetoothCondition::from_scheme(BluetoothConditionScheme {
        device: "Keyboard".to_string(),
        match_by_mac: None,
    });

    assert_eq!(by_name.device, "Headphones");
    assert!(!by_name.match_by_mac);
    assert!(by_mac.match_by_mac);
    assert_eq!(defaulted.device, "Keyboard");
    assert!(!defaulted.match_by_mac);
    assert_eq!(defaulted.name(), "Bluetooth");
    assert_eq!(defaulted.clone_box().name(), "Bluetooth");
}

#[cfg(target_os = "linux")]
#[test]
fn parses_bluetoothctl_output() {
    let output = "Device AA:BB:CC:DD:EE:FF Headphones\nDevice 11:22:33:44:55:66 Keyboard";

    assert!(check_bluetooth_linux(output, "AA:BB:CC:DD:EE:FF", true));
    assert!(check_bluetooth_linux(output, "Keyboard", false));
    assert!(!check_bluetooth_linux(output, "Mouse", false));
}
