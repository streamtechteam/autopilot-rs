// use chrono::{Local, NaiveDate, NaiveTime, TimeZone};
// use log::error;
// use serde::{Deserialize, Serialize};
// use tokio_cron_scheduler::JobScheduler;

// use crate::{
//     conditions::{Condition},
//     time::{DateTimeScheme, Time},
// };

// pub struct TimeCondition {
//     pub time: chrono::DateTime<chrono::Local>,
// }

// impl TimeCondition {
//     pub fn new(time: chrono::DateTime<Local>) -> Self {
//         TimeCondition { time}
//     }
//     pub fn from_scheme(scheme: TimeConditionScheme, scheduler: &JobScheduler) -> Result<Self, String> {
//         // Parse date
//         let mut date: NaiveDate;
//         let mut time: NaiveTime;
//         let mut naive_dt;
//         let mut local_dt;
//         match NaiveTime::parse_from_str(&scheme.time.time, "%H:%M:%S") {
//             Ok(value) => {
//                 time = value;
//                 match NaiveDate::parse_from_str(&scheme.time.date, "%Y/%m/%d") {
//                     Ok(value) => {
//                         date = value;
//                         naive_dt = date.and_time(time);
//                         local_dt = Local
//                             .from_local_datetime(&naive_dt)
//                             .single()
//                             .ok_or("Ambiguous or invalid local datetime".to_string())?;
//                     }
//                     Err(e) => {
//                         local_dt = Local.with_ymd_and_hms(0000, 0, 0, 0, 0, 0).unwrap();

//                         error!("Invalid date format. Expected YYYY/MM/DD: {}", e);
//                     }
//                 }
//             }
//             Err(e) => {
//                 local_dt = Local.with_ymd_and_hms(0000, 0, 0, 0, 0, 0).unwrap();
//                 error!("Invalid time format. Expected HH:MM:SS: {}", e);
//             }
//         }

//         // Parse time

//         // Convert to local time safely

//         Ok(Self { time: local_dt })
//     }
//     // pub fn of_scheme(scheme: TimeConditionScheme) -> Self {
//     //     TimeCondition::new(scheme.time.into())
//     // }
// }
// impl Condition for TimeCondition {
//     fn check(&self , scheduler: Option<&JobScheduler>) -> bool {
//         sync_condition(self.time, scheduler)
//     }
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct TimeConditionScheme {
//     time: DateTimeScheme,
// }

// pub fn sync_condition(time: chrono::DateTime<Local>, scheduler: Option<&JobScheduler>) -> bool {
//     return true;
//     // Implementation of the sync condition
// }

// pub fn async_condition(time: chrono::DateTime<Local>) -> bool {
//     let current_time = chrono::Local::now();

//     current_time == time
//     // Implementation of the async condition
// }
