use crate::{
    conditions::{Condition, ConditionScheme},
    error::AutoPilotError,
};
use dialoguer::{Input, theme::ColorfulTheme};
use duct::cmd;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct OutputCondition {
    command: String,
    target: String,
}
impl OutputCondition {
    pub fn new(command: String, target: String) -> Self {
        Self { command, target }
    }
    pub fn from_scheme(output_condition_scheme: OutputConditionScheme) -> Self {
        Self {
            command: output_condition_scheme.command,
            target: output_condition_scheme.target,
        }
    }
}
impl Condition for OutputCondition {
    fn check(&self) -> bool {
        sync_condition(&self.command, &self.target)
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }

    fn create(&self) -> Result<ConditionScheme, AutoPilotError> {
        //TODO
        let command = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter command : ")
            .interact_text()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;
        let target = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter target : ")
            .interact_text()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;
        Ok(ConditionScheme::Output(OutputConditionScheme {
            command,
            target,
        }))
    }

    fn name(&self) -> &str {
        "Output"
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct OutputConditionScheme {
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub target: String,
}

pub fn sync_condition(command: &str, target: &str) -> bool {
    let args: Vec<&str>;
    let shell: &str;

    #[cfg(target_os = "windows")]
    {
        shell = "powershell";
        args = vec!["-Command", &command];
        // let output = cmd("powershell", &["-Command", &command])
        // .read()
        // .expect("Error while testing condition");
    }
    #[cfg(target_os = "linux")]
    {
        shell = "sh";
        args = vec!["-c", &command];
    }
    #[cfg(target_os = "macos")]
    {
        shell = "zsh";
        args = vec!["-c", &command];
    }

    let output = match cmd(shell, args).read() {
        Ok(out) => out,
        Err(e) => {
            error!("Error while testing condition '{}': {}", command, e);
            return false; // Return false if command execution fails
        }
    };

    output.trim() == target
}

pub async fn async_condition(command: &str, target: &str) -> bool {
    let args: Vec<&str>;
    let shell: &str;
    #[cfg(target_os = "windows")]
    {
        shell = "powershell";
        args = vec!["-Command", &command];
    }
    #[cfg(target_os = "linux")]
    {
        shell = "sh";
        args = vec!["-c", &command];
    }
    #[cfg(target_os = "macos")]
    {
        shell = "zsh";
        args = vec!["-c", &command];
    }

    let output = match cmd(shell, args).read() {
        Ok(out) => out,
        Err(e) => {
            error!("Error while testing condition '{}': {}", command, e);
            return false; // Return false if command execution fails
        }
    };

    output.trim() == target
}
