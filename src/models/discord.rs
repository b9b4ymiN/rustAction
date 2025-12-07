use serde::Serialize;

/// Discord Webhook Payload
#[derive(Debug, Serialize)]
pub struct DiscordWebhook {
    pub content: Option<String>,
    pub embeds: Option<Vec<DiscordEmbed>>,
}

/// Discord Embed
#[derive(Debug, Serialize)]
pub struct DiscordEmbed {
    pub title: String,
    pub description: String,
    pub color: u32,
    pub timestamp: String,
    pub footer: Option<DiscordFooter>,
}

/// Discord Footer
#[derive(Debug, Serialize)]
pub struct DiscordFooter {
    pub text: String,
}
