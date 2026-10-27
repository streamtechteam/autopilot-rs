use std::env;

use log::error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::conditions::Condition;
#[derive(Clone)]
pub struct DesktopEnvCondition {
    target: String,
    /// Per-condition polling interval override in milliseconds.
    /// `None` means the job-level `check_interval` applies.
    check_interval: Option<String>,
}
impl DesktopEnvCondition {
    pub fn new(target: String) -> Self {
        Self {
            target,
            check_interval: None,
        }
    }
    pub fn from_scheme(scheme: DesktopEnvConditionScheme) -> Self {
        Self {
            target: scheme.target,
            check_interval: scheme.check_interval,
        }
    }
}
impl Condition for DesktopEnvCondition {
    fn check(&self) -> bool {
        sync_condition(&self.target)
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }

    fn name(&self) -> &str {
        "Desktop Environment"
    }
    fn check_interval(&self) -> Option<&str> {
        self.check_interval.as_deref()
    }
}

pub fn sync_condition(target: &str) -> bool {
    match get_current_de() {
        Some(current_de) => current_de.to_lowercase() == target.to_lowercase(),
        None => {
            error!("Failed to detect Desktop Environment");
            false
        }
    }
}

pub async fn async_condition(target: &str) -> bool {
    sync_condition(target)
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "Desktop environment condition")]
pub struct DesktopEnvConditionScheme {
    #[serde(default)]
    #[schemars(
        title = "Desktop environment",
        description = "For example GNOME, KDE, or SWAY",
        extend("x-autopilot" = {"required": true})
    )]
    target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = "Check interval",
        description = "Milliseconds between checks. Overrides the job-level check_interval."
    )]
    check_interval: Option<String>,
}

/// Detects the current Desktop Environment using a robust fallback chain.
fn get_current_de() -> Option<String> {
    // 1. XDG_CURRENT_DESKTOP (Most standard on modern Linux)
    if let Ok(val) = env::var("XDG_CURRENT_DESKTOP")
        && !val.is_empty()
    {
        return Some(val);
    }

    // 2. DESKTOP_SESSION (Common fallback)
    if let Ok(val) = env::var("DESKTOP_SESSION")
        && !val.is_empty()
    {
        return Some(val);
    }

    // 3. GNOME Specific
    if env::var("GNOME_DESKTOP_SESSION_ID").is_ok() {
        return Some("GNOME".to_string());
    }

    // 4. KDE Specific
    if env::var("KDE_FULL_SESSION").is_ok() {
        return Some("KDE".to_string());
    }

    // 5. MATE Specific
    if env::var("MATE_DESKTOP_SESSION_ID").is_ok() {
        return Some("MATE".to_string());
    }

    // 6. Sway/Wayland Specific
    if env::var("SWAYSOCK").is_ok() {
        return Some("SWAY".to_string());
    }

    // 7. Generic Wayland
    if env::var("WAYLAND_DISPLAY").is_ok() {
        return Some("WAYLAND".to_string());
    }

    // 8. Windows / macOS Handling
    #[cfg(target_os = "windows")]
    {
        Some("WINDOWS".to_string())
    }

    #[cfg(target_os = "macos")]
    {
        Some("MACOS".to_string())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        None
    }
}

#[cfg(test)]
#[path = "desktop_env_condition.spec.rs"]
mod spec;
