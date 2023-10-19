mod data;

use crate::data::StopSignal::TERM;
use crate::data::{AutoRestart, StopSignal};
use regex::Regex;
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use validator::Validate;

pub const UNIX_DOMAIN_SOCKET_PATH: &str = "/tmp/.unixdomain.sock";

//TODO: Validation of stdout/stderr files path
//TODO: Check existing of working dir

#[derive(Debug, PartialEq, Validate, Deserialize)]
#[serde(default)]
pub struct Task {
    #[serde(deserialize_with = "deserialize_string_and_trim")]
    #[validate(length(min = 1, message = "can't be empty"))]
    cmd: String,
    #[validate(range(min = 1))]
    num_procs: u32,
    #[serde(deserialize_with = "deserialize_umask")]
    umask: u32,
    #[serde(deserialize_with = "deserialize_option_string_and_trim")]
    working_dir: Option<String>,
    auto_start: bool,
    auto_restart: AutoRestart,
    exit_codes: Vec<u8>,
    start_retries: u32,
    start_time: u32,
    stop_signal: StopSignal,
    stop_time: u32,
    #[serde(deserialize_with = "deserialize_option_string_and_trim")]
    stdout: Option<String>,
    #[serde(deserialize_with = "deserialize_option_string_and_trim")]
    stderr: Option<String>,
    env: BTreeMap<String, String>,
}

fn deserialize_umask<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    let regex = Regex::new(r"\b(?:0o)?[0-7]{3}\b").unwrap();

    if !regex.is_match(value.as_str()) {
        return Err(serde::de::Error::custom(format!(
            "\"{}\", Umask should be umask in octal representation",
            value
        )));
    }
    match u32::from_str_radix(value.as_str(), 8) {
        Ok(umask) => Ok(umask),
        Err(_) => Err(serde::de::Error::custom(format!(
            "\"{}\" is not a valid umask in octal format, e.g., 022, 777",
            value
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

impl Default for Task {
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
            stop_signal: TERM,
            stop_time: 10,
            stdout: None,
            stderr: None,
            env: Default::default(),
        }
    }
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
                Err(e) => return Err(e.into()),
            }
        }
        Ok(tasks)
    }
}

#[cfg(test)]
mod tests {
    use crate::data::AutoRestart::Unexpected;
    use crate::data::StopSignal::TERM;
    use crate::Task;
    use std::collections::BTreeMap;

    const CMD_EMPTY: &str = "config_files/test/cmd_empty.yml";
    const CMD_NOT_PROVIDED: &str = "config_files/test/cmd_not_provided.yml";
    const CMD_ONLY_WHITE_SPACES: &str = "config_files/test/cmd_white_spaces.yml";
    const CMD_WHITE_SPACES_BEFORE_AND_AFTER: &str =
        "config_files/test/cmd_whitespaces_before_and_after.yml";
    const CONFIG_ONLY_CMD_PRESENT: &str = "config_files/test/config_only_cmd.yml";
    const MASK_NOT_VALID_1: &str = "config_files/test/umask_not_valid_1.yml";
    const MASK_NOT_VALID_2: &str = "config_files/test/umask_not_valid_2.yml";
    const MASK_NOT_VALID_3: &str = "config_files/test/umask_not_valid_3.yml";
    const WORKING_DIR_WITH_SPACES: &str = "config_files/test/working_dir_with_spaces.yml";
    const STDOUT_WITH_SPACES: &str = "config_files/test/stdout_path_with_spaces.yml";
    const STDERR_WITH_SPACES: &str = "config_files/test/stderr_path_with_spaces.yml";

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
            assert!(error.to_string().contains("cmd: can't be empty"));
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

