use dialoguer::{Input, theme::ColorfulTheme};
use display_info::DisplayInfo;
use serde::{Deserialize, Serialize};

use crate::{
    conditions::{Condition, ConditionScheme},
    error::AutoPilotError,
};

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

    fn create(&self) -> Result<ConditionScheme, AutoPilotError> {
        let screen_count_str: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter target screen count (optional, but at least 1 check is required):")
            .allow_empty(true)
            .interact_text()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;

        let screen_count = if screen_count_str.is_empty() {
            None
        } else {
            match screen_count_str.parse::<u16>() {
                Ok(n) => Some(n),
                Err(_) => return Err(AutoPilotError::Condition("Invalid number".to_string())),
            }
        };

        let active_screen: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter target active screen name (optional):")
            .allow_empty(true)
            .interact_text()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;

        let screen_names_str: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter target screen names (comma separated, optional):")
            .allow_empty(true)
            .interact_text()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;

        let screen_names = if screen_names_str.is_empty() {
            None
        } else {
            Some(
                screen_names_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            )
        };

        if screen_count.is_none() && active_screen.is_empty() && screen_names.is_none() {
            return Err(AutoPilotError::Condition(
                "At least one screen condition check is required".to_string(),
            ));
        }

        Ok(ConditionScheme::Screen(ScreenConditionScheme {
            screen_count,
            active_screen_name: if active_screen.is_empty() {
                None
            } else {
                Some(active_screen)
            },
            screen_names,
        }))
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
            if let Some(count) = expected_count {
                if (displays.len() as u16) != count {
                    return false;
                }
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ScreenConditionScheme {
    pub screen_count: Option<u16>,
    pub active_screen_name: Option<String>,
    pub screen_names: Option<Vec<String>>,
}
