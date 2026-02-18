//! Configuration and credentials for the Kalshi API client.
//!
//! This module provides the [`Config`] struct for managing API credentials
//! and client settings.

use std::time::Duration;

/// API environment (production or demo)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Environment {
    /// Production environment (real money)
    #[default]
    Production,
    /// Demo environment (paper trading)
    Demo,
}

impl Environment {
    /// Get the base URL for REST API
    pub fn rest_base_url(&self) -> &'static str {
        match self {
            Environment::Production => "https://api.elections.kalshi.com/trade-api/v2",
            Environment::Demo => "https://demo-api.kalshi.co/trade-api/v2",
        }
    }

    /// Get the WebSocket URL
    pub fn websocket_url(&self) -> &'static str {
        match self {
            Environment::Production => "wss://api.elections.kalshi.com/trade-api/ws/v2",
            Environment::Demo => "wss://demo-api.kalshi.co/trade-api/ws/v2",
        }
    }
}

/// Configuration for the Kalshi API client
///
/// # Example
///
/// ```rust
/// use kalshi_rs::Config;
///
/// let config = Config::new("my-api-key-id", "-----BEGIN PRIVATE KEY-----\n...");
///
/// // Use demo environment
/// let demo_config = Config::new("key", "private-key")
///     .with_environment(kalshi_rs::config::Environment::Demo);
///
/// // Custom timeout
/// let config = Config::new("key", "private-key")
///     .with_timeout(std::time::Duration::from_secs(30));
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    /// API key ID (from Kalshi dashboard)
    api_key_id: String,

    /// Private key in PEM format (for RSA-PSS signing)
    private_key_pem: String,

    /// API environment
    environment: Environment,

    /// HTTP request timeout
    timeout: Duration,

    /// Subaccount number (0 for primary account)
    subaccount: Option<u32>,
}

impl Config {
    /// Create a new configuration with API credentials
    ///
    /// # Arguments
    ///
    /// * `api_key_id` - Your API key ID from the Kalshi dashboard
    /// * `private_key_pem` - Your RSA private key in PEM format
    ///
    /// # Example
    ///
    /// ```rust
    /// use kalshi_rs::Config;
    ///
    /// let config = Config::new(
    ///     "abc123",
    ///     "-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----",
    /// );
    /// ```
    pub fn new(api_key_id: impl Into<String>, private_key_pem: impl Into<String>) -> Self {
        Self {
            api_key_id: api_key_id.into(),
            private_key_pem: private_key_pem.into(),
            environment: Environment::default(),
            timeout: Duration::from_secs(10),
            subaccount: None,
        }
    }

    /// Set the API environment (production or demo)
    #[must_use]
    pub fn with_environment(mut self, environment: Environment) -> Self {
        self.environment = environment;
        self
    }

    /// Set the HTTP request timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the subaccount number (1-32, or None for primary)
    #[must_use]
    pub fn with_subaccount(mut self, subaccount: Option<u32>) -> Self {
        self.subaccount = subaccount;
        self
    }

    /// Get the API key ID
    pub fn api_key_id(&self) -> &str {
        &self.api_key_id
    }

    /// Get the private key PEM
    pub fn private_key_pem(&self) -> &str {
        &self.private_key_pem
    }

    /// Get the environment
    pub fn environment(&self) -> Environment {
        self.environment
    }

    /// Get the REST API base URL
    pub fn rest_base_url(&self) -> &'static str {
        self.environment.rest_base_url()
    }

    /// Get the WebSocket URL
    pub fn websocket_url(&self) -> &'static str {
        self.environment.websocket_url()
    }

    /// Get the timeout duration
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Get the subaccount number
    pub fn subaccount(&self) -> Option<u32> {
        self.subaccount
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::new("test-key", "test-pem");
        assert_eq!(config.api_key_id(), "test-key");
        assert_eq!(config.environment(), Environment::Production);
        assert_eq!(config.timeout(), Duration::from_secs(10));
        assert_eq!(config.subaccount(), None);
    }

    #[test]
    fn test_demo_environment() {
        let config = Config::new("key", "pem").with_environment(Environment::Demo);
        assert!(config.rest_base_url().contains("demo"));
        assert!(config.websocket_url().contains("demo"));
    }

    #[test]
    fn test_builder_pattern() {
        let config = Config::new("key", "pem")
            .with_environment(Environment::Demo)
            .with_timeout(Duration::from_secs(30))
            .with_subaccount(Some(1));

        assert_eq!(config.environment(), Environment::Demo);
        assert_eq!(config.timeout(), Duration::from_secs(30));
        assert_eq!(config.subaccount(), Some(1));
    }
}
