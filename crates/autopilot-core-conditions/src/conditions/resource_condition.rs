use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, System};

use crate::conditions::Condition;

/// Represents a system resource condition (CPU or RAM)
#[derive(Clone)]
pub struct ResourceCondition {
    /// Type of resource to check: "cpu" or "memory"
    pub resource_type: String,
    /// Threshold percentage (0-100)
    pub threshold: f32,
    /// Comparison operator: "greater" or "less"
    pub operator: String,
}

impl ResourceCondition {
    pub fn new(resource_type: String, threshold: f32, operator: String) -> Self {
        Self {
            resource_type: resource_type.to_lowercase(),
            threshold,
            operator: operator.to_lowercase(),
        }
    }

    pub fn from_scheme(scheme: ResourceConditionScheme) -> Self {
        Self {
            resource_type: scheme.resource_type.to_lowercase(),
            threshold: scheme.threshold,
            operator: scheme
                .operator
                .unwrap_or_else(|| "greater".to_string())
                .to_lowercase(),
        }
    }
}

impl Condition for ResourceCondition {
    fn check(&self) -> bool {
        let current_value = match self.resource_type.as_str() {
            "cpu" => {
                let mut sys = System::new_with_specifics(
                    sysinfo::RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
                );

                // Wait a bit to get a measurement for CPU
                std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
                sys.refresh_cpu_all();

                sys.global_cpu_usage()
            }
            "memory" | "ram" => {
                let sys = System::new_with_specifics(
                    sysinfo::RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()),
                );

                let total = sys.total_memory();
                if total == 0 {
                    0.0
                } else {
                    (sys.used_memory() as f32 / total as f32) * 100.0
                }
            }
            _ => return false,
        };

        match self.operator.as_str() {
            "greater" | "gt" | ">" => current_value > self.threshold,
            "less" | "lt" | "<" => current_value < self.threshold,
            _ => false,
        }
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }

    fn name(&self) -> &str {
        "Resource"
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "Resource condition")]
pub struct ResourceConditionScheme {
    #[serde(default)]
    #[schemars(
        title = "Resource type",
        extend(
            "x-autopilot" = {
                "required": true,
                "widget": "select",
                "options": [
                    {"label": "CPU", "value": "cpu"},
                    {"label": "Memory (RAM)", "value": "memory"}
                ]
            }
        )
    )]
    pub resource_type: String,
    #[serde(default)]
    #[schemars(
        title = "Threshold",
        range(min = 0.0, max = 100.0),
        extend("x-autopilot" = {"required": true, "unit": "%"})
    )]
    pub threshold: f32,
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
}

#[cfg(test)]
#[path = "resource_condition.spec.rs"]
mod spec;
