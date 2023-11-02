use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

const LOGGER_STATUS_VARIABLE_NAME: &'static str = "RUST_LOGGER_ENABLED";
const LOGGER_ENABLED: &'static str = "1";
pub struct Logger {
    enabled: bool,
}

impl Logger {
    fn get_timestamp() -> String {
        let now = SystemTime::now();
        let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap();
        let now_in_sec = since_the_epoch.as_secs();
        let hours = (now_in_sec % (24 * 3600)) / 3600;
        let minutes = (now_in_sec % 3600) / 60;
        let seconds = now_in_sec % 60;
        format!("[{:02}:{:02}:{:02}]: ", hours, minutes, seconds)
    }

    pub fn new() -> Self {
        let enabled = env::var(LOGGER_STATUS_VARIABLE_NAME)
            .ok()
            .map(|s| s == LOGGER_ENABLED)
            .unwrap_or(false);
        Logger { enabled }
    }

    pub fn log<S: AsRef<str>>(&self, message: S) {
        if self.enabled {
            println!("{}{:?}", Logger::get_timestamp(), message.as_ref());
        }
    }

    pub fn log_err<S: AsRef<str>>(&self, message: S) {
        if self.enabled {
            println!("{}{:?}", Logger::get_timestamp(), message.as_ref());
        } else {
            eprintln!("{}", message.as_ref())
        }
    }

    pub fn log_with_prefix<S: AsRef<str>>(&self, prefix: S, message: S) {
        if self.enabled {
            println!(
                "[{}] {}{:?}",
                prefix.as_ref(),
                Logger::get_timestamp(),
                message.as_ref()
            );
        }
    }

    pub fn enable() -> Logger {
        env::set_var(LOGGER_STATUS_VARIABLE_NAME, LOGGER_ENABLED);
        Logger::new()
    }
}
