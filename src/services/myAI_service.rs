use reqwest::Client;

use crate::{config::Config, models::myAI_response::Root};

use serde_json::json;

pub async fn chat_with_ai(
    config: &Config,
    content: String,
) -> Result<Root, Box<dyn std::error::Error>> {
    let myAI_url = &config.my_ai_api_url;
    let client = Client::new();
    let body = json!({
        "persona": "ks-discord",
        "user_id": "ks-discord",
        "messages": [
            {
                "role": "user",
                "content": content
            }
        ]
    });
    let res = client
        .post(myAI_url)
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?
        .error_for_status()? // ถ้า status code != 2xx จะ return error
        .json::<Root>()
        .await?;

    Ok(res)
}
