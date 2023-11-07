extern crate libc;
use crate::core::configuration::State::*;
use crate::core::configuration::{Configuration, State};
use libc::{mode_t, pid_t};
use std::fmt::{Display, Formatter};
use std::fs::{File, OpenOptions};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, SystemTime};

//TODO: Validation of stdout/stderr files path
//TODO: Check existing of working dir

pub struct Task {
    pub configuration: Configuration,
    pub state: State,
    pub restarts_left: u32,
    pub child: Option<Child>,
    pub exit_code: Option<i32>,
}

impl Task {
    pub fn new(configuration: &Configuration) -> Task {
        Task {
            restarts_left: configuration.start_retries,
            configuration: configuration.clone(),
            state: STOPPED(None),
            exit_code: None,
            child: None,
        }
    }

    fn open_file(path: &String) -> Result<File, String> {
        OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .map_err(|e| e.to_string())
    }

    fn setup_stream(&self, stream_type: &Option<String>) -> Result<Stdio, String> {
        match stream_type {
            Some(path) => Task::open_file(path).map(|file| file.into()),
            None => Ok(Stdio::null()),
        }
    }

    unsafe fn setup_child_process(&mut self, stderr: Stdio, stdout: Stdio) -> Result<(), String> {
        let argv: Vec<_> = self.configuration.cmd.split_whitespace().collect();
        let umask_val = self.configuration.umask as mode_t;
        match Command::new(argv[0])
            .args(&argv[1..])
            .current_dir(match &self.configuration.working_dir {
                Some(cwd) => &cwd,
                None => ".",
            })
            .envs(&self.configuration.env)
            .stdout(stdout)
            .stderr(stderr)
            .pre_exec(move || {
                unsafe {
                    libc::umask(umask_val);
                }
                Ok(())
            })
            .spawn()
        {
            Ok(child) => {
                self.child = Some(child);
                Ok(())
            }
            Err(err) => {
                let err_msg = format!("Command: {}", err.to_string());
                self.state = FATAL(err_msg.clone());
                Err(err_msg)
            }
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        self.state = STARTING(SystemTime::now());
        let stderr = self.setup_stream(&self.configuration.stderr).map_err(|e| {
            let error_msg = format!("Stderr log file: {}", e);
            self.state = FATAL(error_msg.clone());
            error_msg
        })?;
        let stdout = self.setup_stream(&self.configuration.stdout).map_err(|e| {
            let error_msg = format!("Stdout log file: {}", e);
            self.state = FATAL(error_msg.clone());
            error_msg
        })?;

        unsafe {
            self.setup_child_process(stderr, stdout)?;
        }

        Ok(())
    }

    pub fn kill(&mut self) -> Result<(), String> {
        return match &mut self.child {
            None => Err(format!(
                "Error! Can't find child process, probably was already stopped or not started"
            )),
            Some(child) => {
                if let Err(error) = child.kill() {
                    return Err(format!("Error! Can't kill child process, {error}"));
                }
                self.state = STOPPED(Some(SystemTime::now()));
                self.child = None;
                Ok(())
            }
        };
    }

    pub fn stop(&mut self) -> Result<(), String> {
        return match &mut self.child {
            None => Err(format!(
                "Error! Can't find child process, probably was already stopped or not started"
            )),
            Some(child) => {
                unsafe {
                    libc::kill(
                        child.id() as pid_t,
                        self.configuration.stop_signal.clone().into(),
                    );
                }
                self.state = STOPPING(SystemTime::now());
                Ok(())
            }
        };
    }

    pub fn get_json_configuration(&self) -> String {
        serde_json::to_string_pretty(&self.configuration).expect("Error! Serialization failed")
    }

    pub fn is_passed_starting_period(&self, started_at: SystemTime) -> bool {
        let current_time = SystemTime::now();
        let elapsed_time = current_time
            .duration_since(started_at)
            .unwrap_or(Duration::from_secs(0));
        elapsed_time.as_secs() >= self.configuration.start_time
    }

    pub fn is_passed_stopping_period(&self, stopped_at: SystemTime) -> bool {
        let current_time = SystemTime::now();
        let elapsed_time = current_time
            .duration_since(stopped_at)
            .unwrap_or(Duration::from_secs(0));
        elapsed_time.as_secs() >= self.configuration.stop_time
    }

    pub fn can_be_launched(&self) -> bool {
        match self.state {
            STOPPED(_) | EXITED(_) | FATAL(_) => true,
            _ => false,
        }
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut result = self.state.to_string();
        match self.state {
            STOPPING(_) => {}
            STOPPED(_) => {}
            STARTING(_) => {}
            RUNNING(_) => {
                let pid = match self.child.as_ref() {
                    None => 0,
                    Some(child) => child.id(),
                };
                result += &format!("\t\tPID {}", pid)
            }
            BACKOFF => result += "\tExited too quickly",
            EXITED(_) => {}
            FATAL(_) => {}
            UNKNOWN => {}
        };
        write!(f, "{result}")
    }
}
