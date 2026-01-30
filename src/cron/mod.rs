use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone};
use log::error;
use serde::{Deserialize, Serialize};

pub mod add;
pub mod init;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateTimeScheme {
    pub date: String,
    pub time: String,
}

pub fn to_datatime(scheme: DateTimeScheme) -> Option<DateTime<Local>> {
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
                        .unwrap();
                }
                Err(e) => {
                    local_dt = Local.with_ymd_and_hms(0000, 0, 0, 0, 0, 0).unwrap();

                    error!("Invalid date format. Expected YYYY/MM/DD: {}", e);
                }
            }
        }
        Err(e) => {
            local_dt = Local.with_ymd_and_hms(0000, 0, 0, 0, 0, 0).unwrap();
            error!("Invalid time format. Expected HH:MM:SS: {}", e);
        }
    }

    // Parse time

    // Convert to local time safely

    Some(local_dt)
}
