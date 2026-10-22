use super::*;

#[test]
fn preserves_device_and_lowercases_identifier() {
    let by_name = ExternalDeviceCondition::new("USB STICK".to_string(), true);
    let by_mount = ExternalDeviceCondition::new("/MEDIA/USB".to_string(), false);
    let from_scheme = ExternalDeviceCondition::from_scheme(ExternalDeviceConditionScheme {
        device_identifier: "My Disk".to_string(),
        check_by_name: None,
    });

    assert_eq!(by_name.device_identifier, "usb stick");
    assert!(by_name.check_by_name);
    assert!(!by_mount.check_by_name);
    assert_eq!(from_scheme.device_identifier, "my disk");
    assert!(!from_scheme.check_by_name);
    assert_eq!(from_scheme.name(), "External Device");
    assert_eq!(from_scheme.clone_box().name(), "External Device");
}
