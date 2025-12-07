use crate::config::{self};

use crate::models::youtube_snippet::Root;
use reqwest::Client;

pub async fn get_youtube_search(channel_id: &str) -> Result<Root, Box<dyn std::error::Error>> {
    let url = "https://www.googleapis.com/youtube/v3/search";

    let key = config::Config::from_env()?.youtube_api_key;
    let query_params = [
        ("part", "snippet"),
        ("channelId", channel_id),
        ("maxResults", "5"),
        ("order", "date"),
        ("type", "video"),
        ("key", &key),
        ("eventType", "completed"),
    ];

    let client = Client::new();
    let res = client
        .get(url)
        .query(&query_params)
        .send()
        .await?
        .error_for_status()? // ถ้า status code != 2xx จะ return error
        .json::<Root>()
        .await?;

    Ok(res)
}
