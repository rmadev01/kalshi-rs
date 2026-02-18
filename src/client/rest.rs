//! HTTP REST client for Kalshi API.
//!
//! This module provides the [`RestClient`] for making authenticated HTTP requests
//! to the Kalshi REST API endpoints.
//!
//! # Example
//!
//! ```rust,no_run
//! use kalshi_trading::{Config, KalshiClient};
//!
//! # async fn example() -> kalshi_trading::Result<()> {
//! let config = Config::new("api-key", "private-key-pem");
//! let client = KalshiClient::new(config)?;
//!
//! // Get markets
//! let markets = client.rest().get_markets(None, None, None).await?;
//! for market in &markets.markets {
//!     println!("{}: {:?}", market.ticker, market.yes_bid);
//! }
//! # Ok(())
//! # }
//! ```

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::Client;

use crate::client::auth::{AuthHeaders, Signer};
use crate::config::Config;
use crate::error::{ApiError, Error};
use crate::types::market::*;
use crate::types::order::*;

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

    /// Make a DELETE request with a JSON body
    pub async fn delete_with_body<T, B>(&self, path: &str, body: &B) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let url = format!("{}{}", self.base_url, path);
        let full_path = format!("/trade-api/v2{}", path);
        let headers = self.auth_headers("DELETE", &full_path)?;

        let response = self
            .client
            .delete(&url)
            .headers(headers)
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a PUT request to the API
    pub async fn put<T, B>(&self, path: &str, body: &B) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let url = format!("{}{}", self.base_url, path);
        let full_path = format!("/trade-api/v2{}", path);
        let headers = self.auth_headers("PUT", &full_path)?;

        let response = self
            .client
            .put(&url)
            .headers(headers)
            .json(body)
            .send()
            .await?;

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

// ============================================================================
// Market Data API
// ============================================================================

impl RestClient {
    /// Get a list of markets with optional filters.
    ///
    /// # Arguments
    /// * `status` - Filter by market status (open, closed, settled)
    /// * `ticker` - Filter by specific market ticker
    /// * `event_ticker` - Filter by event ticker
    /// * `series_ticker` - Filter by series ticker
    /// * `cursor` - Pagination cursor
    /// * `limit` - Maximum number of results (default 100, max 1000)
    ///
    /// # Example
    /// ```rust,no_run
    /// # async fn example(client: &kalshi_trading::client::RestClient) -> kalshi_trading::Result<()> {
    /// let markets = client.get_markets(Some("open"), None, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_markets(
        &self,
        status: Option<&str>,
        event_ticker: Option<&str>,
        cursor: Option<&str>,
    ) -> Result<GetMarketsResponse, Error> {
        let mut path = "/markets".to_string();
        let mut params = Vec::new();

        if let Some(s) = status {
            params.push(format!("status={}", s));
        }
        if let Some(e) = event_ticker {
            params.push(format!("event_ticker={}", e));
        }
        if let Some(c) = cursor {
            params.push(format!("cursor={}", c));
        }

        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        self.get(&path).await
    }

    /// Get a specific market by ticker.
    pub async fn get_market(&self, ticker: &str) -> Result<GetMarketResponse, Error> {
        self.get(&format!("/markets/{}", ticker)).await
    }

    /// Get the orderbook for a market.
    ///
    /// Returns yes bids and no bids (no asks - in binary markets,
    /// yes bid at X is equivalent to no ask at 100-X).
    pub async fn get_orderbook(&self, ticker: &str) -> Result<GetOrderbookResponse, Error> {
        self.get(&format!("/markets/{}/orderbook", ticker)).await
    }

    /// Get a list of events.
    pub async fn get_events(
        &self,
        series_ticker: Option<&str>,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<GetEventsResponse, Error> {
        let mut path = "/events".to_string();
        let mut params = Vec::new();

        if let Some(s) = series_ticker {
            params.push(format!("series_ticker={}", s));
        }
        if let Some(c) = cursor {
            params.push(format!("cursor={}", c));
        }
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }

        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        self.get(&path).await
    }

    /// Get a specific event by ticker.
    pub async fn get_event(&self, event_ticker: &str) -> Result<GetEventResponse, Error> {
        self.get(&format!("/events/{}", event_ticker)).await
    }

    /// Get a series by ticker.
    pub async fn get_series(&self, series_ticker: &str) -> Result<GetSeriesResponse, Error> {
        self.get(&format!("/series/{}", series_ticker)).await
    }

    /// Get public trades for a market.
    pub async fn get_trades(
        &self,
        ticker: Option<&str>,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<GetTradesResponse, Error> {
        let mut path = "/markets/trades".to_string();
        let mut params = Vec::new();

        if let Some(t) = ticker {
            params.push(format!("ticker={}", t));
        }
        if let Some(c) = cursor {
            params.push(format!("cursor={}", c));
        }
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }

        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        self.get(&path).await
    }
}

// ============================================================================
// Order API
// ============================================================================

