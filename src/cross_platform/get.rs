pub fn get_shell_name() -> String {
    #[cfg(target_os = "windows")]
    {
        "powershell".to_string()
    }

    #[cfg(target_os = "linux")]
    {
        "sh".to_string()
    }

    #[cfg(target_os = "macos")]
    {
        "zsh".to_string()
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        "sh".to_string();
    }
}
