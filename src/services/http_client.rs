//! Optimized HTTP client with connection pooling and performance improvements
use crate::error::{AppError, Result};
use once_cell::sync::Lazy;
use reqwest::{Client, ClientBuilder};
use std::time::Duration;

/// Global HTTP client instance with optimized settings
static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    ClientBuilder::new()
        .pool_max_idle_per_host(10) // Keep up to 10 idle connections per host
        .pool_idle_timeout(Duration::from_secs(90))
        .connect_timeout(Duration::from_secs(10))
        .tcp_keepalive(Duration::from_secs(60))
        .http2_keep_alive_interval(Duration::from_secs(30))
        .http2_keep_alive_timeout(Duration::from_secs(10))
        .http2_keep_alive_while_idle(true)
        .tcp_nodelay(true) // Disable Nagle's algorithm for lower latency
        .connection_verbose(false)
        .build()
        .expect("Failed to build HTTP client")
});

/// Get the global HTTP client instance
pub fn client() -> &'static Client {
    &HTTP_CLIENT
}

/// Build a custom client with specific timeout settings
pub fn build_client(timeout_secs: u64) -> Result<Client> {
    ClientBuilder::new()
        .pool_max_idle_per_host(5)
        .pool_idle_timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(timeout_secs))
        .tcp_keepalive(Duration::from_secs(30))
        .tcp_nodelay(true)
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to build client: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_instance() {
        let client = client();
        assert!(client.timeout().is_some()); // Verify client is configured
    }

    #[tokio::test]
    async fn test_build_client() {
        let client = build_client(30).unwrap();
        assert!(client.timeout().is_some());
    }
}
