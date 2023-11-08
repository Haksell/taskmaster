use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum TailType {
    Stream(Option<usize>),
    Fixed(Option<usize>),
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum OutputType {
    Stdout,
    Stderr,
}

impl Display for OutputType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputType::Stdout => write!(f, "stdout"),
            OutputType::Stderr => write!(f, "stderr"),
        }
    }
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum Action {
    Config(String),
    HttpLogging(bool, Option<u16>),
    Maintail(TailType),
    Shutdown,
    Restart(Option<(String, Option<usize>)>),
    Signal(u8, String),
    Start(Option<(String, Option<usize>)>),
    Status(Option<String>),
    Stop(Option<(String, Option<usize>)>),
    Tail(String, OutputType, TailType),
    Update(Option<String>),
}
