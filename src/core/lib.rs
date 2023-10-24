pub mod data;
pub mod api;

use std::fmt::{Display, Formatter};
use std::process::{Child, Command};
use crate::data::Configuration;
use crate::data::State;
use crate::data::State::{FATAL, REGISTERED, STARTING};


pub const UNIX_DOMAIN_SOCKET_PATH: &str = "/tmp/.unixdomain.sock";

//TODO: Validation of stdout/stderr files path
//TODO: Check existing of working dir

pub struct Task {
    configuration: Configuration,
    state: State,
    _restarts_left: u32,
    child: Option<Child>,
    _started_at: &'static str //change type
}

impl Task {
    pub fn new(configuration: Configuration) -> Task {
        Task {
            _restarts_left: configuration.start_retries,
            configuration,
            state: REGISTERED,
            child: None,
            _started_at: "time"
        }
    }
    
    pub fn run(&mut self, _force_launch: bool) {
        //force launch, from client
        //retries
        match Command::new(&self.configuration.cmd)
            .spawn() {
            Ok(child) => {
                self.child = Some(child);
                self.state = STARTING;
            }
            Err(_) => {
                //add logging + exitcode
                self.state = FATAL;
            }
        }
    }
    
    pub fn stop(&mut self) {
        
    }
    
    pub fn get_state(&self) -> &State {
        &self.state
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.state)
    }
}
