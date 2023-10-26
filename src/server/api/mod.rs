pub mod error_log;

use crate::api::Action::{Config, Exit, Help, Start, Status, Stop};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;
use std::slice::Iter;

pub const API_KEYWORD_HELP: &'static str = "help";
pub const API_KEYWORD_STATUS: &'static str = "status";
pub const API_KEYWORD_EXIT: &'static str = "exit";
pub const API_KEYWORD_CONFIG: &'static str = "config";
pub const API_KEYWORD_START: &'static str = "start";
pub const API_KEYWORD_STOP: &'static str = "stop";
pub const API_STATUS_DESCR: &'static str = "status without args returns the status of all tasks\n\
                                             \tstatus <task_name> returns the status of a specific task";
pub const API_EXIT_DESCR: &'static str = "exit from the CLI";
pub const API_CONFIG_DESCR: &'static str = "config <task_name> returns configuration details";

//TODO: add unit tests
#[derive(Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum Action {
    Config(String),
    Status(Option<String>),
    Start(String),
    Stop(String),
    Help,
    Exit,
}

impl Action {
    //TODO: DELETE, will be replaced by py client
    pub fn from(action: &str) -> Result<Action, String> {
        let mut split: Vec<&str> = action.split_whitespace().collect();
        if split.is_empty() {
            return Err(String::new());
        }

        let enum_value = Action::iterator()
            .find(|enum_value| enum_value.to_string() == split[0]);

        if enum_value.is_none() {
            return Err(format!("Unknown action: {}", split[0]));
        }

        let result = enum_value.unwrap();
        split.remove(0);

        match result {
            Status(_) => {
                if split.len() > 1 {
                    Err(format!(
                        "{}: doesn't take more then 1 argument (task name)",
                        result.to_string()
                    ))
                } else if split.len() == 1 {
                    Ok(Status(Some(split[0].to_string())))
                } else {
                    Ok(Status(None))
                }
            }
            Config(_) => {
                if split.len() != 1 {
                    Err(format!(
                        "{}: should have 1 argument (task name)",
                        result.to_string()
                    ))
                } else {
                    Ok(Config(split[0].to_string()))
                }
            }
            Help => {
                if split.len() > 0 {
                    return Err(format!(
                        "Unknown arguments for {} action: {:?}",
                        result.to_string(),
                        split
                    ));
                }
                Ok(Help)
            }
            Start(_) => {
                if split.len() != 1 {
                    Err(format!(
                        "{}: should have 1 argument (task name)",
                        result.to_string()
                    ))
                } else {
                    Ok(Start(split[0].to_string()))
                }
            }
            Stop(_) => {
                if split.len() != 1 {
                    Err(format!(
                        "{}: should have 1 argument (task name)",
                        result.to_string()
                    ))
                } else {
                    Ok(Stop(split[0].to_string()))
                }
            }
            Exit => Ok(Exit),
        }
    }

    pub fn iterator() -> Iter<'static, Action> {
        static ACTIONS: [Action; 6] = [
            Status(None),
            Help,
            Exit,
            Config(String::new()),
            Start(String::new()),
            Stop(String::new())
        ];
        ACTIONS.iter()
    }

    //TODO: DELETE, will be replace by py client
    pub fn get_description(&self) -> String {
        match self {
            Start(_) => String::from("descr"),
            Stop(_) => String::from("descr"),
            Status(_) => String::from(API_STATUS_DESCR),
            Exit => String::from(API_EXIT_DESCR),
            Config(_) => String::from(API_CONFIG_DESCR),
            Help => {
                let mut result = String::new();
                for (i, action) in Action::iterator().enumerate() {
                    if *action != Help {
                        result
                            .push_str(format!("{}: {}", action, action.get_description()).as_str());
                        if i != Action::iterator().len() - 1 {
                            result.push_str("\n")
                        }
                    }
                }
                result
            }
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let keyword = match self {
            Status(_) => API_KEYWORD_STATUS,
            Help => API_KEYWORD_HELP,
            Exit => API_KEYWORD_EXIT,
            Config(_) => API_KEYWORD_CONFIG,
            Start(_) => API_KEYWORD_START,
            Stop(_) => API_KEYWORD_STOP
        };
        write!(f, "{}", keyword)
    }
}
