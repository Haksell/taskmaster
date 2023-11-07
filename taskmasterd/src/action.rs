use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum Action {
    Config(String),
    Maintail(Option<usize>),
    Shutdown,
    Signal(u8, String),
    Start(Option<(String, Option<usize>)>),
    Status(Option<String>),
    Stop(Option<(String, Option<usize>)>),
    Update(Option<String>),
}
