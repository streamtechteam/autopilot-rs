use display_info::DisplayInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::conditions::Condition;
use autopilot_core_form::{ConditionConfig, FormError};

#[derive(Clone)]
pub struct ScreenCondition {
    pub screen_count: Option<u16>,
    pub active_screen_name: Option<String>,
    pub screen_names: Option<Vec<String>>,
}

impl ScreenCondition {
    pub fn new(
        screen_count: Option<u16>,
        active_screen_name: Option<String>,
        screen_names: Option<Vec<String>>,
    ) -> Self {
        ScreenCondition {
            screen_count,
            active_screen_name,
            screen_names,
        }
    }

    pub fn from_scheme(scheme: ScreenConditionScheme) -> Self {
        Self {
            screen_count: scheme.screen_count,
            active_screen_name: scheme.active_screen_name,
            screen_names: scheme.screen_names,
        }
    }
}

impl Condition for ScreenCondition {
    fn check(&self) -> bool {
        sync_condition(
            self.screen_count,
            &self.active_screen_name,
            &self.screen_names,
        )
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }

    fn name(&self) -> &str {
        "Screen"
    }
}

pub fn sync_condition(
    expected_count: Option<u16>,
    expected_active_name: &Option<String>,
    expected_names: &Option<Vec<String>>,
) -> bool {
    // Use display-info crate to get reliable, cross-platform display data
    // without spawning shell processes.
    match DisplayInfo::all() {
        Ok(displays) => {
            // Check count
            if let Some(count) = expected_count
                && (displays.len() as u16) != count
            {
                return false;
            }

            // Check active screen name (primary display)
            if let Some(target_active) = expected_active_name {
                let primary = displays.iter().find(|d| d.is_primary);
                match primary {
                    Some(p) => {
                        // Check exact name match or partial match
                        if !p.name.contains(target_active) {
                            return false;
                        }
                    }
                    None => return false, // No primary display found? Should not happen usually.
                }
            }

            // Check specific screen names presence
            if let Some(targets) = expected_names {
                if targets.is_empty() {
                    return true;
                }

                let current_names: Vec<String> = displays.iter().map(|d| d.name.clone()).collect();

                for target in targets {
                    let found = current_names.iter().any(|name| name.contains(target));
                    if !found {
                        return false;
                    }
                }
            }

            true
        }
        Err(_e) => {
            // Log error if possible, or return false
            // eprintln!("Failed to get display info: {}", e);
            false
        }
    }
}

pub async fn async_condition(
    expected_count: Option<u16>,
    expected_active_name: Option<String>,
    expected_names: Option<Vec<String>>,
) -> bool {
    // DisplayInfo::all() is synchronous but fast (native API calls)
    // Wrapping in spawn_blocking might be safer if it blocks, but usually it's instant.
    // For now, run directly.
    sync_condition(expected_count, &expected_active_name, &expected_names)
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "Screen condition")]
pub struct ScreenConditionScheme {
    #[serde(default)]
    #[schemars(title = "Screen count", range(min = 0))]
    pub screen_count: Option<u16>,
    #[serde(default)]
    #[schemars(
        title = "Active screen name",
        extend(
            "x-autopilot" = {
                "widget": "dynamic-select",
                "provider": "screens",
                "allow_custom": true
            }
        )
    )]
    pub active_screen_name: Option<String>,
    #[serde(default)]
    #[schemars(
        title = "Screen names",
        extend(
            "x-autopilot" = {
                "widget": "dynamic-select",
                "provider": "screens",
                "multiple": true,
                "allow_custom": true
            }
        )
    )]
    pub screen_names: Option<Vec<String>>,
}

impl ConditionConfig for ScreenConditionScheme {
    fn validate(&self) -> Result<(), FormError> {
        if self.screen_count.is_none()
            && self.active_screen_name.is_none()
            && self.screen_names.as_ref().is_none_or(Vec::is_empty)
        {
            return Err(FormError::InvalidValue {
                field: "screen_count".to_string(),
                message: "at least one screen check is required".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "screen_condition.spec.rs"]
mod spec;
