use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone};
use serde::{Deserialize, Serialize};

use crate::conditions::Condition;

/// Represents a time-based condition that checks if the current time matches a specified datetime
#[derive(Clone)]
pub struct TimeCondition {
    /// The target datetime when this condition should be true
    pub target_time: DateTime<Local>,
    /// Optional tolerance in seconds for fuzzy matching (e.g., within 30 seconds of target time)
    pub tolerance_seconds: Option<u64>,
}

impl TimeCondition {
    /// Create a new TimeCondition with exact time matching
    pub fn new(target_time: DateTime<Local>) -> Self {
        TimeCondition {
            target_time,
            tolerance_seconds: None,
        }
    }

    /// Create a new TimeCondition with a tolerance window
    pub fn with_tolerance(target_time: DateTime<Local>, tolerance_seconds: u64) -> Self {
        TimeCondition {
            target_time,
            tolerance_seconds: Some(tolerance_seconds),
        }
    }

    /// Create TimeCondition from a scheme (used for deserialization)
    pub fn from_scheme(scheme: TimeConditionScheme) -> Result<Self, String> {
        let target_time = Self::parse_datetime(&scheme.date, &scheme.time)?;
        
        Ok(TimeCondition {
            target_time,
            tolerance_seconds: scheme.tolerance_seconds,
        })
    }

    /// Parse date and time strings into a DateTime<Local>
    fn parse_datetime(date_str: &str, time_str: &str) -> Result<DateTime<Local>, String> {
        // Parse time in format HH:MM:SS
        let time = NaiveTime::parse_from_str(time_str, "%H:%M:%S")
            .map_err(|e| format!("Invalid time format. Expected HH:MM:SS: {}", e))?;

        // Parse date in format YYYY/MM/DD
        let date = NaiveDate::parse_from_str(date_str, "%Y/%m/%d")
            .map_err(|e| format!("Invalid date format. Expected YYYY/MM/DD: {}", e))?;

        // Combine date and time into a naive datetime
        let naive_dt = date.and_time(time);

        // Convert to local timezone
        let local_dt = Local
            .from_local_datetime(&naive_dt)
            .single()
            .ok_or_else(|| "Ambiguous or invalid local datetime".to_string())?;

        Ok(local_dt)
    }

    /// Check if the current time is within the tolerance window of the target time
    fn is_within_tolerance(&self) -> bool {
        let current_time = Local::now();
        let time_diff = (current_time - self.target_time).num_seconds().abs();

        match self.tolerance_seconds {
            Some(tolerance) => time_diff <= tolerance as i64,
            None => time_diff == 0, // Exact match
        }
    }

    /// Check if the target time has passed
    fn has_passed(&self) -> bool {
        Local::now() > self.target_time
    }
}

impl Condition for TimeCondition {
    fn check(&self) -> bool {
        sync_condition(self)
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }
}

/// Check if the current time matches the target time (synchronously)
pub fn sync_condition(condition: &TimeCondition) -> bool {
    condition.is_within_tolerance()
}

/// Check if the current time matches the target time (asynchronously)
/// This is mainly for API consistency; in practice it's the same as sync_condition
pub async fn async_condition(condition: &TimeCondition) -> bool {
    condition.is_within_tolerance()
}

/// Scheme for deserializing TimeCondition from JSON/JSONC
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeConditionScheme {
    /// Date in format YYYY/MM/DD
    pub date: String,
    /// Time in format HH:MM:SS
    pub time: String,
    /// Optional tolerance in seconds for fuzzy matching
    #[serde(default)]
    pub tolerance_seconds: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_datetime() {
        let result = TimeCondition::parse_datetime("2026/02/02", "11:40:00");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_date_format() {
        let result = TimeCondition::parse_datetime("02-02-2026", "11:40:00");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_time_format() {
        let result = TimeCondition::parse_datetime("2026/02/02", "11:40");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_scheme() {
        let scheme = TimeConditionScheme {
            date: "2026/02/02".to_string(),
            time: "11:40:00".to_string(),
            tolerance_seconds: Some(60),
        };
        let result = TimeCondition::from_scheme(scheme);
        assert!(result.is_ok());
        let condition = result.unwrap();
        assert_eq!(condition.tolerance_seconds, Some(60));
    }
}
