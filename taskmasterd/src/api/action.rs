use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum Action {
    Config(String),
    Update(String),
    Status(Option<String>),
    Start(String, Option<usize>),
    Stop(String),
    Shutdown,
}
