use serde::{Deserialize, Serialize};
use crate::conditions::Condition;
use duct::cmd;
use log::warn;

/// Represents an internet reachability condition (ping based)
#[derive(Clone)]
pub struct InternetCondition {
    /// Host to ping (default: 8.8.8.8)
    pub host: String,
    /// Timeout in seconds (default: 2)
    pub timeout: u64,
}

impl InternetCondition {
    pub fn new(host: String, timeout: u64) -> Self {
        Self { host, timeout }
    }

    pub fn from_scheme(scheme: InternetConditionScheme) -> Self {
        Self {
            host: scheme.host.unwrap_or_else(|| "8.8.8.8".to_string()),
            timeout: scheme.timeout.unwrap_or(2),
        }
    }
}

impl Condition for InternetCondition {
    fn check(&self) -> bool {
        let timeout_str = self.timeout.to_string();
        
        #[cfg(target_os = "windows")]
        {
            // Windows ping: -n is count, -w is timeout in ms
            let ms_timeout = (self.timeout * 1000).to_string();
            cmd("ping", vec!["-n", "1", "-w", &ms_timeout, &self.host])
                .run()
                .is_ok()
        }

        #[cfg(target_os = "macos")]
        {
            // macOS ping: -c is count, -t is timeout in seconds
            cmd("ping", vec!["-c", "1", "-t", &timeout_str, &self.host])
                .run()
                .is_ok()
        }

        #[cfg(target_os = "linux")]
        {
            // Linux ping: -c is count, -W is timeout in seconds
            cmd("ping", vec!["-c", "1", "-W", &timeout_str, &self.host])
                .run()
                .is_ok()
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            warn!("Internet condition not supported on this platform");
            false
        }
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InternetConditionScheme {
    pub host: Option<String>,
    pub timeout: Option<u64>,
}
