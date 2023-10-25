use std::fmt;
use std::fmt::{Display, Formatter};
use std::time::{Duration, SystemTime};

#[derive(Clone)]
pub struct LogMessage {
    timestamp: SystemTime,
    message: String,
    return_code: Option<u8>,
}

impl LogMessage {
    fn new(message: &str, return_code: Option<u8>) -> LogMessage {
        LogMessage {
            timestamp: SystemTime::now(),
            message: message.to_string(),
            return_code: return_code,
        }
    }
}

impl Display for LogMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let elapsed = self.timestamp.elapsed().unwrap_or(Duration::new(0, 0));
        write!(f, "[{}] {}", elapsed.as_secs(), self.message)?;
        if let Some(rc) = self.return_code {
            write!(f, " (Return Code: {})", rc)?;
        }
        Ok(())
    }
}

pub struct ErrorLog {
    errors: Vec<LogMessage>,
}

impl ErrorLog {
    pub fn new() -> ErrorLog {
        ErrorLog {
            errors: Vec::new()
        }
    }

    pub fn log(&mut self, message: &str, return_code: Option<u8>) -> LogMessage {
        let error = LogMessage::new(message, return_code);
        self.errors.push(error.clone());
        error
    }

    pub fn read_last_error(&self) -> Option<&LogMessage> {
        self.errors.last()
    }


    pub fn get_errors(&self) -> &Vec<LogMessage> {
        &self.errors
    }
}