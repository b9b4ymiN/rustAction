use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Todo {
    #[serde(rename = "userId")]
    pub user_id: u32,

    #[serde(rename = "id")]
    pub id: u32,

    #[serde(rename = "title")]
    pub title: String,

    #[serde(rename = "completed")]
    pub completed: bool,
}
