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

    // Log content size to help debug
    let content_len = content.chars().count();
    println!("Request content length: {} chars", content_len);
    if content_len > 10000 {
        println!("WARNING: Content is very long ({} chars), this may cause issues", content_len);
        println!("Content preview (first 200 chars): {}", &content.chars().take(200).collect::<String>());
    }

    // Truncate content if it's too long (adjust limit as needed for your downstream AI service)
    const MAX_CONTENT_LENGTH: usize = 100000; // Reduced limit - downstream AI service may have strict limits
    let processed_content = if content_len > MAX_CONTENT_LENGTH {
        println!("Truncating content from {} to {} chars", content_len, MAX_CONTENT_LENGTH);
        content.chars().take(MAX_CONTENT_LENGTH).collect::<String>()
    } else {
        content
    };

    let client = Client::new();
    let body = json!({
        "persona": "ks-summary",
        "user_id": "ks-summary",
        "messages": [
            {
                "role": "user",
                "content": processed_content
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
                    // Check if response is actually JSON
                    let trimmed = text.trim();
                    if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
                        // Server returned plain text instead of JSON
                        // This is a server bug, but we can handle it gracefully
                        eprintln!("=== Server returned plain text instead of JSON ===");
                        eprintln!("URL: {}", url);
                        eprintln!("Response length: {} chars", text.len());
                        eprintln!("Response preview (first 200 chars): {}", &text.chars().take(200).collect::<String>());
                        eprintln!("==============================================");

                        // Create a mock Root response with the plain text as the answer
                        let fallback_response = Root {
                            answer: text.clone(),
                            events: vec![],
                            session_id: "unknown".to_string(),
                            context_used: false,
                        };
                        return Ok(fallback_response);
                    }

                    // Parse JSON - handle cases where API returns extra characters after JSON
                    let parsed: Result<Root, _> = serde_json::from_str(&text);
                    match parsed {
                        Ok(root) => return Ok(root),
                        Err(parse_err) => {
                            // Try to extract JSON object from the response
                            // Handle cases where there are trailing characters
                            let fallback_result = if let Some(start) = text.find('{') {
                                if let Some(end) = text.rfind('}') {
                                    let json_str = &text[start..=end];
                                    serde_json::from_str::<Root>(json_str).ok()
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            if let Some(root) = fallback_result {
                                eprintln!("=== JSON extracted from response with trailing characters ===");
                                return Ok(root);
                            }

                            // Enhanced error logging
                            eprintln!("=== JSON Parse Error ===");
                            eprintln!("Status: {}", status);
                            eprintln!("URL: {}", url);
                            eprintln!("Response body length: {} bytes", text.len());
                            eprintln!("Response body preview (first 200 chars): {}", &text.chars().take(200).collect::<String>());
                            eprintln!("Full response body:\n{}", text);
                            eprintln!("Parse error: {}", parse_err);
                            eprintln!("========================");
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

pub async fn chat_with_ai_msg4Discord(
    config: &Config,
    content: String,
) -> Result<Root, Box<dyn std::error::Error>> {
    println!("Sending to discord API");
    let myAI_url = &config.my_ai_api_url;
    println!("discord myAI_url [discord-sum] : {}", myAI_url);

    // Log content size to help debug
    let content_len = content.len();
    println!("Original content length: {} chars", content_len);
    if content_len > 10000 {
        println!("WARNING: Content is very long ({} chars), this may cause issues", content_len);
        println!("Content preview (first 200 chars): {}", &content.chars().take(200).collect::<String>());
    }

    // Truncate content if it's too long (adjust limit as needed for your downstream AI service)
    const MAX_CONTENT_LENGTH: usize = 100000; // Reduced limit - downstream AI service may have strict limits
    let processed_content = if content_len > MAX_CONTENT_LENGTH {
        println!("Truncating content from {} to {} chars", content_len, MAX_CONTENT_LENGTH);
        content.chars().take(MAX_CONTENT_LENGTH).collect::<String>()
    } else {
        content
    };

    let client = Client::new();
    let body = json!({
        "persona": "discord-sum",
        "user_id": "discord-sum",
        "messages": [
            {
                "role": "user",
                "content": processed_content
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
                    // Check if response is actually JSON
                    let trimmed = text.trim();
                    if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
                        // Server returned plain text instead of JSON
                        // This is a server bug, but we can handle it gracefully
                        eprintln!("=== Server returned plain text instead of JSON ===");
                        eprintln!("URL: {}", url);
                        eprintln!("Response length: {} chars", text.len());
                        eprintln!("Response preview (first 200 chars): {}", &text.chars().take(200).collect::<String>());
                        eprintln!("==============================================");

                        // Create a mock Root response with the plain text as the answer
                        let fallback_response = Root {
                            answer: text.clone(),
                            events: vec![],
                            session_id: "unknown".to_string(),
                            context_used: false,
                        };
                        return Ok(fallback_response);
                    }

                    // Parse JSON - handle cases where API returns extra characters after JSON
                    let parsed: Result<Root, _> = serde_json::from_str(&text);
                    match parsed {
                        Ok(root) => return Ok(root),
                        Err(parse_err) => {
                            // Try to extract JSON object from the response
                            // Handle cases where there are trailing characters
                            let fallback_result = if let Some(start) = text.find('{') {
                                if let Some(end) = text.rfind('}') {
                                    let json_str = &text[start..=end];
                                    serde_json::from_str::<Root>(json_str).ok()
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            if let Some(root) = fallback_result {
                                eprintln!("=== JSON extracted from response with trailing characters ===");
                                return Ok(root);
                            }

                            // Enhanced error logging
                            eprintln!("=== JSON Parse Error ===");
                            eprintln!("Status: {}", status);
                            eprintln!("URL: {}", url);
                            eprintln!("Response body length: {} bytes", text.len());
                            eprintln!("Response body preview (first 200 chars): {}", &text.chars().take(200).collect::<String>());
                            eprintln!("Full response body:\n{}", text);
                            eprintln!("Parse error: {}", parse_err);
                            eprintln!("========================");
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
