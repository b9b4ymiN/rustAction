mod config;
mod models;
mod services;

use config::Config;
use tokio;

use crate::{
    models::youtube_snippet::SearchResult,
    services::ksForword_service::{get_lastest_ksForword, get_summary_link},
};
use services::youtube_service::get_youtube_search;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;

    get_lastest_ksForword(&config).await?;

    //manul test link
    //let test_link = "https://www.youtube.com/watch?v=snsuWNDhmLc";
    //get_summary_link(&config, test_link).await?;

    Ok(())
}
