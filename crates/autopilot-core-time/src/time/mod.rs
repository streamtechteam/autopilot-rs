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
    Weekly(TimeScheme),
    Monthly(TimeScheme),
    Yearly(TimeScheme),
    Cron(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
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

pub fn to_cron_expression(when: When) -> Result<String, ParseError> {
    match when {
        When::Daily(time_scheme) => {
            let time = time_scheme.parse()?;
            // Run every day at the specified time
            // Format: "second minute hour * * *"
            Ok(format!("0 {} {} * * *", time.minute(), time.hour()))
        }
        When::Weekly(time_scheme) => {
            let time = time_scheme.parse()?;
            // Run every week on Monday at the specified time
            // Format: "second minute hour * * 1" (1 = Monday)
            Ok(format!("0 {} {} * * 1", time.minute(), time.hour()))
        }
        When::Monthly(time_scheme) => {
            let time = time_scheme.parse()?;
            // Run every month on the 1st at the specified time
            // Format: "second minute hour 1 * *"
            Ok(format!("0 {} {} 1 * *", time.minute(), time.hour()))
        }
        When::Yearly(time_scheme) => {
            let time = time_scheme.parse()?;
            // Run every year on January 1st at the specified time
            // Format: "second minute hour 1 1 *"
            Ok(format!("0 {} {} 1 1 *", time.minute(), time.hour()))
        }
        _ => Ok("".to_string()),
    }
}

#[cfg(test)]
#[path = "time.spec.rs"]
mod spec;
