use std::fs::Permissions;

use reqwest::Client;
use tokio::time::{sleep, Duration};

use crate::{config::Config, models::myAI_response::Root};

use serde_json::json;

/// Legacy method - sends simple text content to AI service
/// Deprecated: Use `chat_with_ai_v2` instead
pub async fn chat_with_ai(
    config: &Config,
    content: String,
) -> Result<Root, Box<dyn std::error::Error>> {
    println!("Sending to myAI API (legacy format)");
    let myAI_url = &config.my_ai_api_url;
    let api_key = &config.my_ai_api_key;
    println!("myAI_url: {}", myAI_url);
    //println!("API Key: {}", api_key); // For debugging; remove in production

    // Log content size to help debug
    let content_len = content.chars().count();
    println!("Request content length: {} chars", content_len);
    if content_len > 10000 {
        println!(
            "WARNING: Content is very long ({} chars), this may cause issues",
            content_len
        );
        println!(
            "Content preview (first 200 chars): {}",
            &content.chars().take(200).collect::<String>()
        );
    }

    // Truncate content if it's too long
    const MAX_CONTENT_LENGTH: usize = 100000;
    let processed_content = if content_len > MAX_CONTENT_LENGTH {
        println!(
            "Truncating content from {} to {} chars",
            content_len, MAX_CONTENT_LENGTH
        );
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

    // Retry policy
    let max_retries = 3usize;
    let mut attempt = 0usize;

    loop {
        attempt += 1;
        let resp_result = client
            .post(myAI_url)
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header("X-API-Key", api_key) // ✅ Added X-API-Key header
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
                        eprintln!("=== Server returned plain text instead of JSON ===");
                        eprintln!("URL: {}", url);
                        eprintln!("Response length: {} chars", text.len());
                        eprintln!(
                            "Response preview (first 200 chars): {}",
                            &text.chars().take(200).collect::<String>()
                        );
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
                                eprintln!(
                                    "=== JSON extracted from response with trailing characters ==="
                                );
                                return Ok(root);
                            }

                            // Enhanced error logging
                            eprintln!("=== JSON Parse Error ===");
                            eprintln!("Status: {}", status);
                            eprintln!("URL: {}", url);
                            eprintln!("Response body length: {} bytes", text.len());
                            eprintln!(
                                "Response body preview (first 200 chars): {}",
                                &text.chars().take(200).collect::<String>()
                            );
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
                    // 5xx — retry after backoff
                    eprintln!(
                        "myAI API returned server error ({}). attempt {}/{}. url: {}\nheaders: {:?}\nbody: {}",
                        status, attempt, max_retries, url, headers, text
                    );
                    let backoff = Duration::from_millis(500 * (attempt as u64));
                    sleep(backoff).await;
                    continue;
                } else {
                    // Client error (4xx) or server error after retries
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

/// New V2 API method - uses updated API format with X-API-Key header and structured content
///
/// # Example
/// ```bash
/// curl -X POST http://localhost:8000/chat \
///   -H "Content-Type: application/json" \
///   -H "X-API-Key: dev-key-123456789" \
///   -d '{
///     "persona": "oi-trader",
///     "messages": [
///       {"role": "user", "content": [{"type": "text", "text": "Analyze BTC trend"}]}
///     ]
///   }'
/// ```
pub async fn chat_with_ai_v2(
    config: &Config,
    persona: &str,
    content: &str,
) -> Result<Root, Box<dyn std::error::Error>> {
    println!("Sending to myAI API v2");
    let myAI_url = &config.my_ai_api_url;
    let api_key = &config.my_ai_api_key;
    println!("myAI_url: {}", myAI_url);
    println!("persona: {}", persona);

    // Log content size
    let content_len = content.chars().count();
    println!("Request content length: {} chars", content_len);
    if content_len > 10000 {
        println!(
            "WARNING: Content is very long ({} chars), this may cause issues",
            content_len
        );
    }

    // Truncate content if needed
    const MAX_CONTENT_LENGTH: usize = 100000;
    let processed_content = if content_len > MAX_CONTENT_LENGTH {
        println!(
            "Truncating content from {} to {} chars",
            content_len, MAX_CONTENT_LENGTH
        );
        content.chars().take(MAX_CONTENT_LENGTH).collect::<String>()
    } else {
        content.to_string()
    };

    let client = Client::new();

    // New API format with structured content
    let body = json!({
        "persona": persona,
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": processed_content
                    }
                ]
            }
        ]
    });

    println!(
        "Request body: {}",
        serde_json::to_string_pretty(&body).unwrap_or_default()
    );

    // Retry policy
    let max_retries = 3usize;
    let mut attempt = 0usize;

    loop {
        attempt += 1;
        println!("Sending request (attempt {}/{})...", attempt, max_retries);
        println!("API Key: {}", api_key);
        let resp_result = client
            .post(myAI_url)
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header("X-API-Key", api_key)
            .json(&body)
            .send()
            .await;

        match resp_result {
            Ok(resp) => {
                let status = resp.status();
                let url = resp.url().clone();
                let headers = resp.headers().clone();
                let text = resp.text().await.unwrap_or_default();

                println!("Response status: {}", status);
                println!("Response length: {} chars", text.len());

                if status.is_success() {
                    // Check if response is JSON
                    let trimmed = text.trim();
                    if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
                        eprintln!("=== Server returned non-JSON response ===");
                        eprintln!("URL: {}", url);
                        eprintln!(
                            "Response preview: {}",
                            &text.chars().take(200).collect::<String>()
                        );

                        // Fallback response
                        let fallback_response = Root {
                            answer: text.clone(),
                            events: vec![],
                            session_id: "unknown".to_string(),
                            context_used: false,
                        };
                        return Ok(fallback_response);
                    }

                    // Parse JSON
                    let parsed: Result<Root, _> = serde_json::from_str(&text);
                    match parsed {
                        Ok(root) => {
                            println!("✅ Successfully parsed AI response");
                            println!("Session ID: {}", root.session_id);
                            return Ok(root);
                        }
                        Err(parse_err) => {
                            // Try to extract JSON from response with trailing characters
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
                                eprintln!(
                                    "⚠️  Extracted JSON from response with trailing characters"
                                );
                                return Ok(root);
                            }

                            eprintln!("=== JSON Parse Error ===");
                            eprintln!("Status: {}", status);
                            eprintln!("Parse error: {}", parse_err);
                            eprintln!(
                                "Response preview: {}",
                                &text.chars().take(500).collect::<String>()
                            );
                            return Err(
                                format!("Failed to parse AI response: {}", parse_err).into()
                            );
                        }
                    }
                } else if status.is_server_error() && attempt < max_retries {
                    eprintln!(
                        "⚠️  Server error ({}) - retrying (attempt {}/{})",
                        status, attempt, max_retries
                    );
                    let backoff = Duration::from_millis(500 * (attempt as u64));
                    sleep(backoff).await;
                    continue;
                } else if status.as_u16() == 401 {
                    eprintln!("❌ Authentication failed - check MY_AI_API_KEY");
                    return Err(format!(
                        "Authentication failed with status {}. Please check MY_AI_API_KEY.",
                        status
                    )
                    .into());
                } else {
                    return Err(format!(
                        "AI API request failed with status {}. body: {}",
                        status, text
                    )
                    .into());
                }
            }
            Err(err) => {
                if attempt < max_retries {
                    eprintln!(
                        "⚠️  Request error (attempt {}/{}): {}",
                        attempt, max_retries, err
                    );
                    let backoff = Duration::from_millis(500 * (attempt as u64));
                    sleep(backoff).await;
                    continue;
                } else {
                    return Err(format!("AI request failed: {}", err).into());
                }
            }
        }
    }
}

/// Discord-specific method - formats AI response for Discord
/// Uses "ks-discord" persona for optimized Discord message formatting
pub async fn chat_with_ai_msg4Discord(
    config: &Config,
    content: String,
) -> Result<Root, Box<dyn std::error::Error>> {
    chat_with_ai_v2(config, "ks-discord", &content).await
}
