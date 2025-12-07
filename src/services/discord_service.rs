use crate::{
    config::{self, Config},
    models::discord::{self, DiscordEmbed, DiscordFooter, DiscordWebhook},
};
use chrono::Local;
use reqwest::Client;

pub async fn send_message(title: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let now = Local::now();
    // Build embeds and split long messages into multiple embeds if needed.
    // Discord limits embed description to ~4096 characters; we use a safe limit.
    let max_desc = 4000usize;
    let chars: Vec<char> = message.chars().collect();
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
            description: message.to_string(),
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
