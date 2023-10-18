mod data;

use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Deserializer};
use validator::Validate;
use regex::Regex;
use crate::data::Autorestart;

pub const UNIX_DOMAIN_SOCKET_PATH: &str = ".unixdomain.sock";


#[derive(Debug, Validate, Deserialize)]
pub struct Task {
    #[serde(deserialize_with = "deserialize_string_and_trim")]
    #[validate(length(min = 1, message = "can't be empty"))]
    cmd: String,
    #[serde(default = "default_numprocs")]
    #[validate(range(min = 1))]
    num_procs: u32,
    #[serde(deserialize_with = "deserialize_umask", default = "default_mask")]
    umask: u32,
    working_dir: Option<String>,
    #[serde(default = "default_autostart")]
    auto_start: bool,
    auto_restart: Autorestart,
    #[serde(default = "default_exitcodes")]
    exit_codes: Vec<u8>,
    #[serde(default = "default_retries")]
    start_retries: u32,
    #[serde(default = "default_start_time")]
    start_time: u32,
    stop_signal: String, //todo
    stop_time: u32, //todo
    stdout: String, //todo
    stderr: String, //todo
    #[serde(default = "default_env")]
    env: BTreeMap<String, String>,
}

fn default_env() -> BTreeMap<String, String> {
    BTreeMap::new()
}

fn default_numprocs() -> u32 {
    1
}

fn default_mask() -> u32 {
    0o022
}

fn default_autostart() -> bool {
    true
}

fn default_exitcodes() -> Vec<u8> {
    vec![0]
}

fn default_retries() -> u32 {
    3
}

fn default_start_time() -> u32 {
    3
}

fn deserialize_umask<'de, D>(deserializer: D) -> Result<u32, D::Error>
    where
        D: Deserializer<'de>
{
    let value = String::deserialize(deserializer)?;
    let regex = Regex::new(r"\b(?:0o)?[0-7]{3}\b").unwrap();

    match regex.captures(value.as_str()) {
        None => { return Err(serde::de::Error::custom(format!("\"{}\", Umask should be umask in octal representation", value))); }
        Some(_) => {}
    };
    if let Ok(umask) = u32::from_str_radix(value.as_str(), 8) {
        Ok(umask)
    } else {
        Err(serde::de::Error::custom(format!("\"{}\", Mask should be in octal format e.g. 022, 777", value)))
    }
}

fn deserialize_string_and_trim<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>
{
    let trimmed = String::deserialize(deserializer)?.trim().to_string();
    Ok(trimmed)
}

impl Task {
    pub fn from_yml(path: String) -> Result<BTreeMap<String, Task>, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let tasks: BTreeMap<String, Task> = serde_yaml::from_str(&content)?;
        for (_key, task) in &tasks {
            match task.validate() {
                Ok(_) => {}
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
        Ok(tasks)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use crate::data::Autorestart::Unexpected;
    use crate::Task;
    const CMD_EMPTY: &str = "config_files/test/cmd_empty.yml";
    const CMD_NOT_PROVIDED: &str = "config_files/test/cmd_not_provided.yml";
    const CMD_ONLY_WHITE_SPACES: &str = "config_files/test/cmd_white_spaces.yml";
    const CMD_WHITE_SPACES_BEFORE_AND_AFTER: &str = "config_files/test/cmd_whitespaces_before_and_after.yml";

    #[test]
    fn cmd_empty_should_return_error() {
        //given && when
        let task = Task::from_yml(CMD_EMPTY.into());
        
        //then
        assert!(task.is_err());
        if let Err(error) = task {
            assert_eq!("cmd: can't be empty", error.to_string())
        }
    }

    #[test]
    fn cmd_not_provided_should_return_error() {
        //given && when
        let task = Task::from_yml(CMD_NOT_PROVIDED.into());

        //then
        assert!(task.is_err());
        if let Err(error) = task {
            assert!(error.to_string().contains("missing field `cmd`"));
        }
    }

    #[test]
    fn cmd_white_spaces_should_return_error() {
        //given && when
        let task = Task::from_yml(CMD_ONLY_WHITE_SPACES.into());

        //then
        assert!(task.is_err());
        if let Err(error) = task {
            assert_eq!("cmd: can't be empty", error.to_string())
        }
    }


    
    
}