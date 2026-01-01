use reqwest::Client;
use tokio::time::{sleep, Duration};

use crate::{config::Config, models::myAI_response::Root};

use serde_json::json;

pub async fn chat_with_ai(
    config: &Config,
    content: String,
) -> Result<Root, Box<dyn std::error::Error>> {
    println!("Sending to myAI API");
    let myAI_url = &config.my_ai_api_url;
    println!("myAI_url: {}", myAI_url);

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

    // Retry policy: try a few times for transient server/network errors (5xx or transport errors).
    let max_retries = 3usize;
    let mut attempt = 0usize;

    loop {
        attempt += 1;
        let resp_result = client
            .post(myAI_url)
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await;

        match resp_result {
            Ok(resp) => {
                let status = resp.status();
                let url = resp.url().clone();
                let headers = resp.headers().clone();
                let text = resp.text().await.unwrap_or_default();
                if status.is_success() {
                    // Parse JSON
                    let parsed: Result<Root, _> = serde_json::from_str(&text);
                    match parsed {
                        Ok(root) => return Ok(root),
                        Err(parse_err) => {
                            return Err(format!(
                                "Failed to parse myAI response JSON: {}\nurl: {}\nheaders: {:?}\nbody: {}",
                                parse_err, url, headers, text
                            )
                            .into());
                        }
                    }
                } else if status.is_server_error() && attempt < max_retries {
                    // 5xx — retry after backoff; log more details to help debugging
                    eprintln!(
                        "myAI API returned server error ({}). attempt {}/{}. url: {}\nheaders: {:?}\nbody: {}",
                        status, attempt, max_retries, url, headers, text
                    );
                    let backoff = Duration::from_millis(500 * (attempt as u64));
                    sleep(backoff).await;
                    continue;
                } else {
                    // Client error (4xx) or server error after retries — return a descriptive error
                    return Err(format!(
                        "myAI API request failed with status {}. url: {}\nheaders: {:?}\nbody: {}",
                        status, url, headers, text
                    )
                    .into());
                }
            }
            Err(err) => {
                // Network/transport error; may be transient
                if attempt < max_retries {
                    eprintln!(
                        "myAI request error (attempt {}/{}): {}. retrying...",
                        attempt, max_retries, err
                    );
                    let backoff = Duration::from_millis(500 * (attempt as u64));
                    sleep(backoff).await;
                    continue;
                } else {
                    return Err(format!("myAI request failed: {}", err).into());
                }
            }
        }
    }
}
