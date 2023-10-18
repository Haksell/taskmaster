use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum Autorestart {
    #[serde(rename = "true")]
    True,
    #[serde(rename = "false")]
    False,
    #[serde(rename = "unexpected")]
    Unexpected,
}