    #[test]
    fn cmd_white_spaces_before_and_after_should_return_trimmed_cmd() {
        //given
        let expected_key = String::from("task1");
        let expected_value = Task {
            cmd: String::from("while true; do echo 'Task 1 output'; sleep 3; done"),
            num_procs: 1,
            umask: 0o777,
            working_dir: Some(String::from("/tmp")),
            auto_start: true,
            auto_restart: Unexpected,
            exit_codes: vec![0, 2],
            start_retries: 3,
            start_time: 5,
            stop_signal: TERM,
            stop_time: 10,
            stdout: Some(String::from("/tmp/task1.stdout")),
            stderr: Some(String::from("/tmp/task1.stderr")),
            env: BTreeMap::new(),
        };

        // when
        let task = Task::from_yml(CMD_WHITE_SPACES_BEFORE_AND_AFTER.into()).unwrap();

        //then
        let mut expected_map = BTreeMap::new();
        expected_map.insert(expected_key, expected_value);
        assert_eq!(task, expected_map);
    }

    #[test]
    fn only_cmd_cfg_should_put_default_values() {
        //given
        let mut expected_task = Task::default();
        expected_task.cmd = String::from("only_cmd_is_present");
        let expected_key = String::from("task1");
        let mut expected: BTreeMap<String, Task> = BTreeMap::new();
        expected.insert(expected_key, expected_task);

        // when
        let task = Task::from_yml(CONFIG_ONLY_CMD_PRESENT.into()).unwrap();

        //then
        assert_eq!(expected, task);
    }

    #[test]
    fn umask_unvalid_1_should_return_error() {
        //given && when
        let task = Task::from_yml(MASK_NOT_VALID_1.into());

        //then
        assert!(task.is_err());
        if let Err(error) = task {
            assert!(error
                .to_string()
                .contains("Umask should be umask in octal representation"));
        }
    }

    #[test]
    fn umask_unvalid_2_should_return_error() {
        //given && when
        let task = Task::from_yml(MASK_NOT_VALID_2.into());

        //then
        assert!(task.is_err());
        if let Err(error) = task {
            assert!(error
                .to_string()
                .contains("Umask should be umask in octal representation"));
        }
    }

    #[test]
    fn umask_unvalid_3_should_return_error() {
        //given && when
        let task = Task::from_yml(MASK_NOT_VALID_3.into());

        //then
        assert!(task.is_err());
        if let Err(error) = task {
            assert!(error
                .to_string()
                .contains("Umask should be umask in octal representation"));
        }
    }

    #[test]
    fn working_dir_with_spaces_should_be_trimmed() {
        //given
        let mut expected_task = Task::default();
        expected_task.cmd = String::from("cmd");
        expected_task.working_dir = Some(String::from("/tmp"));
        let expected_key = String::from("task1");
        let mut expected: BTreeMap<String, Task> = BTreeMap::new();
        expected.insert(expected_key, expected_task);

        // when
        let task = Task::from_yml(WORKING_DIR_WITH_SPACES.into()).unwrap();

        //then
        assert_eq!(expected, task);
    }

    #[test]
    fn stdout_path_with_spaces_should_be_trimmed() {
        //given
        let mut expected_task = Task::default();
        expected_task.cmd = String::from("cmd");
        expected_task.stdout = Some(String::from("/tmp/task1.stdout"));
        let expected_key = String::from("task1");
        let mut expected: BTreeMap<String, Task> = BTreeMap::new();
        expected.insert(expected_key, expected_task);

        // when
        let task = Task::from_yml(STDOUT_WITH_SPACES.into()).unwrap();

        //then
        assert_eq!(expected, task);
    }

    #[test]
    fn stderr_path_with_spaces_should_be_trimmed() {
        //given
        let mut expected_task = Task::default();
        expected_task.cmd = String::from("cmd");
        expected_task.stderr = Some(String::from("/tmp/task1.stderr"));
        let expected_key = String::from("task1");
        let mut expected: BTreeMap<String, Task> = BTreeMap::new();
        expected.insert(expected_key, expected_task);

        // when
        let task = Task::from_yml(STDERR_WITH_SPACES.into()).unwrap();

        //then
        assert_eq!(expected, task);
    }
}
