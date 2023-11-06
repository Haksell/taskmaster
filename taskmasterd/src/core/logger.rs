use std::time::{SystemTime, UNIX_EPOCH};

pub struct Logger {
    prefix: Option<&'static str>,
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

    pub fn new(prefix: Option<&'static str>) -> Self {
        Logger { prefix }
    }

    pub fn log<S: AsRef<str>>(&self, message: S) -> S {
        match self.prefix.as_ref() {
            None => println!("{}{:?}", Logger::get_timestamp(), message.as_ref()),
            Some(prefix) => println!(
                "[{prefix}] {}{:?}",
                Logger::get_timestamp(),
                message.as_ref()
            ),
        }
        message
    }

    pub fn log_err<S: AsRef<str>>(&self, message: S) {
        eprintln!("{}", message.as_ref())
    }
}
