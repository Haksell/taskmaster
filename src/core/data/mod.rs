use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub enum AutoRestart {
    #[serde(rename = "true")]
    True,
    #[serde(rename = "false")]
    False,
    #[serde(rename = "unexpected")]
    Unexpected,
}

#[derive(Debug, PartialEq, Deserialize)]
pub enum StopSignal {
    TERM,
    HUP,
    INT,
    QUIT,
    KILL,
    USR1,
    USR2
}