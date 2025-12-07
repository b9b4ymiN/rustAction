use crate::config::{self};

use crate::models::youtube_snippet::Root;
use reqwest::Client;

pub async fn get_youtube_search(channel_id: &str) -> Result<Root, Box<dyn std::error::Error>> {
    let url = "https://www.googleapis.com/youtube/v3/search";

    let key = config::Config::from_env()?.youtube_api_key;
    if key.trim().is_empty() {
        return Err("YOUTUBE_API_KEY is empty; set the secret/env before running".into());
    }
    if channel_id.trim().is_empty() {
        return Err("channel_id is empty; set KSFORWORD_CHANNEL_ID before running".into());
    }
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

pub async fn get_detail_byLink(url: &str) -> Result<Root, Box<dyn std::error::Error>> {
    let video_id = extract_video_id(url).await?;
    let key = config::Config::from_env()?.youtube_api_key;
    if key.trim().is_empty() {
        return Err("YOUTUBE_API_KEY is empty; set the secret/env before running".into());
    }
    if video_id.trim().is_empty() {
        return Err("video_id is empty; cannot extract from the provided link".into());
    }

    println!("Extracted video ID: {}", video_id);

    let api_url = "https://www.googleapis.com/youtube/v3/videos";
    let query_params = [
        ("part", "snippet"),
        ("id", video_id.as_str()),
        ("key", &key),
    ];

    let client = Client::new();
    let res = client
        .get(api_url)
        .query(&query_params)
        .send()
        .await?
        .error_for_status()? // ถ้า status code != 2xx จะ return error
        .json::<Root>()
        .await?;

    Ok(res)
}

pub async fn extract_video_id(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = url.trim();
    if url.contains("youtube.com/watch?v=") {
        let parts: Vec<&str> = url.split("v=").collect();
        if parts.len() > 1 {
            let id_part = parts[1];
            let id = id_part.split('&').next().unwrap_or("");
            return Ok(id.to_string());
        }
    } else if url.contains("youtu.be/") {
        let parts: Vec<&str> = url.split("youtu.be/").collect();
        if parts.len() > 1 {
            let id_part = parts[1];
            let id = id_part.split('?').next().unwrap_or("");
            return Ok(id.to_string());
        }
    }
    Err("Could not extract video ID from URL".into())
}
