
use chrono::{
    DateTime, Local, NaiveDate, NaiveTime, ParseError, TimeZone, Timelike,
};
use log::error;
use serde::{Deserialize, Serialize};


pub mod add;
pub mod init;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(tag = "type", content = "trigger", rename_all = "lowercase")]
pub enum When {
    Once(DateTimeScheme),
    Daily(TimeScheme),
    Weekly(TimeScheme),
    Monthly(TimeScheme),
    Yearly(TimeScheme),
    Cron(String),
}

// pub struct When {
//     type: WhenTypes,
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateTimeScheme {
    pub date: String,
    pub time: String,
}

impl DateTimeScheme {
    pub fn parse(&self) -> Result<DateTime<Local>, ParseError> {
        let date: NaiveDate;
        let time: NaiveTime;
        let naive_dt;
        let local_dt;
        match NaiveTime::parse_from_str(&self.time, "%H:%M") {
            Ok(value) => {
                time = value;
                match NaiveDate::parse_from_str(&self.date, "%Y/%m/%d") {
                    Ok(value) => {
                        date = value;
                        naive_dt = date.and_time(time);
                        local_dt = Local
                            .from_local_datetime(&naive_dt)
                            .single()
                            .ok_or("Ambiguous or invalid local datetime".to_string())
                            .expect("invalid local datetime");
                    }
                    Err(e) => {
                        error!("Invalid date format. Expected YYYY/MM/DD: {}", e);
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                error!("Invalid time format. Expected HH:MM: {}", e);
                return Err(e);
            }
        }

        Ok(local_dt)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
            // Format: "minute hour * * *"
            Ok(format!("{} {} * * *", time.minute(), time.hour()))
        }
        When::Weekly(time_scheme) => {
            let time = time_scheme.parse()?;
            // Run every week on Monday at the specified time
            // Format: "minute hour * * 1" (1 = Monday)
            Ok(format!("{} {} * * 1", time.minute(), time.hour()))
        }
        When::Monthly(time_scheme) => {
            let time = time_scheme.parse()?;
            // Run every month on the 1st at the specified time
            // Format: "minute hour 1 * *"
            Ok(format!("{} {} 1 * *", time.minute(), time.hour()))
        }
        When::Yearly(time_scheme) => {
            let time = time_scheme.parse()?;
            // Run every year on January 1st at the specified time
            // Format: "minute hour 1 1 *"
            Ok(format!("{} {} 1 1 *", time.minute(), time.hour()))
        }
        _ => Ok("".to_string()),
    }
}
