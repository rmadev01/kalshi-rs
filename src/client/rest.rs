//! HTTP REST client for Kalshi API.
//!
//! This module provides the [`RestClient`] for making authenticated HTTP requests
//! to the Kalshi REST API endpoints.
//!
//! # Example
//!
//! ```rust,no_run
//! use kalshi_rs::{Config, KalshiClient};
//!
//! # async fn example() -> kalshi_rs::Result<()> {
//! let config = Config::new("api-key", "private-key-pem");
//! let client = KalshiClient::new(config)?;
//!
//! // REST client is accessed through the main client
//! let rest = client.rest();
//! // let markets = rest.get_markets().await?;
//! # Ok(())
//! # }
//! ```

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::Client;

use crate::client::auth::{AuthHeaders, Signer};
use crate::config::Config;
use crate::error::{ApiError, Error};

/// HTTP client for Kalshi REST API
#[derive(Debug)]
pub struct RestClient {
    client: Client,
    base_url: String,
    api_key_id: String,
    signer: Signer,
}

impl RestClient {
    /// Create a new REST client
    ///
    /// # Arguments
    ///
    /// * `config` - Client configuration with credentials
    ///
    /// # Errors
    ///
    /// Returns an error if the private key cannot be parsed or the HTTP client
    /// cannot be initialized.
    pub fn new(config: &Config) -> Result<Self, Error> {
        let signer = Signer::new(config.private_key_pem())?;

        let client = Client::builder()
            .timeout(config.timeout())
            .build()?;

        Ok(Self {
            client,
            base_url: config.rest_base_url().to_string(),
            api_key_id: config.api_key_id().to_string(),
            signer,
        })
    }

    /// Build authentication headers for a request
    fn auth_headers(&self, method: &str, path: &str) -> Result<HeaderMap, Error> {
        let timestamp = Signer::current_timestamp_ms();
        let signature = self.signer.sign(timestamp, method, path)?;

        let mut headers = HeaderMap::new();
        headers.insert(
            AuthHeaders::KEY_HEADER,
            HeaderValue::from_str(&self.api_key_id).unwrap(),
        );
        headers.insert(
            AuthHeaders::TIMESTAMP_HEADER,
            HeaderValue::from_str(&timestamp.to_string()).unwrap(),
        );
        headers.insert(
            AuthHeaders::SIGNATURE_HEADER,
            HeaderValue::from_str(&signature).unwrap(),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        Ok(headers)
    }

    /// Make a GET request to the API
    ///
    /// # Arguments
    ///
    /// * `path` - API path (without base URL)
    ///
    /// # Returns
    ///
    /// Deserialized response body
    pub async fn get<T>(&self, path: &str) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        let full_path = format!("/trade-api/v2{}", path);
        let headers = self.auth_headers("GET", &full_path)?;

        let response = self.client.get(&url).headers(headers).send().await?;

        self.handle_response(response).await
    }

    /// Make a POST request to the API
    ///
    /// # Arguments
    ///
    /// * `path` - API path (without base URL)
    /// * `body` - Request body to serialize as JSON
    ///
    /// # Returns
    ///
    /// Deserialized response body
    pub async fn post<T, B>(&self, path: &str, body: &B) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let url = format!("{}{}", self.base_url, path);
        let full_path = format!("/trade-api/v2{}", path);
        let headers = self.auth_headers("POST", &full_path)?;

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a DELETE request to the API
    ///
    /// # Arguments
    ///
    /// * `path` - API path (without base URL)
    ///
    /// # Returns
    ///
    /// Deserialized response body
    pub async fn delete<T>(&self, path: &str) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        let full_path = format!("/trade-api/v2{}", path);
        let headers = self.auth_headers("DELETE", &full_path)?;

        let response = self.client.delete(&url).headers(headers).send().await?;

        self.handle_response(response).await
    }

    /// Handle the HTTP response, checking for errors
    async fn handle_response<T>(&self, response: reqwest::Response) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();

        // Check for rate limiting
        if status.as_u16() == 429 {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());

            return Err(Error::RateLimited {
                retry_after_ms: retry_after,
            });
        }

        // Check for errors
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            // Try to parse as API error
            if let Ok(error_response) = serde_json::from_str::<serde_json::Value>(&body) {
                let message = error_response
                    .get("message")
                    .or_else(|| error_response.get("error"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(&body)
                    .to_string();

                let code = error_response
                    .get("code")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                return Err(Error::Api(ApiError {
                    status: status.as_u16(),
                    code,
                    message,
                }));
            }

            return Err(Error::Api(ApiError::new(status.as_u16(), body)));
        }

        // Deserialize successful response
        let body = response.text().await?;
        serde_json::from_str(&body).map_err(Error::from)
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

// TODO: Implement specific API methods
// impl RestClient {
//     pub async fn get_markets(&self) -> Result<Vec<Market>, Error> { ... }
//     pub async fn get_market(&self, ticker: &str) -> Result<Market, Error> { ... }
//     pub async fn create_order(&self, order: &CreateOrderRequest) -> Result<Order, Error> { ... }
//     pub async fn cancel_order(&self, order_id: &str) -> Result<Order, Error> { ... }
//     pub async fn get_balance(&self) -> Result<Balance, Error> { ... }
//     pub async fn get_positions(&self) -> Result<Vec<Position>, Error> { ... }
// }

#[cfg(test)]
mod tests {
    // Integration tests would go here with mock server or test credentials
}
