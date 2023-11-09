use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::net::TcpStream;
use std::time::{SystemTime, UNIX_EPOCH};

const MONITOR_THREAD_PREFIX: &'static str = "MONITOR THREAD";
const MONITOR_PREFIX: &'static str = "    MONITOR   ";
const RESPONDER_PREFIX: &'static str = "   RESPONDER  ";
const GLOBAL_PREFIX: &'static str = "  RUSTMASTER  ";
const HTTP_LOGGER_PREFIX: &'static str = " HTTP_LOGGER  ";
const MAX_MESSAGES: usize = 1000;
const BUFFER_SIZE: usize = MAX_MESSAGES * 6 / 5;

pub type LogLine = (usize, String);

pub struct Logger {
    pub history: VecDeque<LogLine>,
    file: File,
    idx: usize,
    http_log_stream: Option<(u16, TcpStream)>,
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
            http_log_stream: None,
        })
    }

    pub fn enable_http_logging(&mut self, port: u16) -> Result<(), String> {
        if let Some(_) = self.http_log_stream {
            return Err("http logging is already enabled".to_string());
        }
        let stream = TcpStream::connect(format!("{}:{}", "localhost", port))
            .map_err(|e| format!("can't connect to localhost:{port}: {e}"))?;
        self.http_log(format!(
            "Connection with localhost:{port} has been established"
        ));
        self.http_log_stream = Some((port, stream));
        let history: Vec<String> = self.history.iter().map(|(_, msg)| msg.clone()).collect();
        for msg in history.iter() {
            if let Err(err_msg) = self.do_log_via_http(msg) {
                return Err(self.http_log(err_msg));
            }
        }
        Ok(())
    }

    pub fn disable_http_logging(&mut self) -> String {
        if self.http_log_stream.is_none() {
            return "http logging is already disabled".to_string();
        }
        self.http_log_stream = None;
        self.http_log(format!("http logging has been disabled"))
    }

    pub fn get_http_logging_status(&mut self) -> String {
        self.http_log("Http logging status is requested".to_string());
        let message = if let Some((port, stream)) = &mut self.http_log_stream {
            match stream.write(&[]) {
                Ok(_) => Ok(format!("enabled localhost:{}", port)),
                Err(err) => Err(format!(
                    "connection with localhost:{} is dead: {}",
                    port, err
                )),
            }
        } else {
            Ok("disabled".to_string())
        };
        let final_msg = match message {
            Ok(msg) => msg,
            Err(msg) => {
                self.http_log_stream = None;
                msg
            }
        };
        self.http_log(final_msg)
    }

    fn do_log_via_http(&mut self, body: &str) -> Result<(), String> {
        if let Some((port, stream)) = &mut self.http_log_stream {
            let request = format!(
                "POST /e28d4bc5-666f-4b91-92a4-b46547c6a1cd HTTP/1.1\r\n\
                Connection: Keep-Alive
                Host: {} \r\n\
                Content-Type: application/x-www-form-urlencoded\r\n\
                Content-Length: {} \r\n\
                \r\n\
                {}",
                "localhost",
                body.len(),
                body
            );

            if let Err(err) = stream.write_all(request.as_bytes()) {
                let err_msg = format!("can't write log in localhost:{port}: {err}, disabling...");
                self.http_log_stream = None;
                return Err(err_msg);
            }
            Ok(())
        } else {
            Err("can't log, http logging is disabled".to_string())
        }
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
            self.history.push_back((self.idx, log_msg.clone()));
            if self.history.len() > (BUFFER_SIZE as f32 * 0.95) as usize {
                self.history.drain(..(self.history.len() - MAX_MESSAGES));
            }
        }
        if self.http_log_stream.is_some()
            && prefix != HTTP_LOGGER_PREFIX
            && prefix != RESPONDER_PREFIX
        {
            if let Err(err_msg) = self.do_log_via_http(&log_msg) {
                println!("[{HTTP_LOGGER_PREFIX}]: {err_msg}")
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

    pub fn http_log(&mut self, message: String) -> String {
        self.do_log(HTTP_LOGGER_PREFIX, &message);
        message
    }

    pub fn log_err<S: AsRef<str>>(&self, message: S) {
        eprintln!("{}", message.as_ref())
    }
}
