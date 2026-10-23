use super::*;

#[test]
fn returns_the_platform_shell() {
    #[cfg(target_os = "linux")]
    assert_eq!(get_shell_name(), "sh");

    #[cfg(target_os = "macos")]
    assert_eq!(get_shell_name(), "zsh");

    #[cfg(target_os = "windows")]
    assert_eq!(get_shell_name(), "powershell");
}

#[test]
fn supported_editors_are_a_subset_of_known_editors() {
    let supported = get_supported_editors();

    assert!(
        supported
            .iter()
            .all(|editor| TERMINAL_EDITORS.contains(editor))
    );
    assert!(TERMINAL_EDITORS.contains(&"vi"));
}
