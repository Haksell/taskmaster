use std::fmt::{Display, Formatter};
use std::fs::{File, OpenOptions};
use std::process::{Child, Command, Stdio};
use std::time::SystemTime;
use crate::core::configuration::{Configuration, State};
use crate::core::configuration::State::{FATAL, REGISTERED, STARTING, STOPPED};
use crate::core::logger::Logger;

//TODO: Validation of stdout/stderr files path
//TODO: Check existing of working dir

pub struct Task {
    pub(crate) configuration: Configuration,
    pub(crate) state: State,
    _restarts_left: u32,
    pub(crate) child: Option<Child>,
    pub(crate) exit_code: Option<i32>,
    started_at: Option<SystemTime>,
    last_error: Option<String>,
}

impl Task {
    pub fn new(configuration: &Configuration) -> Task {
        Task {
            _restarts_left: configuration.start_retries,
            configuration: configuration.clone(),
            state: REGISTERED,
            exit_code: None,
            child: None,
            started_at: None,
            last_error: None,
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


    fn setup_child_process(&mut self, stderr: Stdio, stdout: Stdio) -> Result<(), String> {
        let argv: Vec<_> = self.configuration.cmd.split_whitespace().collect();

        match Command::new(argv[0])
            .args(&argv[1..])
            .current_dir(match &self.configuration.working_dir {
                Some(cwd) => &cwd,
                None => ".",
            })
            .envs(&self.configuration.env)
            .stdout(stdout)
            .stderr(stderr)
            .spawn() {
            Ok(child) => {
                self.child = Some(child);
                self.state = STARTING;
                self.started_at = Some(SystemTime::now());
                Ok(())
            }
            Err(err) => {
                self.last_error = Some(err.to_string());
                self.state = FATAL;
                Err(err.to_string())
            }
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        let stderr = self.setup_stream(&self.configuration.stderr)
            .map_err(|e| {
                self.state = FATAL;
                self.last_error = Some(e.to_string());
                e
            })?;
        let stdout = self.setup_stream(&self.configuration.stdout)
            .map_err(|e| {
                self.state = FATAL;
                self.last_error = Some(e.to_string());
                e
            })?;

        self.setup_child_process(stderr, stdout)?;

        Ok(())
    }


    pub fn stop(&mut self) -> Result<(), String> {
       return match &mut self.child {
           None => Err(format!("Can't find child process, probably was already stopped or not stared")),
           Some(child) => {
               if let Err(error) = child.kill() {
                   return Err(format!("Can't kill child process, {error}"))
               }
               self.state = STOPPED;
               self.child = None;
               Ok(())
           }
       } 
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
