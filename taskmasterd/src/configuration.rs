use crate::logger::Logger;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use validator::{Validate, ValidationError};

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Clone)]
pub enum AutoRestart {
    #[serde(rename = "true")]
    True,
    #[serde(rename = "false")]
    False,
    #[serde(rename = "unexpected")]
    Unexpected,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Clone)]
pub enum StopSignal {
    TERM = libc::SIGTERM as isize,
    HUP = libc::SIGHUP as isize,
    INT = libc::SIGINT as isize,
    QUIT = libc::SIGQUIT as isize,
    KILL = libc::SIGKILL as isize,
    USR1 = libc::SIGUSR1 as isize,
    USR2 = libc::SIGUSR2 as isize,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum State {
    STOPPED(Option<SystemTime>),
    STARTING(SystemTime),
    RUNNING(SystemTime),
    BACKOFF,
    STOPPING(SystemTime),
    EXITED(SystemTime),
    FATAL(String),
}

impl Display for StopSignal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StopSignal::TERM => write!(f, "TERM"),
            StopSignal::HUP => write!(f, "HUP"),
            StopSignal::INT => write!(f, "INT"),
            StopSignal::QUIT => write!(f, "QUIT"),
            StopSignal::KILL => write!(f, "KILL"),
            StopSignal::USR1 => write!(f, "USR1"),
            StopSignal::USR2 => write!(f, "USR2"),
        }
    }
}

impl Into<libc::c_int> for StopSignal {
    fn into(self) -> libc::c_int {
        match self {
            StopSignal::TERM => libc::SIGTERM,
            StopSignal::HUP => libc::SIGHUP,
            StopSignal::INT => libc::SIGINT,
            StopSignal::QUIT => libc::SIGQUIT,
            StopSignal::KILL => libc::SIGKILL,
            StopSignal::USR1 => libc::SIGUSR1,
            StopSignal::USR2 => libc::SIGUSR2,
        }
    }
}

impl State {
    fn at(time_stamp: &SystemTime) -> String {
        let since_the_epoch = time_stamp.duration_since(UNIX_EPOCH).unwrap();
        let now_in_sec = since_the_epoch.as_secs();
        let hours = (now_in_sec % (24 * 3600)) / 3600;
        let minutes = (now_in_sec % 3600) / 60;
        let seconds = now_in_sec % 60;
        format!("at {:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let keyword = match self {
            State::STOPPED(stopped_at) => match stopped_at {
                None => "stopped".to_string(),
                Some(stopped_at) => {
                    format!("stopped {}", State::at(stopped_at))
                }
            },
            State::STARTING(_) => "starting".to_string(),
            State::RUNNING(start_time) => {
                let current_time = SystemTime::now();
                let elapsed_time = current_time
                    .duration_since(start_time.clone())
                    .unwrap_or(Duration::from_secs(0));
                let elapsed_time_seconds = elapsed_time.as_secs();
                let hours = elapsed_time_seconds / 3600;
                let minutes = (elapsed_time_seconds % 3600) / 60;
                let seconds = elapsed_time_seconds % 60;
                format!(
                    "running (uptime {:02}:{:02}:{:02})",
                    hours, minutes, seconds
                )
            }
            State::BACKOFF => "backoff".to_string(),
            State::EXITED(exited_at) => {
                format!("exited {}", State::at(exited_at))
            }
            State::FATAL(error) => {
                format!("fatal ({error})")
            }
            State::STOPPING(_) => "stopping".to_string(),
        };
        write!(f, "{}", keyword)
    }
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Clone, Validate)]
#[serde(default)]
pub struct Configuration {
    #[serde(deserialize_with = "deserialize_string_and_trim")]
    #[validate(length(min = 1, message = "cmd: can't be empty"))]
    pub cmd: String,
    #[validate(range(
        min = 1,
        max = 100000,
        message = "num_procs value should be between 1 and 100000"
    ))]
    pub num_procs: u32,
    #[serde(deserialize_with = "deserialize_umask")]
    #[validate(custom = "validate_umask")]
    pub umask: u32,
    #[serde(deserialize_with = "deserialize_option_string_and_trim")]
    pub working_dir: Option<String>,
    pub auto_start: bool,
    pub auto_restart: AutoRestart,
    pub exit_codes: Vec<i32>,
    pub start_retries: u32,
    pub start_time: u64,
    pub stop_signal: StopSignal,
    #[validate(range(min = 1, message = "invalid stop_time"))]
    pub stop_time: u64,
    #[serde(deserialize_with = "deserialize_option_string_and_trim")]
    pub stdout: Option<String>,
    #[serde(deserialize_with = "deserialize_option_string_and_trim")]
    pub stderr: Option<String>,
    pub env: BTreeMap<String, String>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            cmd: String::new(),
            num_procs: 1,
            umask: 0o022,
            working_dir: None,
            auto_start: true,
            auto_restart: AutoRestart::Unexpected,
            exit_codes: vec![0],
            start_retries: 3,
            start_time: 1,
            stop_signal: StopSignal::TERM,
            stop_time: 10,
            stdout: None,
            stderr: None,
            env: Default::default(),
        }
    }
}

impl Configuration {
    pub fn from_yml(
        path: String,
        logger: Arc<Mutex<Logger>>,
    ) -> Result<BTreeMap<String, Configuration>, String> {
        let mut logger = logger.lock().unwrap();
        logger.log(format!("Reading {path}"));
        let mut file = File::open(&path).map_err(|err| format!("{}: {}", path, err))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|err| format!("Can't read the file: {err}"))?;
        let tasks: BTreeMap<String, Configuration> =
            serde_yaml::from_str(&content).map_err(|err| err.to_string())?;
        let mut errors = Vec::new();
        for (key, task) in &tasks {
            match task.validate() {
                Ok(_) => {
                    logger.log(format!("{key}: validated"));
                }
                Err(e) => {
                    for (_k, value) in e.field_errors() {
                        for validation_error in value {
                            errors.push(format!(
                                "Configuration error: {key}: {:?}",
                                if let Some(message) = validation_error.message.as_ref() {
                                    message.to_string()
                                } else {
                                    validation_error.code.to_string()
                                }
                            ));
                        }
                    }
                }
            }
        }
        if errors.is_empty() {
            Ok(tasks)
        } else {
            Err(errors.join("\n"))
        }
    }
}

fn validate_umask(value: u32) -> Result<(), ValidationError> {
    if !(value & 0o777 == value) {
        return Err(ValidationError::new("Invalid umask"));
    }
    Ok(())
}

fn deserialize_umask<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    match u32::from_str_radix(value.as_str(), 8) {
        Ok(umask) => Ok(umask),
        Err(_) => Err(serde::de::Error::custom(format!(
            "\"{value}\" is not a valid umask."
        ))),
    }
}

fn deserialize_string_and_trim<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer).map(|s| s.trim().to_string())
}

fn deserialize_option_string_and_trim<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer).map(|s| Some(s.trim().to_string()))
}
