use crate::{
    conditions::{Condition, ConditionScheme},
    error::AutoPilotError,
};
use dialoguer::{Input, theme::ColorfulTheme};
use duct::cmd;
use serde::{Deserialize, Serialize};

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
            duct_sh::sh_dangerous(
                vec!["ping", "-n", "1", "-w", &ms_timeout, &self.host]
                    .join(" ")
                    .to_string(),
            )
            .run()
            .is_ok()
        }

        #[cfg(target_os = "macos")]
        {
            // macOS ping: -c is count, -t is timeout in seconds
            duct_sh::sh_dangerous(
                vec!["ping", "-c", "1", "-t", &timeout_str, &self.host]
                    .join(" ")
                    .to_string(),
            )
            .run()
            .is_ok()
        }

        #[cfg(target_os = "linux")]
        {
            // Linux ping: -c is count, -W is timeout in seconds
            duct_sh::sh_dangerous(
                vec!["ping", "-c", "1", "-W", &timeout_str, &self.host]
                    .join(" ")
                    .to_string(),
            )
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

    fn name(&self) -> &str {
        "Internet"
    }

    fn create(&self) -> Result<ConditionScheme, AutoPilotError> {
        let host: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter host to ping (default: 8.8.8.8):")
            .allow_empty(true)
            .interact_text()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;

        let host = if host.is_empty() {
            "8.8.8.8".to_string()
        } else {
            host
        };

        let timeout_input: u64 = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter timeout in seconds (default: 2):")
            .allow_empty(true)
            .interact_text()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;

        let timeout = if timeout_input == 0 { 2 } else { timeout_input };

        Ok(ConditionScheme::Internet(InternetConditionScheme {
            host: Some(host),
            timeout: Some(timeout),
        }))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct InternetConditionScheme {
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub timeout: Option<u64>,
}
