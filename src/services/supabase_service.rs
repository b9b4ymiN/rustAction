use crate::config::{self};

use crate::models::youtube_transcript::Root;
use reqwest::Client;

pub async fn get_youtube_transcript(url: &str) -> Result<Root, Box<dyn std::error::Error>> {
    let supabase_url = "https://api.supadata.ai/v1/transcript";

    let supabase_key = config::Config::from_env()?.supabase_api_key;

    let query_params = [("url", url)];

    print!("supabase_url = {}", supabase_url);
    print!("youtube link = {}", url);

    let client = Client::new();
    let res = client
        .get(supabase_url)
        .header("x-api-key", &supabase_key)
        .query(&query_params)
        .send()
        .await?
        .error_for_status()? // ถ้า status code != 2xx จะ return error
        .json::<Root>()
        .await?;

    Ok(res)
}
