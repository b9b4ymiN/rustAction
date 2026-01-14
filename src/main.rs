//! KS Forward Video Processor
//! Automated YouTube transcript summarization with Discord integration

mod config;
mod error;
mod models;
mod services;

use config::Config;
use error::{AppError, Result};
use services::ksForword_service::get_lastest_ksForword;
use tracing::{error, info};
use tracing_subscriber::{
    fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

/// Application entry point
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    init_tracing();

    info!("🚀 Starting KS Forward Video Processor");
    info!("📅 Version: {}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = Config::from_env()?;
    config.validate()?;
    info!("✅ Configuration loaded and validated");

    // Run main processing
    match process(&config).await {
        Ok(_) => {
            info!("✅ Processing completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("❌ Processing failed: {}", e);
            error!("   Category: {}", e.category());
            error!("   Retryable: {}", e.is_retryable());

            // Exit with appropriate error code
            std::process::exit(if e.is_retryable() { 2 } else { 1 });
        }
    }
}

/// Initialize structured logging
fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,schRust=debug,reqwest=warn"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_line_number(true)
                .with_thread_ids(false)
                .with_file(false)
        )
        .init();
}

/// Main processing logic
async fn process(config: &Config) -> Result<()> {
    info!("🎬 Processing latest KS Forward video");

    get_lastest_ksForword(config)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to process KS Forward: {}", e)))?;

    info!("📊 Latest KS Forward video processed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categories() {
        let err = AppError::config("test");
        assert_eq!(err.category(), "config");
        assert!(!err.is_retryable());

        let err = AppError::ApiTimeout { seconds: 30 };
        assert_eq!(err.category(), "api");
        assert!(err.is_retryable());
    }
}
