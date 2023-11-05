extern crate libc;
use crate::core::configuration::State::*;
use crate::core::configuration::{Configuration, State};
use libc::mode_t;
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
    _restarts_left: u32,
    pub child: Option<Child>,
    pub exit_code: Option<i32>,
}

impl Task {
    pub fn new(configuration: &Configuration) -> Task {
        Task {
            _restarts_left: configuration.start_retries,
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
                self.state = FATAL(err.to_string());
                Err(err.to_string())
            }
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        self.state = STARTING;
        let stderr = self.setup_stream(&self.configuration.stderr).map_err(|e| {
            self.state = FATAL(e.to_string());
            e
        })?;
        let stdout = self.setup_stream(&self.configuration.stdout).map_err(|e| {
            self.state = FATAL(e.to_string());
            e
        })?;

        unsafe {
            self.setup_child_process(stderr, stdout)?;
        }

        Ok(())
    }

    pub fn kill(&mut self) -> Result<(), String> {
        return match &mut self.child {
            None => Err(format!(
                "Can't find child process, probably was already stopped or not stared"
            )),
            Some(child) => {
                if let Err(error) = child.kill() {
                    return Err(format!("Can't kill child process, {error}"));
                }
                self.state = STOPPED(Some(SystemTime::now()));
                self.child = None;
                Ok(())
            }
        };
    }

    //TODO: finish that to send good signal
    pub fn _stop(&mut self) -> Result<(), String> {
        return match &mut self.child {
            None => Err(format!(
                "Can't find child process, probably was already stopped or not stared"
            )),
            Some(child) => {
                let _kill = Command::new("kill")
                    .args([
                        "-s",
                        &self.configuration.stop_signal.to_string(),
                        &child.id().to_string(),
                    ])
                    .spawn();
                if let Err(error) = child.kill() {
                    return Err(format!("Can't kill child process, {error}"));
                }
                self.state = STOPPED(Some(SystemTime::now()));
                self.child = None;
                Ok(())
            }
        };
    }

    pub fn get_json_configuration(&self) -> String {
        serde_json::to_string_pretty(&self.configuration).expect("Serialization failed")
    }

    fn is_exited_too_quickly(&self, started_at: SystemTime) -> bool {
        let current_time = SystemTime::now();
        let elapsed_time = current_time
            .duration_since(started_at)
            .unwrap_or(Duration::from_secs(0));
        elapsed_time.as_secs() < self.configuration.start_time
    }

    pub fn set_finished(&mut self, exit_code: Option<i32>) {
        let old_state = self.state.clone();
        self.state = FINISHED;
        if let RUNNING(started_time) = old_state {
            if self.is_exited_too_quickly(started_time) {
                self.state = BACKOFF;
            }
        }
        self.exit_code = exit_code;
        self.child = None;
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut result = self.state.to_string();
        match self.state {
            FINISHED => {}
            STOPPED(_) => {}
            STARTING => {}
            RUNNING(_) => {
                let pid = match self.child.as_ref() {
                    None => 0,
                    Some(child) => child.id(),
                };
                result += &format!("\t\tPID {}", pid)
            }
            BACKOFF => result += "\tExited too quickly",
            _EXITED => {}
            FATAL(_) => {}
            _UNKNOWN => {}
        };
        write!(f, "{result}")
    }
}
