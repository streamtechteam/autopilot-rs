use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::conditions::Condition;

/// Represents a power/battery condition
#[derive(Clone)]
pub struct PowerCondition {
    /// Check if charging (true) or battery level (false)
    pub check_charging: bool,
    /// Target battery level (0-100) if check_charging is false
    pub threshold: Option<f32>,
    /// Comparison operator: "greater" or "less"
    pub operator: Option<String>,
    /// Per-condition polling interval override in milliseconds.
    /// `None` means the job-level `check_interval` applies.
    pub check_interval: Option<String>,
}

impl PowerCondition {
    pub fn is_charging() -> Self {
        Self {
            check_charging: true,
            threshold: None,
            operator: None,
            check_interval: None,
        }
    }

    pub fn battery_level(threshold: f32, operator: String) -> Self {
        Self {
            check_charging: false,
            threshold: Some(threshold),
            operator: Some(operator.to_lowercase()),
            check_interval: None,
        }
    }

    pub fn from_scheme(scheme: PowerConditionScheme) -> Self {
        Self {
            check_charging: scheme.check_charging.unwrap_or(false),
            threshold: scheme.threshold,
            operator: scheme.operator.map(|s| s.to_lowercase()),
            check_interval: scheme.check_interval,
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
                    && let Ok(val) = capacity.trim().parse::<f32>()
                {
                    return match self.operator.as_deref() {
                        Some("less") | Some("<") => val < threshold,
                        _ => val > threshold,
                    };
                }
                false
            } else {
                false
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = duct::cmd("pmset", vec!["-g", "batt"]).read() {
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
                let is_charging = if let Ok(output) = duct::cmd(
                    "powershell",
                    vec![
                        "-Command",
                        "(Get-CimInstance -ClassName Win32_Battery).BatteryStatus",
                    ],
                )
                .read()
                {
                    output.trim() == "2"
                } else {
                    false
                };

                if is_charging {
                    return true;
                }

                false
            } else if let Some(threshold) = self.threshold {
                let charge = if let Ok(output) = duct::cmd(
                    "powershell",
                    vec![
                        "-Command",
                        "(Get-CimInstance -ClassName Win32_Battery).EstimatedChargeRemaining",
                    ],
                )
                .read()
                {
                    output.trim().parse::<f32>().ok()
                } else {
                    None
                };

                if let Some(val) = charge {
                    return match self.operator.as_deref() {
                        Some("less") | Some("<") => val < threshold,
                        _ => val > threshold,
                    };
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
    fn check_interval(&self) -> Option<&str> {
        self.check_interval.as_deref()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "Power condition")]
pub struct PowerConditionScheme {
    #[serde(default)]
    #[schemars(title = "Check charging")]
    pub check_charging: Option<bool>,
    #[serde(default)]
    #[schemars(
        title = "Battery threshold",
        range(min = 0.0, max = 100.0),
        extend("x-autopilot" = {"unit": "%"})
    )]
    pub threshold: Option<f32>,
    #[serde(default)]
    #[schemars(
        title = "Comparison operator",
        extend(
            "x-autopilot" = {
                "widget": "select",
                "options": [
                    {"label": "Greater than (>)", "value": "greater"},
                    {"label": "Less than (<)", "value": "less"}
                ]
            }
        )
    )]
    pub operator: Option<String>,
    #[serde(default)]
    #[schemars(
        title = "Check interval",
        description = "Milliseconds between checks. Overrides the job-level check_interval."
    )]
    pub check_interval: Option<String>,
}

#[cfg(test)]
#[path = "power_condition.spec.rs"]
mod spec;
