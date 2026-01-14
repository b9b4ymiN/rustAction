//! Configuration management with validation
use crate::error::{AppError, Result};
use dotenvy::dotenv;
use once_cell::sync::OnceCell;
use std::env;

/// Global configuration instance (lazy-loaded)
static CONFIG: OnceCell<Config> = OnceCell::new();

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// API base URL
    pub api_url: String,
    /// Authentication token
    pub token: String,
    /// YouTube Data API key
    pub youtube_api_key: String,
    /// Supabase/transcript API key
    pub supabase_api_key: String,
    /// KS Forward YouTube channel ID
    pub ksforword_channel_id: String,
    /// Use mock data for testing
    pub use_mock_data: bool,
    /// AI service API URL
    pub my_ai_api_url: String,
    /// AI service API Key (X-API-Key header)
    pub my_ai_api_key: String,
    /// Discord bot webhook URL
    pub discord_ks_bot_token: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenv().ok();

        let api_url = env::var("API_URL")
            .map_err(|_| AppError::config("API_URL must be set"))?;
        let token = env::var("TOKEN")
            .map_err(|_| AppError::config("TOKEN must be set"))?;
        let youtube_api_key = env::var("YOUTUBE_API_KEY")
            .map_err(|_| AppError::config("YOUTUBE_API_KEY must be set"))?;
        let supabase_api_key = env::var("SUPABASE_API_KEY")
            .map_err(|_| AppError::config("SUPABASE_API_KEY must be set"))?;
        let ksforword_channel_id = env::var("KSFORWORD_CHANNEL_ID")
            .map_err(|_| AppError::config("KSFORWORD_CHANNEL_ID must be set"))?;
        let my_ai_api_url = env::var("MY_AI_API_URL")
            .map_err(|_| AppError::config("MY_AI_API_URL must be set"))?;
        let my_ai_api_key = env::var("MY_AI_API_KEY")
            .map_err(|_| AppError::config("MY_AI_API_KEY must be set"))?;
        let discord_ks_bot_token = env::var("DISCORD_KS_BOT_TOKEN")
            .map_err(|_| AppError::config("DISCORD_KS_BOT_TOKEN must be set"))?;

        let use_mock_data = env::var("USE_MOCK_DATA")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            == "true";

        Ok(Self {
            api_url,
            token,
            youtube_api_key,
            supabase_api_key,
            ksforword_channel_id,
            use_mock_data,
            my_ai_api_url,
            my_ai_api_key,
            discord_ks_bot_token,
        })
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        // Validate URLs
        Self::validate_url(&self.api_url, "API_URL")?;
        Self::validate_url(&self.my_ai_api_url, "MY_AI_API_URL")?;
        Self::validate_url(&self.discord_ks_bot_token, "DISCORD_KS_BOT_TOKEN")?;

        // Validate API keys (basic format check)
        if self.youtube_api_key.len() < 10 {
            return Err(AppError::config(
                "YOUTUBE_API_KEY appears to be invalid (too short)",
            ));
        }

        if self.supabase_api_key.is_empty() {
            return Err(AppError::config("SUPABASE_API_KEY cannot be empty"));
        }

        // Validate channel ID format (YouTube channel IDs are typically 24 chars)
        if self.ksforword_channel_id.len() < 10 {
            return Err(AppError::config(
                "KSFORWORD_CHANNEL_ID appears to be invalid (too short)",
            ));
        }

        tracing::debug!("Configuration validation passed");
        Ok(())
    }

    /// Validate URL format
    fn validate_url(url: &str, name: &str) -> Result<()> {
        if url.is_empty() {
            return Err(AppError::config(format!("{} cannot be empty", name)));
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(AppError::config(format!(
                "{} must be a valid URL (starting with http:// or https://)",
                name
            )));
        }

        Ok(())
    }

    /// Get global configuration instance (cached)
    pub fn global() -> &'static Config {
        CONFIG.get_or_init(|| {
            let config = Self::from_env()
                .expect("Failed to load configuration");
            config.validate()
                .expect("Configuration validation failed");
            config
        })
    }

    /// Format configuration for logging (redacts sensitive values)
    pub fn to_safe_string(&self) -> String {
        format!(
            "Config {{ \
             api_url: {}, \
             youtube_api_key: {}, \
             ksforword_channel_id: {}, \
             my_ai_api_url: {}, \
             my_ai_api_key: {}, \
             discord_webhook: {}, \
             use_mock_data: {} \
             }}",
            self.api_url,
            Self::mask_key(&self.youtube_api_key),
            self.ksforword_channel_id,
            self.my_ai_api_url,
            Self::mask_key(&self.my_ai_api_key),
            Self::mask_url(&self.discord_ks_bot_token),
            self.use_mock_data
        )
    }

    /// Mask API key for logging
    fn mask_key(key: &str) -> String {
        if key.len() <= 8 {
            "***".to_string()
        } else {
            format!("{}...{}", &key[..4], &key[key.len() - 4..])
        }
    }

    /// Mask URL for logging
    fn mask_url(url: &str) -> String {
        if let Some(start) = url.find("://") {
            let after_protocol = &url[start + 3..];
            if let Some(end) = after_protocol.find('/') {
                return format!(
                    "{}://{}/***{}",
                    &url[..start],
                    &after_protocol[..end],
                    &after_protocol[after_protocol.len().saturating_sub(20)..]
                );
            }
        }
        "***".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url() {
        assert!(Config::validate_url("https://example.com", "TEST").is_ok());
        assert!(Config::validate_url("http://localhost:8000", "TEST").is_ok());
        assert!(Config::validate_url("invalid-url", "TEST").is_err());
        assert!(Config::validate_url("", "TEST").is_err());
    }

    #[test]
    fn test_mask_key() {
        assert_eq!(Config::mask_key("12345678"), "***");
        assert_eq!(Config::mask_key("ABCDEFGHIJKL"), "ABCD...IJKL");
    }
}
