use crate::config::{self};

use crate::models::youtube_transcript::Root;
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;

pub async fn get_youtube_transcript(url: &str) -> Result<Root, Box<dyn std::error::Error>> {
    let supabase_url = "https://api.supadata.ai/v1/transcript";

    if url.trim().is_empty() {
        return Err("youtube url is empty".into());
    }

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
