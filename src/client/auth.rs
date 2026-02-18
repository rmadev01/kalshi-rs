//! RSA-PSS authentication for Kalshi API requests.
//!
//! Kalshi uses RSA-PSS signatures for API authentication. Each request must include:
//!
//! - `KALSHI-ACCESS-KEY`: Your API key ID
//! - `KALSHI-ACCESS-TIMESTAMP`: Unix timestamp in milliseconds
//! - `KALSHI-ACCESS-SIGNATURE`: RSA-PSS signature of `timestamp + method + path`
//!
//! # Example
//!
//! ```rust,no_run
//! use kalshi_rs::client::auth::Signer;
//!
//! let private_key_pem = "-----BEGIN PRIVATE KEY-----\n...";
//! let signer = Signer::new(private_key_pem).expect("Failed to parse key");
//!
//! let timestamp = std::time::SystemTime::now()
//!     .duration_since(std::time::UNIX_EPOCH)
//!     .unwrap()
//!     .as_millis() as u64;
//!
//! let signature = signer.sign(timestamp, "GET", "/trade-api/v2/markets")
//!     .expect("Failed to sign");
//! ```

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rsa::pkcs8::DecodePrivateKey;
use rsa::pss::SigningKey;
use rsa::sha2::Sha256;
use rsa::signature::RandomizedSigner;
use rsa::RsaPrivateKey;

use crate::error::Error;

/// RSA-PSS signer for Kalshi API authentication
#[derive(Debug)]
pub struct Signer {
    signing_key: SigningKey<Sha256>,
}

impl Signer {
    /// Create a new signer from a PEM-encoded private key
    ///
    /// # Arguments
    ///
    /// * `private_key_pem` - RSA private key in PEM format (PKCS#8)
    ///
    /// # Errors
    ///
    /// Returns an error if the PEM cannot be parsed as a valid RSA private key.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use kalshi_rs::client::auth::Signer;
    ///
    /// let pem = std::fs::read_to_string("private_key.pem").unwrap();
    /// let signer = Signer::new(&pem).expect("Invalid key");
    /// ```
    pub fn new(private_key_pem: &str) -> Result<Self, Error> {
        let private_key = RsaPrivateKey::from_pkcs8_pem(private_key_pem)?;
        let signing_key = SigningKey::<Sha256>::new(private_key);
        Ok(Self { signing_key })
    }

    /// Sign a request and return the base64-encoded signature
    ///
    /// # Arguments
    ///
    /// * `timestamp_ms` - Unix timestamp in milliseconds
    /// * `method` - HTTP method (GET, POST, DELETE, etc.)
    /// * `path` - Request path (e.g., "/trade-api/v2/markets")
    ///
    /// # Returns
    ///
    /// Base64-encoded RSA-PSS signature
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use kalshi_rs::client::auth::Signer;
    /// # let signer = Signer::new("").unwrap();
    /// let timestamp = 1700000000000u64;
    /// let signature = signer.sign(timestamp, "GET", "/trade-api/v2/markets").unwrap();
    /// ```
    pub fn sign(&self, timestamp_ms: u64, method: &str, path: &str) -> Result<String, Error> {
        // Build the message: timestamp + method + path
        let message = format!("{}{}{}", timestamp_ms, method, path);

        // Sign with RSA-PSS
        let mut rng = rand::thread_rng();
        let signature = self.signing_key.sign_with_rng(&mut rng, message.as_bytes());

        // Encode to base64 - signature implements AsRef<[u8]> via SignatureEncoding
        use rsa::signature::SignatureEncoding;
        Ok(BASE64.encode(signature.to_bytes()))
    }

    /// Get the current timestamp in milliseconds
    pub fn current_timestamp_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_millis() as u64
    }
}

/// Authentication headers for a Kalshi API request
#[derive(Debug, Clone)]
pub struct AuthHeaders {
    /// API key ID
    pub key: String,
    /// Unix timestamp in milliseconds
    pub timestamp: String,
    /// RSA-PSS signature (base64)
    pub signature: String,
}

impl AuthHeaders {
    /// Header name for API key
    pub const KEY_HEADER: &'static str = "KALSHI-ACCESS-KEY";
    /// Header name for timestamp
    pub const TIMESTAMP_HEADER: &'static str = "KALSHI-ACCESS-TIMESTAMP";
    /// Header name for signature
    pub const SIGNATURE_HEADER: &'static str = "KALSHI-ACCESS-SIGNATURE";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp() {
        let ts = Signer::current_timestamp_ms();
        // Should be after 2024
        assert!(ts > 1704067200000);
    }

    // Note: Can't test actual signing without a real private key
    // Integration tests would use a test key
}