impl RestClient {
    /// Create a new order.
    ///
    /// # Example
    /// ```rust,no_run
    /// use kalshi_trading::types::{CreateOrderRequest, Side, Action};
    ///
    /// # async fn example(client: &kalshi_trading::client::RestClient) -> kalshi_trading::Result<()> {
    /// let order = CreateOrderRequest::limit("TICKER", Side::Yes, Action::Buy, 10, 5000);
    /// let response = client.create_order(&order).await?;
    /// println!("Order ID: {}", response.order.order_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_order(
        &self,
        request: &CreateOrderRequest,
    ) -> Result<CreateOrderResponse, Error> {
        self.post("/portfolio/orders", request).await
    }

    /// Get a list of orders with optional filters.
    pub async fn get_orders(
        &self,
        ticker: Option<&str>,
        status: Option<&str>,
        cursor: Option<&str>,
    ) -> Result<GetOrdersResponse, Error> {
        let mut path = "/portfolio/orders".to_string();
        let mut params = Vec::new();

        if let Some(t) = ticker {
            params.push(format!("ticker={}", t));
        }
        if let Some(s) = status {
            params.push(format!("status={}", s));
        }
        if let Some(c) = cursor {
            params.push(format!("cursor={}", c));
        }

        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        self.get(&path).await
    }

    /// Get a specific order by ID.
    pub async fn get_order(&self, order_id: &str) -> Result<GetOrderResponse, Error> {
        self.get(&format!("/portfolio/orders/{}", order_id)).await
    }

    /// Cancel an order.
    ///
    /// Returns the canceled order with remaining_count set to 0.
    pub async fn cancel_order(&self, order_id: &str) -> Result<CancelOrderResponse, Error> {
        self.delete(&format!("/portfolio/orders/{}", order_id))
            .await
    }

    /// Amend an order's price and/or quantity.
    ///
    /// The new count must be >= the current fill_count.
    pub async fn amend_order(
        &self,
        order_id: &str,
        request: &AmendOrderRequest,
    ) -> Result<AmendOrderResponse, Error> {
        self.post(&format!("/portfolio/orders/{}/amend", order_id), request)
            .await
    }

    /// Decrease an order's quantity.
    pub async fn decrease_order(
        &self,
        order_id: &str,
        request: &DecreaseOrderRequest,
    ) -> Result<DecreaseOrderResponse, Error> {
        self.post(
            &format!("/portfolio/orders/{}/decrease", order_id),
            request,
        )
        .await
    }

    /// Batch create multiple orders (up to 20).
    ///
    /// Each order counts against your rate limit.
    pub async fn batch_create_orders(
        &self,
        request: &BatchCreateOrdersRequest,
    ) -> Result<BatchCreateOrdersResponse, Error> {
        self.post("/portfolio/orders/batched", request).await
    }

    /// Batch cancel multiple orders (up to 20).
    pub async fn batch_cancel_orders(
        &self,
        request: &BatchCancelOrdersRequest,
    ) -> Result<BatchCancelOrdersResponse, Error> {
        self.delete_with_body("/portfolio/orders/batched", request)
            .await
    }

    /// Get queue positions for resting orders.
    pub async fn get_queue_positions(
        &self,
        market_tickers: Option<&str>,
    ) -> Result<GetOrderQueuePositionsResponse, Error> {
        let path = match market_tickers {
            Some(tickers) => format!("/portfolio/orders/queue_positions?market_tickers={}", tickers),
            None => "/portfolio/orders/queue_positions".to_string(),
        };
        self.get(&path).await
    }
}

// ============================================================================
// Portfolio API
// ============================================================================

impl RestClient {
    /// Get account balance and portfolio value.
    ///
    /// Returns values in centi-cents (divide by 100 for cents, 10000 for dollars).
    pub async fn get_balance(&self) -> Result<GetBalanceResponse, Error> {
        self.get("/portfolio/balance").await
    }

    /// Get positions in markets.
    pub async fn get_positions(
        &self,
        ticker: Option<&str>,
        event_ticker: Option<&str>,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<GetPositionsResponse, Error> {
        let mut path = "/portfolio/positions".to_string();
        let mut params = Vec::new();

        if let Some(t) = ticker {
            params.push(format!("ticker={}", t));
        }
        if let Some(e) = event_ticker {
            params.push(format!("event_ticker={}", e));
        }
        if let Some(c) = cursor {
            params.push(format!("cursor={}", c));
        }
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }

        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        self.get(&path).await
    }

    /// Get fills (matched trades) for your orders.
    pub async fn get_fills(
        &self,
        ticker: Option<&str>,
        order_id: Option<&str>,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<GetFillsResponse, Error> {
        let mut path = "/portfolio/fills".to_string();
        let mut params = Vec::new();

        if let Some(t) = ticker {
            params.push(format!("ticker={}", t));
        }
        if let Some(o) = order_id {
            params.push(format!("order_id={}", o));
        }
        if let Some(c) = cursor {
            params.push(format!("cursor={}", c));
        }
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }

        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        self.get(&path).await
    }

    /// Get settlement history.
    pub async fn get_settlements(
        &self,
        ticker: Option<&str>,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<GetSettlementsResponse, Error> {
        let mut path = "/portfolio/settlements".to_string();
        let mut params = Vec::new();

        if let Some(t) = ticker {
            params.push(format!("ticker={}", t));
        }
        if let Some(c) = cursor {
            params.push(format!("cursor={}", c));
        }
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }

        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        self.get(&path).await
    }
}

// ============================================================================
// Exchange API
// ============================================================================

impl RestClient {
    /// Get exchange status (trading active, exchange active).
    pub async fn get_exchange_status(&self) -> Result<ExchangeStatus, Error> {
        self.get("/exchange/status").await
    }

    /// Get exchange schedule.
    pub async fn get_exchange_schedule(&self) -> Result<GetExchangeScheduleResponse, Error> {
        self.get("/exchange/schedule").await
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would go here with mock server or test credentials
}
