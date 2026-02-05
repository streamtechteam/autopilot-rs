use chrono::NaiveDate;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use log::error;

use crate::conditions::ConditionScheme;

pub fn create() {
    let name: String = match Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a name :")
        .interact_text()
    {
        Ok(value) => value,
        Err(err) => {
            error!("Failed to get job name");
            "".to_string()
        }
    };

    let description: String = match Input::with_theme(&ColorfulTheme::default())
        .allow_empty(true)
        .with_prompt("Choose a description :")
        .interact_text()
    {
        Ok(value) => value,
        Err(err) => {
            error!("Failed to get job description");
            "".to_string()
        }
    };

    let mut when: bool = match Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to have a time condition ? (when) :")
        .interact()
    {
        Ok(value) => value,
        Err(err) => {
            error!("Failed to get user input");
            false
        }
    };

    let mut date: String = String::new();
    if when {
        loop {
            let input_date: String = match Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Choose a date (YYYY/MM/DD):")
                .interact_text()
            {
                Ok(value) => value,
                Err(err) => {
                    error!("Failed to get job date");
                    continue;
                }
            };

            match NaiveDate::parse_from_str(&input_date, "%Y/%m/%d") {
                Ok(parsed_date) => {
                    date = parsed_date.to_string();
                    break;
                }
                Err(err) => {
                    println!("Date format is invalid. Please use YYYY/MM/DD format.");
                    continue;
                }
            }
        }
    }

    // let conditions  = Select::with_theme(&ColorfulTheme::default()).items(ConditionScheme::varient_names(&self))
    // .default(" ").;
}
