//! Professional error handling module using thiserror
use thiserror::Error;

/// Main application error type
#[derive(Error, Debug)]
pub enum AppError {
    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Network/API errors
    #[error("API request failed to {url}: {status}")]
    ApiError { url: String, status: u16 },

    #[error("API request timeout after {seconds}s")]
    ApiTimeout { seconds: u64 },

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Data parsing errors
    #[error("JSON parse error at {location}: {message}")]
    JsonParse { location: String, message: String },

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    /// YouTube specific errors
    #[error("YouTube API error: {0}")]
    YouTube(String),

    #[error("Transcript not found for video: {video_id}")]
    TranscriptNotFound { video_id: String },

    /// AI service errors
    #[error("AI service error: {0}")]
    AIService(String),

    #[error("AI response parsing failed: {0}")]
    AIParse(String),

    /// Discord errors
    #[error("Discord webhook failed: {status}")]
    Discord { status: u16 },

    #[error("Discord message too long: {length} chars (max: {max})")]
    MessageTooLong { length: usize, max: usize },

    /// Cache errors
    #[error("Cache error: {0}")]
    Cache(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic errors
    #[error("Internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// Create a configuration error
    pub fn config(message: impl Into<String>) -> Self {
        AppError::Config {
            message: message.into(),
        }
    }

    /// Create a YouTube error
    pub fn youtube(message: impl Into<String>) -> Self {
        AppError::YouTube(message.into())
    }

    /// Create an AI service error
    pub fn ai_service(message: impl Into<String>) -> Self {
        AppError::AIService(message.into())
    }

    /// Create a cache error
    pub fn cache(message: impl Into<String>) -> Self {
        AppError::Cache(message.into())
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AppError::ApiTimeout { .. }
                | AppError::Network(_)
                | AppError::ApiError { status: 500..=599, .. }
                | AppError::ApiError { status: 429, .. }
        )
    }

    /// Get error category for logging/metrics
    pub fn category(&self) -> &'static str {
        match self {
            AppError::Config { .. } => "config",
            AppError::ApiError { .. } | AppError::ApiTimeout { .. } => "api",
            AppError::Network(_) => "network",
            AppError::JsonParse { .. } | AppError::InvalidResponse(_) => "parse",
            AppError::YouTube(_) | AppError::TranscriptNotFound { .. } => "youtube",
            AppError::AIService(_) | AppError::AIParse(_) => "ai_service",
            AppError::Discord { .. } | AppError::MessageTooLong { .. } => "discord",
            AppError::Cache(_) => "cache",
            AppError::Io(_) => "io",
            AppError::Internal(_) => "internal",
        }
    }
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, AppError>;
