//! Error types for the kalshi-rs crate.
//!
//! This module defines the error types that can occur when interacting with
//! the Kalshi API, including network errors, authentication failures, and
//! API-specific errors.

use std::fmt;

/// The main error type for this crate
#[derive(Debug)]
pub enum Error {
    /// HTTP request failed
    Http(reqwest::Error),

    /// WebSocket error
    WebSocket(tokio_tungstenite::tungstenite::Error),

    /// JSON serialization/deserialization error
    Json(serde_json::Error),

    /// RSA cryptography error (key parsing, signing)
    Crypto(String),

    /// Invalid configuration (missing fields, bad format)
    Config(String),

    /// API returned an error response
    Api(ApiError),

    /// Rate limit exceeded
    RateLimited {
        /// Retry after this many milliseconds
        retry_after_ms: Option<u64>,
    },

    /// Authentication failed
    Authentication(String),

    /// WebSocket connection closed unexpectedly
    ConnectionClosed,

    /// Orderbook sequence gap detected (missed messages)
    SequenceGap {
        /// Expected sequence number
        expected: u64,
        /// Actual sequence number received
        got: u64,
    },

    /// Invalid market ticker
    InvalidTicker(String),

    /// Operation timed out
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Http(e) => write!(f, "HTTP error: {}", e),
            Error::WebSocket(e) => write!(f, "WebSocket error: {}", e),
            Error::Json(e) => write!(f, "JSON error: {}", e),
            Error::Crypto(msg) => write!(f, "Crypto error: {}", msg),
            Error::Config(msg) => write!(f, "Configuration error: {}", msg),
            Error::Api(e) => write!(f, "API error ({}): {}", e.status, e.message),
            Error::RateLimited { retry_after_ms } => {
                if let Some(ms) = retry_after_ms {
                    write!(f, "Rate limited, retry after {}ms", ms)
                } else {
                    write!(f, "Rate limited")
                }
            }
            Error::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            Error::ConnectionClosed => write!(f, "WebSocket connection closed"),
            Error::SequenceGap { expected, got } => {
                write!(f, "Sequence gap: expected {}, got {}", expected, got)
            }
            Error::InvalidTicker(ticker) => write!(f, "Invalid ticker: {}", ticker),
            Error::Timeout => write!(f, "Operation timed out"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Http(e) => Some(e),
            Error::WebSocket(e) => Some(e),
            Error::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Http(err)
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        Error::WebSocket(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
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

impl ApiError {
    /// Create a new API error
    pub fn new(status: u16, message: impl Into<String>) -> Self {
        Self {
            status,
            code: None,
            message: message.into(),
        }
    }

    /// Create an API error with an error code
    pub fn with_code(status: u16, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status,
            code: Some(code.into()),
            message: message.into(),
        }
    }

    /// Check if this is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status)
    }

    /// Check if this is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status)
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
}
