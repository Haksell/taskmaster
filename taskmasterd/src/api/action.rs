use crate::api::action::Action::Update;
use crate::api::Action::{Config, Shutdown, Start, Status, Stop};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;
use std::slice::Iter;

pub const API_KEYWORD_STATUS: &'static str = "status";
pub const API_KEYWORD_SHUTDOWN: &'static str = "exit";
pub const API_KEYWORD_CONFIG: &'static str = "config";
pub const API_KEYWORD_START: &'static str = "start";
pub const API_KEYWORD_STOP: &'static str = "stop";
pub const API_STATUS_DESCR: &'static str = "status without args returns the status of all tasks\n\
                                             \tstatus <task_name> returns the status of a specific task";
pub const API_EXIT_DESCR: &'static str = "exit from the CLI";
pub const API_CONFIG_DESCR: &'static str = "config <task_name> returns configuration details";

//TODO: add unit tests
//TODO: Refactor after changing client to py
#[derive(Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum Action {
    Config(String),
    Update(String),
    Status(Option<String>),
    Start(String),
    Stop(String),
    Shutdown,
}

impl Action {
    pub fn iterator() -> Iter<'static, Action> {
        static ACTIONS: [Action; 6] = [
            Status(None),
            Shutdown,
            Config(String::new()),
            Start(String::new()),
            Stop(String::new()),
            Update(String::new()),
        ];
        ACTIONS.iter()
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let keyword = match self {
            Status(_) => API_KEYWORD_STATUS,
            Shutdown => API_KEYWORD_SHUTDOWN,
            Config(_) => API_KEYWORD_CONFIG,
            Start(_) => API_KEYWORD_START,
            Stop(_) => API_KEYWORD_STOP,
            Update(_) => "update",
        };
        write!(f, "{}", keyword)
    }
}
