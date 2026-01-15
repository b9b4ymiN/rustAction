//! Discord webhook service with professional logging and error handling
use crate::{
    config::Config,
    models::discord::{DiscordEmbed, DiscordFooter, DiscordWebhook},
};
use chrono::{Local, Datelike, Timelike};
use reqwest::Client;
use serde_json::Value;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

/// Format chrono DateTime to Discord-compatible ISO8601 timestamp (millisecond precision)
fn format_discord_timestamp(dt: &chrono::DateTime<Local>) -> String {
    // Discord expects ISO8601 with millisecond precision: 2024-01-14T12:00:00.123+07:00
    let offset = dt.offset().local_minus_utc();
    let sign = if offset >= 0 { "+" } else { "-" };
    let offset_abs = offset.abs();
    let hours = offset_abs / 3600;
    let minutes = (offset_abs % 3600) / 60;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}{}{:02}:{:02}",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
        dt.timestamp_subsec_millis(),
        sign,
        hours,
        minutes
    )
}

/// Send a message to Discord webhook with professional logging
///
/// # Errors
/// Returns an error if:
/// - Configuration is invalid
/// - Discord API request fails (after retries)
/// - Discord returns 4xx error (client error)
/// - Discord returns 500 error after all retries
pub async fn send_message(title: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let now = Local::now();

    info!("📤 Preparing to send message to Discord");
    info!("   Title: {}", title);
    info!("   Message byte length: {}", message.len());
    info!("   Message char length: {}", message.chars().count());

    // Try to parse message as JSON and extract "answer" field if it exists
    let clean_message = extract_clean_message(message);

    info!("✓ Clean message length: {} chars", clean_message.len());
    info!("   Preview: {}", &clean_message.chars().take(100).collect::<String>());

    // Build embeds and split long messages into multiple embeds if needed
    let embeds = build_embeds(title, &clean_message, now);

    info!("📦 Created {} embed(s)", embeds.len());

    let discord_webhook_url = Config::from_env()?.discord_ks_bot_token;

    // Discord accepts up to 10 embeds per webhook request
    let total_batches = (embeds.len() + 9) / 10;
    info!("🚀 Sending to Discord in {} batch(es)", total_batches);

    for (batch_idx, batch) in embeds.chunks(10).enumerate() {
        let batch_num = batch_idx + 1;
        let batch_embeds: Vec<DiscordEmbed> = batch
            .iter()
            .map(|e| DiscordEmbed {
                title: e.title.clone(),
                description: e.description.clone(),
                color: e.color,
                timestamp: e.timestamp.clone(),
                footer: e.footer.as_ref().map(|f| DiscordFooter {
                    text: f.text.clone(),
                }),
            })
            .collect();

        let webhook = DiscordWebhook {
            content: None,
            embeds: Some(batch_embeds),
        };

        // Log the payload for debugging (only on first batch to avoid spam)
        if batch_num == 1 {
            let json_payload = serde_json::to_string_pretty(&webhook).unwrap_or_default();
            info!("📋 Discord webhook payload:");
            //info!("{}", json_payload);
        }

        // Retry logic for transient Discord errors
        let max_retries = 3;
        let mut attempt = 0;
        let mut last_error = None;

        loop {
            attempt += 1;
            info!(
                "📤 Sending batch {}/{} (attempt {}/{})",
                batch_num, total_batches, attempt, max_retries
            );

            match send_discord_request(&client, &discord_webhook_url, &webhook).await {
                Ok(status) if status.is_success() => {
                    info!("✅ Discord batch {} accepted (status: {})", batch_num, status);
                    break;
                }
                Ok(status) => {
                    // Discord returned an error status
                    let is_retryable = status.is_server_error() || status.as_u16() == 429; // 5xx or 429 (rate limit)

                    if is_retryable && attempt < max_retries {
                        warn!(
                            "⚠️  Discord returned {} for batch {} - retrying (attempt {}/{})",
                            status, batch_num, attempt, max_retries
                        );
                        let backoff = Duration::from_millis(1000 * attempt as u64);
                        sleep(backoff).await;
                        continue;
                    } else {
                        // Client error (4xx) or exhausted retries
                        error!(
                            "❌ Discord webhook failed for batch {}",
                            batch_num
                        );
                        error!("   Status: {}", status);
                        error!("   URL: {}", mask_webhook_url(&discord_webhook_url));

                        if status.is_client_error() {
                            error!("   Type: Client error (4xx) - check webhook URL and permissions");
                        } else {
                            error!("   Type: Server error (5xx) - Discord service issue");
                        }

                        return Err(format!(
                            "Discord webhook failed with status {} for batch {}",
                            status, batch_num
                        ).into());
                    }
                }
                Err(e) => {
                    // Network/transport error
                    last_error = Some(e.to_string());

                    if attempt < max_retries {
                        warn!(
                            "⚠️  Network error sending batch {} (attempt {}/{}): {}",
                            batch_num, attempt, max_retries, e
                        );
                        let backoff = Duration::from_millis(1000 * attempt as u64);
                        sleep(backoff).await;
                        continue;
                    } else {
                        // Extract error once to avoid ownership issues
                        let error_msg = last_error.clone().unwrap_or_default();
                        error!(
                            "❌ Failed to send to Discord after {} attempts: {}",
                            max_retries, error_msg
                        );
                        error!("   Batch: {}", batch_num);
                        error!("   URL: {}", mask_webhook_url(&discord_webhook_url));

                        return Err(format!(
                            "Failed to send to Discord after {} retries: {}",
                            max_retries, error_msg
                        ).into());
                    }
                }
            }
        }
    }

    info!("✅ All messages sent to Discord successfully");
    Ok(())
}

