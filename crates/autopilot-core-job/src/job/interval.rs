use std::time::Duration;

use log::error;

use autopilot_core_conditions::Condition;

/// Fallback polling interval used when a configured value cannot be parsed.
pub const DEFAULT_CHECK_INTERVAL_MS: u64 = 1000;

/// Parse a `check_interval` value in milliseconds.
///
/// Invalid values log an error and fall back to [`DEFAULT_CHECK_INTERVAL_MS`],
/// matching the historical behaviour of the job-level interval.
pub fn parse_check_interval_ms(raw: &str) -> u64 {
    raw.parse::<u64>().unwrap_or_else(|_| {
        error!("check_interval value is not valid, using 1000ms as default");
        DEFAULT_CHECK_INTERVAL_MS
    })
}

/// Resolve the effective polling interval for a condition.
///
/// A per-condition override wins; otherwise the job-level interval applies.
/// `None` means the condition is checked once and never polled.
pub fn effective_check_interval(
    condition: &dyn Condition,
    global: Option<&str>,
) -> Option<Duration> {
    let raw = condition.check_interval().or(global)?;
    Some(Duration::from_millis(parse_check_interval_ms(raw)))
}

#[cfg(test)]
#[path = "interval.spec.rs"]
mod spec;
