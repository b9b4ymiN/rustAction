use dotenvy::dotenv;
use std::env;

pub struct Config {
    pub api_url: String,
    pub token: String,
    pub youtube_api_key: String,
    pub supabase_api_key: String,
    pub ksforword_channel_id: String,
    pub use_mock_data: bool,
    pub my_ai_api_url: String,
    pub discord_ks_bot_token: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        // โหลดไฟล์ .env ถ้ามี
        dotenv().ok();

        let api_url = env::var("API_URL")?;
        let token = env::var("TOKEN")?;
        let youtube_api_key = env::var("YOUTUBE_API_KEY")?;
        let supabase_api_key = env::var("SUPABASE_API_KEY")?;
        let ksforword_channel_id = env::var("KSFORWORD_CHANNEL_ID")?;
        let use_mock_data = env::var("USE_MOCK_DATA")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            == "true";

        let my_ai_api_url = env::var("MY_AI_API_URL")?;
        let discord_ks_bot_token = env::var("DISCORD_KS_BOT_TOKEN")?;
            
        Ok(Self {
            api_url,
            token,
            youtube_api_key,
            supabase_api_key,
            ksforword_channel_id,
            use_mock_data,
            my_ai_api_url,
            discord_ks_bot_token,
        })
    }
}
