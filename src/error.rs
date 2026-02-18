//! Error types for the kalshi-rs crate.
//!
//! This module defines the error types that can occur when interacting with
//! the Kalshi API, including network errors, authentication failures, and
//! API-specific errors.

use std::fmt;
use thiserror::Error;

/// The main error type for this crate
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// HTTP request failed
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// WebSocket error (boxed to reduce enum size)
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] Box<tokio_tungstenite::tungstenite::Error>),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// IO error (file reading, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// RSA cryptography error (key parsing, signing)
    #[error("Crypto error: {0}")]
    Crypto(String),

    /// Invalid configuration (missing fields, bad format)
    #[error("Configuration error: {0}")]
    Config(String),

    /// API returned an error response
    #[error("API error ({status}): {message}", status = .0.status, message = .0.message)]
    Api(ApiError),

    /// Rate limit exceeded
    #[error("{}", match .retry_after_ms {
        Some(ms) => format!("Rate limited, retry after {}ms", ms),
        None => "Rate limited".to_string(),
    })]
    RateLimited {
        /// Retry after this many milliseconds
        retry_after_ms: Option<u64>,
    },

    /// Authentication failed
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// WebSocket connection closed unexpectedly
    #[error("WebSocket connection closed")]
    ConnectionClosed,

    /// Orderbook sequence gap detected (missed messages)
    #[error("Sequence gap: expected {expected}, got {got}")]
    SequenceGap {
        /// Expected sequence number
        expected: u64,
        /// Actual sequence number received
        got: u64,
    },

    /// Invalid market ticker
    #[error("Invalid ticker: {0}")]
    InvalidTicker(String),

    /// Operation timed out
    #[error("Operation timed out")]
    Timeout,
}

/// Error returned by the Kalshi API
#[derive(Debug, Clone)]
pub struct ApiError {
    /// HTTP status code
    pub status: u16,
    /// Error code from API (if provided)
    pub code: Option<String>,
    /// Error message
    pub message: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(code) = &self.code {
            write!(f, "[{}] {} (status {})", code, self.message, self.status)
        } else {
            write!(f, "{} (status {})", self.message, self.status)
        }
    }
}

impl std::error::Error for ApiError {}

// Manual From impl for tungstenite since it's boxed
impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        Error::WebSocket(Box::new(err))
    }
}

impl From<rsa::Error> for Error {
    fn from(err: rsa::Error) -> Self {
        Error::Crypto(err.to_string())
    }
}

impl From<rsa::pkcs8::Error> for Error {
    fn from(err: rsa::pkcs8::Error) -> Self {
        Error::Crypto(format!("PKCS8 error: {}", err))
    }
}

impl From<rsa::pkcs1::Error> for Error {
    fn from(err: rsa::pkcs1::Error) -> Self {
        Error::Crypto(format!("PKCS1 error: {}", err))
    }
}

impl ApiError {
    /// Create a new API error
    #[must_use]
    pub fn new(status: u16, message: impl Into<String>) -> Self {
        Self {
            status,
            code: None,
            message: message.into(),
        }
    }

    /// Create an API error with an error code
    #[must_use]
    pub fn with_code(status: u16, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status,
            code: Some(code.into()),
            message: message.into(),
        }
    }

    /// Check if this is a client error (4xx)
    #[must_use]
    pub const fn is_client_error(&self) -> bool {
        self.status >= 400 && self.status < 500
    }

    /// Check if this is a server error (5xx)
    #[must_use]
    pub const fn is_server_error(&self) -> bool {
        self.status >= 500 && self.status < 600
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_display() {
        let err = Error::Api(ApiError::new(400, "Bad request"));
        assert!(err.to_string().contains("400"));
        assert!(err.to_string().contains("Bad request"));
    }

    #[test]
    fn test_rate_limited_display() {
        let err = Error::RateLimited {
            retry_after_ms: Some(1000),
        };
        assert!(err.to_string().contains("1000"));
    }

    #[test]
    fn test_sequence_gap() {
        let err = Error::SequenceGap {
            expected: 5,
            got: 8,
        };
        assert!(err.to_string().contains("5"));
        assert!(err.to_string().contains("8"));
    }

    #[test]
    fn test_api_error_with_code() {
        let err = ApiError::with_code(401, "UNAUTHORIZED", "Invalid credentials");
        assert!(err.to_string().contains("UNAUTHORIZED"));
        assert!(err.to_string().contains("Invalid credentials"));
        assert!(err.to_string().contains("401"));
    }

    #[test]
    fn test_error_is_client_server() {
        let client_err = ApiError::new(404, "Not found");
        let server_err = ApiError::new(500, "Internal error");

        assert!(client_err.is_client_error());
        assert!(!client_err.is_server_error());
        assert!(!server_err.is_client_error());
        assert!(server_err.is_server_error());
    }
}
