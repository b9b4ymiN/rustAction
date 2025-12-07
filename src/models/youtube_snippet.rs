use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub video_id: String,
    pub link: String,
    pub title: String,
    pub publish_time: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub kind: String,
    pub etag: String,
    pub next_page_token: Option<String>,
    pub region_code: Option<String>,
    pub page_info: PageInfo,
    pub items: Vec<Item>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub total_results: i64,
    pub results_per_page: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub kind: String,
    pub etag: String,
    pub id: Id,
    pub snippet: Snippet,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Id {
    pub kind: Option<String>,
    pub video_id: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    pub published_at: Option<String>,
    pub channel_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnails: Option<Thumbnails>,
    pub channel_title: Option<String>,
    pub live_broadcast_content: Option<String>,
    pub publish_time: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnails {
    pub default: Option<ThumbnailDefault>,
    pub medium: Option<Medium>,
    pub high: Option<High>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailDefault {
    pub url: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Medium {
    pub url: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct High {
    pub url: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
}
