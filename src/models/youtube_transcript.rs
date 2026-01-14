use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    #[serde(default)]
    pub lang: Option<String>,
    #[serde(default)]
    pub available_langs: Vec<String>,
    pub content: Vec<Content>,  // API returns content as array of objects
}

// Content struct is no longer needed with the new API format
// Kept for backward compatibility if needed elsewhere
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    #[serde(default)]
    pub lang: String,
    pub text: String,
    pub offset: f64,
    pub duration: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transcript {
    pub full_transcript:  String
}