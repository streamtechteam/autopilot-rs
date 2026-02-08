use std::error::Error;

use chrono::{DateTime, Local, NaiveDate, NaiveTime, ParseError, TimeZone};
use log::error;
use serde::{Deserialize, Serialize};

use crate::error::AutoPilotError;

pub mod add;
pub mod init;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateTimeScheme {
    pub date: String,
    pub time: String,
}

pub fn to_datatime(scheme: DateTimeScheme) -> Result<DateTime<Local>, ParseError> {
    let date: NaiveDate;
    let time: NaiveTime;
    let naive_dt;
    let local_dt;
    match NaiveTime::parse_from_str(&scheme.time, "%H:%M:%S") {
        Ok(value) => {
            time = value;
            match NaiveDate::parse_from_str(&scheme.date, "%Y/%m/%d") {
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
            error!("Invalid time format. Expected HH:MM:SS: {}", e);
            return Err(e);
        }
    }

    Ok(local_dt)
}
