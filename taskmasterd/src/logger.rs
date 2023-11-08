use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

const MONITOR_THREAD_PREFIX: &'static str = "MONITOR THREAD";
const MONITOR_PREFIX: &'static str = "    MONITOR   ";
const RESPONDER_PREFIX: &'static str = "   RESPONDER  ";
const GLOBAL_PREFIX: &'static str = "  RUSTMASTER  ";
const MAX_MESSAGES: usize = 1000;
const BUFFER_SIZE: usize = MAX_MESSAGES * 6 / 5;

pub type LogLine = (usize, String);

pub struct Logger {
    pub history: VecDeque<LogLine>,
    file: File,
    idx: usize,
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

    pub fn get_history(&self, num_lines: Option<usize>) -> Vec<String> {
        self.history
            .iter()
            .rev()
            .take(num_lines.unwrap_or(self.history.len()))
            .rev()
            .map(|(_, message)| message.to_string())
            .collect()
    }

    pub fn new(file_path: &'static str) -> Result<Self, String> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_path)
            .map_err(|e| format!("Can't create logging file: {file_path}. Error: {e}"))?;
        Ok(Logger {
            history: VecDeque::with_capacity(BUFFER_SIZE),
            file,
            idx: 0,
        })
    }

    fn do_log(&mut self, prefix: &'static str, message: &str) {
        let log_msg = format!(
            "[{prefix}]: {}{:?}\n",
            Logger::get_timestamp(),
            message.trim()
        );
        print!("{log_msg}");
        if let Err(e) = self.file.write_all(log_msg.as_bytes()) {
            eprintln!("Error! Can't write log {message} in log file: {e}")
        }
        if prefix != RESPONDER_PREFIX {
            self.idx = self.idx.wrapping_add(1);
            self.history.push_back((self.idx, log_msg));
            if self.history.len() > (BUFFER_SIZE as f32 * 0.95) as usize {
                self.history.drain(..(self.history.len() - MAX_MESSAGES));
            }
        }
    }

    pub fn sth_log(&mut self, message: String) -> String {
        self.do_log(MONITOR_THREAD_PREFIX, &message);
        message
    }

    pub fn monit_log(&mut self, message: String) -> String {
        self.do_log(MONITOR_PREFIX, &message);
        message
    }

    pub fn log<S: AsRef<str>>(&mut self, message: S) {
        self.do_log(GLOBAL_PREFIX, message.as_ref());
    }

    pub fn resp_log<S: AsRef<str>>(&mut self, message: S) {
        self.do_log(RESPONDER_PREFIX, message.as_ref());
    }

    pub fn log_err<S: AsRef<str>>(&self, message: S) {
        eprintln!("{}", message.as_ref())
    }
}
