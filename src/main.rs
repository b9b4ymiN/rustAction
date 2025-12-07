mod config;
mod models;
mod services;

use config::Config;
use tokio;

use crate::{
    models::youtube_snippet::SearchResult, services::ksForword_service::get_lastest_ksForword,
};
use services::youtube_service::get_youtube_search;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;

    get_lastest_ksForword(&config).await?;

    Ok(())
}
