use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::cross_platform::TERMINAL_EDITORS;

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

pub fn get_supported_editors() -> Vec<&'static str> {
    TERMINAL_EDITORS
        .par_iter() // Parallel iteration
        .copied()
        .filter(|&editor| {
            #[cfg(target_os = "windows")]
            {
                match duct_sh::sh_dangerous(format!("where {}", editor))
                    .stdout_null()
                    .stderr_null()
                    .run()
                {
                    Ok(_) => return true,
                    Err(_) => return false,
                }
            }
            match duct_sh::sh_dangerous(format!("which {}", editor))
                .stdout_null()
                .stderr_null()
                .run()
            {
                Ok(_) => true,
                Err(_) => false,
            }
        })
        .collect()
}
