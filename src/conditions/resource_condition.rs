use crate::conditions::Condition;
use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, System};

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
        let mut sys = System::new_with_specifics(
            sysinfo::RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );

        // Wait a bit to get a measurement for CPU if needed
        if self.resource_type == "cpu" {
            std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
            sys.refresh_cpu_all();
        }

        let current_value = match self.resource_type.as_str() {
            "cpu" => sys.global_cpu_usage(),
            "memory" | "ram" => {
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
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ResourceConditionScheme {
    #[serde(default)]
    pub resource_type: String,
    #[serde(default)]
    pub threshold: f32,
    #[serde(default)]
    pub operator: Option<String>,
}
