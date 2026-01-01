use crate::config::{self};

use crate::models::youtube_transcript::Root;
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;
use std::path::Path;
use tokio::fs;

// Helper function to extract video ID from YouTube URL
fn extract_video_id_from_url(url: &str) -> Option<String> {
    // Handle various YouTube URL formats:
    // - https://www.youtube.com/watch?v=VIDEO_ID
    // - https://youtu.be/VIDEO_ID
    // - https://www.youtube.com/embed/VIDEO_ID
    
    if let Some(pos) = url.find("v=") {
        let start = pos + 2;
        let end = url[start..].find('&').map(|p| start + p).unwrap_or(url.len());
        return Some(url[start..end].to_string());
    }
    
    if let Some(pos) = url.find("youtu.be/") {
        let start = pos + 9;
        let end = url[start..].find('?').map(|p| start + p).unwrap_or(url.len());
        return Some(url[start..end].to_string());
    }
    
    None
}

pub async fn get_youtube_transcript(url: &str) -> Result<Root, Box<dyn std::error::Error>> {
    let supabase_url = "https://api.supadata.ai/v1/transcript";

    if url.trim().is_empty() {
        return Err("youtube url is empty".into());
    }

    // Extract video ID for cache filename
    let video_id = extract_video_id_from_url(url)
        .ok_or("Failed to extract video ID from URL")?;
    
    // Create cache directory if it doesn't exist
    let cache_dir = "transcript_cache";
    fs::create_dir_all(cache_dir).await?;
    
    let cache_file = format!("{}/{}.json", cache_dir, video_id);
    
    // Check if cached file exists
    if Path::new(&cache_file).exists() {
        println!("Loading transcript from cache: {}", cache_file);
        let cached_data = fs::read_to_string(&cache_file).await?;
        let transcript: Root = serde_json::from_str(&cached_data)?;
        return Ok(transcript);
    }
    
    println!("Cache miss - fetching transcript from API for video: {}", video_id);

    let supabase_key = config::Config::from_env()?.supabase_api_key;
    if supabase_key.trim().is_empty() {
        return Err("SUPABASE_API_KEY is empty; set the secret/env before running".into());
    }

    let query_params = [("url", url)];

    let client = Client::new();
    let max_retries = 3usize;
    let mut last_error = String::new();

    for attempt in 1..=max_retries {
        println!(
            "Calling transcript API (attempt {}/{}): {}?url={}",
            attempt, max_retries, supabase_url, url
        );

        let response = client
            .get(supabase_url)
            .header("x-api-key", &supabase_key)
            .query(&query_params)
            .timeout(Duration::from_secs(30))
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();

                if status.is_success() {
                    let transcript: Root = serde_json::from_str(&body)?;
                    
                    // Save to cache file
                    if let Err(e) = fs::write(&cache_file, &body).await {
                        eprintln!("Warning: Failed to save transcript to cache: {}", e);
                    } else {
                        println!("Transcript saved to cache: {}", cache_file);
                    }
                    
                    return Ok(transcript);
                } else {
                    last_error = format!(
                        "Transcript API {} returned {} with body: {}",
                        supabase_url, status, body
                    );
                }
            }
            Err(err) => {
                last_error = format!("Transcript API request error: {}", err);
            }
        }

        if attempt < max_retries {
            let backoff = Duration::from_secs(2) * attempt as u32;
            println!(
                "Retrying transcript API after {:?} due to error: {}",
                backoff, last_error
            );
            sleep(backoff).await;
        }
    }

    Err(last_error.into())
}
