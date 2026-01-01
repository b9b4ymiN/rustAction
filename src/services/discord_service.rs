use crate::{
    config::{self, Config},
    models::discord::{self, DiscordEmbed, DiscordFooter, DiscordWebhook},
};
use chrono::Local;
use reqwest::Client;
use serde_json::Value;

pub async fn send_message(title: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let now = Local::now();

    // Try to parse message as JSON and extract "answer" field if it exists
    let clean_message = {
        let trimmed = message.trim();

        // Check if message is wrapped in markdown code fence (handle newlines properly)
        let json_str = if trimmed.starts_with("```") {
            let lines: Vec<&str> = trimmed.lines().collect();
            // Check if first line is fence and last line is closing fence
            if lines.len() > 2 && lines[0].starts_with("```") && lines[lines.len() - 1] == "```" {
                // Join lines between fences (skip first and last)
                lines[1..lines.len()-1].join("\n")
            } else {
                trimmed.to_string()
            }
        } else {
            trimmed.to_string()
        };

        // Try to parse as JSON and extract "answer" field
        match serde_json::from_str::<Value>(&json_str) {
            Ok(json_val) => {
                println!("✓ Parsed message as JSON successfully");
                if let Some(answer) = json_val.get("answer").and_then(|v| v.as_str()) {
                    println!("✓ Found 'answer' field, length: {}", answer.len());
                    answer.to_string()
                } else {
                    println!("✗ No 'answer' field found in JSON");
                    println!("JSON keys: {:?}", json_val.as_object().map(|o| o.keys().collect::<Vec<_>>()));
                    json_str
                }
            }
            Err(e) => {
                println!("✗ Failed to parse as JSON: {}", e);
                println!("First 300 chars of content: {}", &json_str.chars().take(300).collect::<String>());
                json_str
            }
        }
    };

    println!("Clean message length: {}", clean_message.len());
    println!(
        "Clean message preview: {}",
        &clean_message.chars().take(200).collect::<String>()
    );

    // Build embeds and split long messages into multiple embeds if needed.
    // Discord limits embed description to ~4096 characters; we use a safe limit.
    let max_desc = 4000usize;
    let chars: Vec<char> = clean_message.chars().collect();
    let total = if chars.is_empty() {
        0
    } else {
        (chars.len() + max_desc - 1) / max_desc
    };

    let mut embeds: Vec<DiscordEmbed> = Vec::new();
    for i in 0..total {
        let start = i * max_desc;
        let end = ((i + 1) * max_desc).min(chars.len());
        let part: String = chars[start..end].iter().collect();

        embeds.push(DiscordEmbed {
            title: title.to_string(),
            description: part,
            color: 0x5865F2, // Discord Blurple
            timestamp: now.to_rfc3339(),
            footer: Some(DiscordFooter {
                text: "KS Forward".to_string(),
            }),
        });
    }

    // Fallback: if message was empty, still send a minimal embed with the provided message.
    if embeds.is_empty() {
        embeds.push(DiscordEmbed {
            title: "Daily Summary".to_string(),
            description: clean_message.clone(),
            color: 0x5865F2,
            timestamp: now.to_rfc3339(),
            footer: Some(DiscordFooter {
                text: "KS Forward".to_string(),
            }),
        });
    }

    let discord_webhook_url = Config::from_env()?.discord_ks_bot_token;

    // Discord accepts up to 10 embeds in a single webhook request. Send in batches if needed.
    for (batch_idx, batch) in embeds.chunks(10).enumerate() {
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

        let response = client
            .post(&discord_webhook_url)
            .header("Content-Type", "application/json")
            .json(&webhook)
            .send()
            .await?;

        print!(
            "Discord batch {} status: {}",
            batch_idx + 1,
            response.status()
        );
        println!(
            " Discord response body: {}",
            response.text().await.unwrap_or_default()
        );
    }

    Ok(())
}
