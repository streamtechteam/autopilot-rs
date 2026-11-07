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

#[test]
fn the_root_mount_does_not_match_arbitrary_paths() {
    assert!(!mount_matches("/", "/media/user/USB"));
    assert!(!mount_matches("/", "/home/user"));
}

#[test]
fn exact_mount_points_still_match() {
    assert!(mount_matches("/", "/"));
    assert!(mount_matches("/media/user/USB", "/media/user/USB"));
    assert!(mount_matches("/media/user/USB/", "/media/user/USB"));
    assert!(!mount_matches("/media/user/USB2", "/media/user/USB"));
    assert!(!mount_matches("", "/media/user/USB"));
    assert!(!mount_matches("/media/user/USB", ""));
}

#[test]
fn names_match_one_way_and_ignore_empty_values() {
    assert!(name_matches("my usb drive", "usb"));
    assert!(name_matches("my usb drive", "my usb drive"));
    assert!(!name_matches("", "usb"));
    assert!(!name_matches("my usb drive", ""));
    assert!(!name_matches("sda1", "/dev/sda1"));
}
