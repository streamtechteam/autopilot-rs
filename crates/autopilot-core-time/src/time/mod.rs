use chrono::{DateTime, Local, NaiveDate, NaiveTime, ParseError, TimeZone, Timelike};
use log::error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use autopilot_core_error::AutoPilotError;

pub mod init;
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[serde(tag = "type", content = "trigger", rename_all = "lowercase")]
#[schemars(title = "Schedule")]
pub enum When {
    Once(DateTimeScheme),
    Daily(TimeScheme),
    Weekly(WeeklyScheme),
    Monthly(MonthlyScheme),
    Yearly(YearlyScheme),
    Cron(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DateTimeScheme {
    pub date: String,
    pub time: String,
}

impl DateTimeScheme {
    pub fn parse(&self) -> Result<DateTime<Local>, AutoPilotError> {
        let time = NaiveTime::parse_from_str(&self.time, "%H:%M").map_err(|error| {
            error!("Invalid time format. Expected HH:MM: {error}");
            AutoPilotError::Time(format!("Invalid time format. Expected HH:MM: {error}"))
        })?;
        let date = NaiveDate::parse_from_str(&self.date, "%Y/%m/%d").map_err(|error| {
            error!("Invalid date format. Expected YYYY/MM/DD: {error}");
            AutoPilotError::Time(format!("Invalid date format. Expected YYYY/MM/DD: {error}"))
        })?;
        let naive_dt = date.and_time(time);

        Local
            .from_local_datetime(&naive_dt)
            .single()
            .ok_or_else(|| {
                AutoPilotError::Time(format!(
                    "Ambiguous or invalid local datetime: {} {}",
                    self.date, self.time
                ))
            })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TimeScheme {
    pub time: String,
}

impl TimeScheme {
    pub fn parse(&self) -> Result<NaiveTime, ParseError> {
        match NaiveTime::parse_from_str(&self.time, "%H:%M") {
            Ok(value) => Ok(value),
            Err(e) => {
                error!("Invalid time format. Expected HH:MM: {}", e);
                Err(e)
            }
        }
    }
}

fn parse_cron_time(time: &str) -> Result<NaiveTime, AutoPilotError> {
    NaiveTime::parse_from_str(time, "%H:%M").map_err(|err| {
        error!("Invalid time format. Expected HH:MM: {err}");
        AutoPilotError::Time(format!("Invalid time format. Expected HH:MM: {err}"))
    })
}

fn default_weekday() -> u8 {
    1
}

fn default_day() -> u8 {
    1
}

fn default_month() -> u8 {
    1
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "Weekly schedule")]
pub struct WeeklyScheme {
    pub time: String,
    /// Day of week: 0 or 7 is Sunday, 1 is Monday, ..., 6 is Saturday.
    /// Defaults to Monday.
    #[serde(default = "default_weekday")]
    #[schemars(range(min = 0, max = 7))]
    pub weekday: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "Monthly schedule")]
pub struct MonthlyScheme {
    pub time: String,
    /// Day of month, from 1 to 31. Defaults to the first day.
    #[serde(default = "default_day")]
    #[schemars(range(min = 1, max = 31))]
    pub day: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "Yearly schedule")]
pub struct YearlyScheme {
    pub time: String,
    /// Month, from 1 to 12. Defaults to January.
    #[serde(default = "default_month")]
    #[schemars(range(min = 1, max = 12))]
    pub month: u8,
    /// Day of month, from 1 to 31. Defaults to the first day.
    #[serde(default = "default_day")]
    #[schemars(range(min = 1, max = 31))]
    pub day: u8,
}

pub fn to_cron_expression(when: When) -> Result<String, AutoPilotError> {
    match when {
        When::Daily(time_scheme) => {
            let time = parse_cron_time(&time_scheme.time)?;
            // Run every day at the specified time.
            // Format: "second minute hour * * *"
            Ok(format!("0 {} {} * * *", time.minute(), time.hour()))
        }
        When::Weekly(scheme) => {
            let time = parse_cron_time(&scheme.time)?;
            let weekday = match scheme.weekday {
                0..=6 => scheme.weekday,
                7 => 0,
                value => {
                    return Err(AutoPilotError::Time(format!(
                        "Weekday must be between 0 and 7, got {value}"
                    )));
                }
            };
            // Format: "second minute hour * * weekday"
            Ok(format!(
                "0 {} {} * * {}",
                time.minute(),
                time.hour(),
                weekday
            ))
        }
        When::Monthly(scheme) => {
            let time = parse_cron_time(&scheme.time)?;
            if !(1..=31).contains(&scheme.day) {
                return Err(AutoPilotError::Time(format!(
                    "Day of month must be between 1 and 31, got {}",
                    scheme.day
                )));
            }
            // Format: "second minute hour day * *"
            Ok(format!(
                "0 {} {} {} * *",
                time.minute(),
                time.hour(),
                scheme.day
            ))
        }
        When::Yearly(scheme) => {
            let time = parse_cron_time(&scheme.time)?;
            if !(1..=12).contains(&scheme.month) {
                return Err(AutoPilotError::Time(format!(
                    "Month must be between 1 and 12, got {}",
                    scheme.month
                )));
            }
            if !(1..=31).contains(&scheme.day) {
                return Err(AutoPilotError::Time(format!(
                    "Day of month must be between 1 and 31, got {}",
                    scheme.day
                )));
            }
            // Format: "second minute hour day month *"
            Ok(format!(
                "0 {} {} {} {} *",
                time.minute(),
                time.hour(),
                scheme.day,
                scheme.month
            ))
        }
        When::Once(_) | When::Cron(_) => Ok("".to_string()),
    }
}

#[cfg(test)]
#[path = "time.spec.rs"]
mod spec;
