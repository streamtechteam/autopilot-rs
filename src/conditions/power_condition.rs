use crate::{
    conditions::{Condition, ConditionScheme},
    error::AutoPilotError,
};
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use serde::{Deserialize, Serialize};

/// Represents a power/battery condition
#[derive(Clone)]
pub struct PowerCondition {
    /// Check if charging (true) or battery level (false)
    pub check_charging: bool,
    /// Target battery level (0-100) if check_charging is false
    pub threshold: Option<f32>,
    /// Comparison operator: "greater" or "less"
    pub operator: Option<String>,
}

impl PowerCondition {
    pub fn is_charging() -> Self {
        Self {
            check_charging: true,
            threshold: None,
            operator: None,
        }
    }

    pub fn battery_level(threshold: f32, operator: String) -> Self {
        Self {
            check_charging: false,
            threshold: Some(threshold),
            operator: Some(operator.to_lowercase()),
        }
    }

    pub fn from_scheme(scheme: PowerConditionScheme) -> Self {
        Self {
            check_charging: scheme.check_charging.unwrap_or(false),
            threshold: scheme.threshold,
            operator: scheme.operator.map(|s| s.to_lowercase()),
        }
    }
}

impl Condition for PowerCondition {
    fn check(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            if self.check_charging {
                // Try multiple common paths for AC adapter status
                let paths = [
                    "/sys/class/power_supply/AC/online",
                    "/sys/class/power_supply/ACAD/online",
                ];
                for path in paths {
                    if let Ok(status) = std::fs::read_to_string(path) {
                        return status.trim() == "1";
                    }
                }
                false
            } else if let Some(threshold) = self.threshold {
                if let Ok(capacity) =
                    std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
                {
                    if let Ok(val) = capacity.trim().parse::<f32>() {
                        return match self.operator.as_deref() {
                            Some("less") | Some("<") => val < threshold,
                            _ => val > threshold,
                        };
                    }
                }
                false
            } else {
                false
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = cmd("pmset", vec!["-g", "batt"]).read() {
                if self.check_charging {
                    return output.contains("AC Power");
                } else if let Some(threshold) = self.threshold {
                    // Output example: " -InternalBattery-0 (id=123)	100%; charged; 0:00 remaining"
                    if let Some(pct_pos) = output.find('%') {
                        let start = output[..pct_pos]
                            .rfind(|c: char| !c.is_numeric())
                            .unwrap_or(0);
                        if let Ok(val) = output[start..pct_pos].trim().parse::<f32>() {
                            return match self.operator.as_deref() {
                                Some("less") | Some("<") => val < threshold,
                                _ => val > threshold,
                            };
                        }
                    }
                }
            }
            false
        }

        #[cfg(target_os = "windows")]
        {
            if self.check_charging {
                // 2 = AC, 1 = Battery
                if let Ok(output) = cmd(
                    "powershell",
                    vec![
                        "-Command",
                        "(Get-CimInstance -ClassName Win32_Battery).BatteryStatus",
                    ],
                )
                .read()
                {
                    return output.trim() == "2";
                }
                false
            } else if let Some(threshold) = self.threshold {
                if let Ok(output) = cmd(
                    "powershell",
                    vec![
                        "-Command",
                        "(Get-CimInstance -ClassName Win32_Battery).EstimatedChargeRemaining",
                    ],
                )
                .read()
                {
                    if let Ok(val) = output.trim().parse::<f32>() {
                        return match self.operator.as_deref() {
                            Some("less") | Some("<") => val < threshold,
                            _ => val > threshold,
                        };
                    }
                }
                false
            } else {
                false
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            warn!("Power condition not supported on this platform");
            false
        }
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }

    fn name(&self) -> &str {
        "Power"
    }

    fn create(&self) -> Result<ConditionScheme, AutoPilotError> {
        let check_charging = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Check if charging? (Otherwise check battery level)")
            .interact_opt()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?
            .unwrap_or(true);

        if check_charging {
            Ok(ConditionScheme::Power(PowerConditionScheme {
                check_charging: Some(true),
                threshold: None,
                operator: None,
            }))
        } else {
            let threshold: f32 = Input::<f32>::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter battery threshold percentage (0-100):")
                .interact_text()
                .map_err(|err| AutoPilotError::Condition(err.to_string()))?;
            // .parse()
            // .map_err(|_| AutoPilotError::Condition("Invalid threshold value".to_string()))?;

            let operators = ["Greater than (>)", "Less than (<)"];
            let selected_op = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select comparison operator:")
                .items(&operators)
                .default(0)
                .interact_opt()
                .map_err(|err| AutoPilotError::Condition(err.to_string()))?
                .unwrap_or(0);

            let operator = if selected_op == 0 { "greater" } else { "less" }.to_string();

            Ok(ConditionScheme::Power(PowerConditionScheme {
                check_charging: Some(false),
                threshold: Some(threshold),
                operator: Some(operator),
            }))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct PowerConditionScheme {
    #[serde(default)]
    pub check_charging: Option<bool>,
    #[serde(default)]
    pub threshold: Option<f32>,
    #[serde(default)]
    pub operator: Option<String>,
}
