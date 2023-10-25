pub mod api;
pub mod data;

use crate::data::Configuration;
use crate::data::State;
use crate::data::State::{FATAL, REGISTERED, STARTING};
use std::fmt::{Display, Formatter};
use std::process::{Child, Command};

pub const UNIX_DOMAIN_SOCKET_PATH: &str = "/tmp/.unixdomain.sock";

//TODO: Validation of stdout/stderr files path
//TODO: Check existing of working dir

pub struct Task {
    configuration: Configuration,
    state: State,
    _restarts_left: u32,
    child: Option<Child>,
    _started_at: &'static str, //change type
}

impl Task {
    pub fn new(configuration: Configuration) -> Task {
        Task {
            _restarts_left: configuration.start_retries,
            configuration,
            state: REGISTERED,
            child: None,
            _started_at: "time",
        }
    }

    pub fn run(&mut self, _force_launch: bool) {
        let argv: Vec<_> = self.configuration.cmd.split_whitespace().collect();
        match Command::new(argv[0])
            .args(&argv[1..])
            .current_dir(match &self.configuration.working_dir {
                Some(cwd) => &cwd,
                None => ".",
            })
            .envs(&self.configuration.env)
            .spawn()
        {
            Ok(child) => {
                self.child = Some(child);
                self.state = STARTING;
            }
            Err(err) => {
                println!("{err}");
                //add logging + exitcode
                self.state = FATAL;
            }
        }
    }

    pub fn stop(&mut self) {}

    pub fn get_state(&self) -> &State {
        &self.state
    }

    pub fn get_json_configuration(&self) -> String {
        serde_json::to_string_pretty(&self.configuration).expect("Serialization failed")
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.state)
    }
}
