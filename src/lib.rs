//! # kalshi-rs
//!
//! A high-performance Rust client for the [Kalshi](https://kalshi.com) prediction market API.
//!
//! ## Features
//!
//! - **REST API Client** - Full coverage of trading and market data endpoints
//! - **WebSocket Client** - Real-time orderbook, trades, fills, and lifecycle events
//! - **HFT-Grade Orderbook** - Cache-optimized with O(log n) updates
//! - **Async/Await** - Built on Tokio for maximum concurrency
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use kalshi_rs::{Config, KalshiClient};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), kalshi_rs::Error> {
//!     let config = Config::new("api-key-id", "private-key-pem-contents");
//!     let client = KalshiClient::new(config)?;
//!     
//!     // Use the client...
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! This crate is organized into several modules:
//!
//! - [`client`] - REST and WebSocket clients for API communication
//! - [`types`] - Request/response types matching the Kalshi API
//! - [`orderbook`] - High-performance orderbook data structure
//! - [`config`] - Configuration and credentials management
//! - [`error`] - Error types for the crate
//!
//! ## Performance
//!
//! This crate is designed for low-latency trading workloads:
//!
//! - Integer prices (cents) instead of floating point
//! - `FxHashMap` for faster hashing of small keys
//! - `parking_lot` mutexes (faster than std)
//! - Minimal allocations in hot paths
//! - `BTreeMap` for sorted price levels

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unsafe_code)]

pub mod client;
pub mod config;
pub mod error;
pub mod orderbook;
pub mod types;

// Re-export main types at crate root for convenience
pub use config::Config;
pub use error::Error;

/// Result type alias using the crate's Error type
pub type Result<T> = std::result::Result<T, Error>;

/// The main Kalshi API client
///
/// This struct provides access to both REST and WebSocket APIs.
///
/// # Example
///
/// ```rust,no_run
/// use kalshi_rs::{Config, KalshiClient};
///
/// # async fn example() -> kalshi_rs::Result<()> {
/// let config = Config::new("api-key", "private-key-pem");
/// let client = KalshiClient::new(config)?;
///
/// // REST API calls
/// // let markets = client.get_markets().await?;
///
/// // WebSocket connection
/// // let ws = client.websocket().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct KalshiClient {
    config: Config,
    rest_client: client::rest::RestClient,
}

impl KalshiClient {
    /// Create a new Kalshi client with the given configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the private key cannot be parsed or the HTTP client
    /// cannot be initialized.
    pub fn new(config: Config) -> Result<Self> {
        let rest_client = client::rest::RestClient::new(&config)?;
        Ok(Self {
            config,
            rest_client,
        })
    }

    /// Get a reference to the REST client
    pub fn rest(&self) -> &client::rest::RestClient {
        &self.rest_client
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    // TODO: Add WebSocket connection method
    // pub async fn websocket(&self) -> Result<client::websocket::WebSocketClient> { ... }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = Config::new("test-key", "test-private-key");
        assert_eq!(config.api_key_id(), "test-key");
    }
}
