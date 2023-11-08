use serde::{Deserialize, Serialize};

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

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum Action {
    Config(String),
    Maintail(TailType),
    Tail(String, TailType, OutputType),
    Shutdown,
    Signal(u8, String),
    Start(Option<(String, Option<usize>)>),
    Status(Option<String>),
    Stop(Option<(String, Option<usize>)>),
    Update(Option<String>),
}
