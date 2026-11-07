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
            use std::path::Path;

            let root = Path::new("/sys/class/power_supply");

            if self.check_charging {
                return any_ac_online(root);
            }

            if let Some(threshold) = self.threshold
                && let Some(val) = battery_capacity(root)
            {
                return match self.operator.as_deref() {
                    Some("less") | Some("<") => val < threshold,
                    _ => val > threshold,
                };
            }

            false
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
}

#[cfg(any(target_os = "linux", test))]
fn any_ac_online(root: &std::path::Path) -> bool {
    let Ok(entries) = std::fs::read_dir(root) else {
        return false;
    };

    entries.flatten().any(|entry| {
        std::fs::read_to_string(entry.path().join("online"))
            .is_ok_and(|status| status.trim() == "1")
    })
}

#[cfg(any(target_os = "linux", test))]
fn battery_capacity(root: &std::path::Path) -> Option<f32> {
    let mut entries: Vec<_> = std::fs::read_dir(root)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .collect();
    entries.sort();

    for path in entries {
        let Ok(kind) = std::fs::read_to_string(path.join("type")) else {
            continue;
        };
        if kind.trim() != "Battery" {
            continue;
        }
        if let Ok(capacity) = std::fs::read_to_string(path.join("capacity"))
            && let Ok(value) = capacity.trim().parse::<f32>()
        {
            return Some(value);
        }
    }

    None
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "Power condition")]
pub struct PowerConditionScheme {
    #[serde(default)]
    #[schemars(title = "Check charging", extend("x-autopilot" = {"group": "check"}))]
    #[schemars(extend("default" = false))]
    pub check_charging: Option<bool>,
    #[serde(default)]
    #[schemars(
        title = "Battery threshold",
        range(min = 0.0, max = 100.0),
        extend(
            "x-autopilot" = {
                "unit": "%",
                "group": "thresholds",
                "visible_when": {"field": "check_charging", "equals": false}
            }
        )
    )]
    pub threshold: Option<f32>,
    #[serde(default)]
    #[schemars(
        title = "Comparison operator",
        extend(
            "x-autopilot" = {
                "widget": "select",
                "group": "thresholds",
                "visible_when": {"field": "check_charging", "equals": false},
                "options": [
                    {"label": "Greater than (>)", "value": "greater"},
                    {"label": "Less than (<)", "value": "less"}
                ]
            }
        )
    )]
    #[schemars(extend("default" = "greater"))]
    pub operator: Option<String>,
}

#[cfg(test)]
#[path = "power_condition.spec.rs"]
mod spec;
