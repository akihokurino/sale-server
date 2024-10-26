use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EventData {
    #[serde(rename = "Records")]
    pub records: Vec<Record>,
}
#[derive(Serialize, Deserialize)]
pub struct Record {
    #[serde(rename = "Sns")]
    pub sns: Sns,
}
#[derive(Serialize, Deserialize)]
pub struct Sns {
    #[serde(rename = "Message")]
    pub message: String,
}