/// Extract clean message content, handling JSON and plain text
fn extract_clean_message(message: &str) -> String {
    let trimmed = message.trim();

    // Check if message is wrapped in markdown code fence
    let json_str = if trimmed.starts_with("```") {
        if let Some(first_newline_pos) = trimmed.find('\n') {
            let content_start = first_newline_pos + 1;
            if let Some(last_fence_pos) = trimmed.rfind("\n```") {
                if last_fence_pos > content_start {
                    trimmed[content_start..last_fence_pos].to_string()
                } else {
                    trimmed.to_string()
                }
            } else {
                trimmed.to_string()
            }
        } else {
            trimmed.to_string()
        }
    } else {
        trimmed.to_string()
    };

    // Try to parse as JSON and extract "answer" field
    match serde_json::from_str::<Value>(&json_str) {
        Ok(json_val) => {
            info!("✓ Parsed message as JSON successfully");
            if let Some(answer) = json_val.get("answer").and_then(|v| v.as_str()) {
                info!("✓ Found 'answer' field, length: {}", answer.len());
                answer.to_string()
            } else {
                info!("✗ No 'answer' field found in JSON, using raw content");
                info!("   JSON keys: {:?}", json_val.as_object().map(|o| o.keys().collect::<Vec<_>>()));
                json_str
            }
        }
        Err(e) => {
            warn!("✗ Failed to parse as JSON: {}", e);
            // If JSON parsing fails, return user-friendly error message
            if json_str.len() > 100 && json_str.contains("\"thought\"") {
                "⚠️ เกิดข้อผิดพลาดในการประมวลผล AI response (JSON parsing failed)\n\nกรุณาลองใหม่อีกครั้ง".to_string()
            } else {
                json_str.to_string()
            }
        }
    }
}

/// Build Discord embeds from message, splitting if necessary
fn build_embeds(title: &str, message: &str, now: chrono::DateTime<Local>) -> Vec<DiscordEmbed> {
    const MAX_DESC: usize = 4000; // Safe limit for Discord embed description (Discord limit is 4096)

    let chars: Vec<char> = message.chars().collect();
    let total = if chars.is_empty() {
        0
    } else {
        (chars.len() + MAX_DESC - 1) / MAX_DESC
    };

    let mut embeds: Vec<DiscordEmbed> = Vec::new();

    for i in 0..total {
        let start = i * MAX_DESC;
        let end = ((i + 1) * MAX_DESC).min(chars.len());
        let part: String = chars[start..end].iter().collect();
        let part_bytes = part.len();
        let part_chars = part.chars().count();

        // Log for debugging first embed only
        if i == 0 {
            info!("📊 Embed 1: {} chars, {} bytes (max: {})", part_chars, part_bytes, MAX_DESC);
        }

        let display_title = if total > 1 {
            format!("{} ({}/{})", title, i + 1, total)
        } else {
            title.to_string()
        };

        embeds.push(DiscordEmbed {
            title: display_title,
            description: part,
            color: 0x5865F2, // Discord Blurple
            timestamp: format_discord_timestamp(&now),
            footer: Some(DiscordFooter {
                text: "KS Forward".to_string(),
            }),
        });
    }

    // Fallback: if message was empty, send a minimal embed
    if embeds.is_empty() {
        embeds.push(DiscordEmbed {
            title: "Daily Summary".to_string(),
            description: message.to_string(),
            color: 0x5865F2,
            timestamp: format_discord_timestamp(&now),
            footer: Some(DiscordFooter {
                text: "KS Forward".to_string(),
            }),
        });
    }

    embeds
}

/// Send a single request to Discord webhook
async fn send_discord_request(
    client: &Client,
    url: &str,
    webhook: &DiscordWebhook,
) -> Result<reqwest::StatusCode, reqwest::Error> {
    // Note: .json() automatically sets Content-Type: application/json
    let response = client
        .post(url)
        .json(webhook)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    // Log response details
    if status.is_success() {
        info!("   Discord response: {}", status);
    } else {
        error!("   Discord response: {}", status);
        error!("   Response body: {}", body);
    }

    Ok(status)
}

/// Mask webhook URL for logging (hide sensitive parts)
fn mask_webhook_url(url: &str) -> String {
    if let Some(last_slash) = url.rfind('/') {
        let first_part = &url[..last_slash];
        format!("{}/***", first_part)
    } else {
        "***".to_string()
    }
}
