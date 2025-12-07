use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub answer: String,
    pub events: Vec<Event>,
    #[serde(rename = "session_id")]
    pub session_id: String,
    #[serde(rename = "context_used")]
    pub context_used: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub step: i64,
    pub agent: String,
    pub action: String,
    pub tool: Value,
    #[serde(rename = "target_agent")]
    pub target_agent: Value,
    pub thought: String,
}
