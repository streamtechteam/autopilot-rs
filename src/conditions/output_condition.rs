use crate::conditions::Condition;
use duct::cmd;
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutputConditionScheme {
    pub command: String,
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
        args = vec!["-Command", &command];
    }

    let output = cmd(shell, args)
        .read()
        .expect("Error while testing condition");

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
        args = vec!["-Command", &command];
    }

    let output = cmd(shell, args)
        .read()
        .expect("Error while testing condition");

    output.trim() == target
}
