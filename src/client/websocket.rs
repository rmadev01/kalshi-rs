//! WebSocket client for real-time Kalshi market data.
//!
//! This module provides the [`WebSocketClient`] for streaming real-time data:
//!
//! - Orderbook snapshots and deltas
//! - Trade executions
//! - Fill notifications
//! - Market lifecycle events
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
//! // WebSocket connection (not yet implemented)
//! // let mut ws = client.websocket().await?;
//! // ws.subscribe_orderbook(&["KXBTC-25JAN"]).await?;
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::http::Request;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::client::auth::Signer;
use crate::config::Config;
use crate::error::Error;
use crate::types::messages::{SubscribeParams, UpdateSubscriptionParams, WsCommand, WsMessage};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Information about an active subscription
#[derive(Debug, Clone)]
pub struct SubscriptionInfo {
    /// Subscription ID (used for unsubscribe)
    pub sid: u64,
    /// Channel name
    pub channel: String,
    /// Market tickers (if applicable)
    pub market_tickers: Option<Vec<String>>,
}

/// WebSocket client for real-time market data
///
/// Provides methods to subscribe to various channels and receive real-time updates.
/// Automatically tracks subscriptions by their subscription ID (sid) for easy management.
///
/// # Thread Safety
///
/// This client is NOT thread-safe. For concurrent access from multiple tasks,
/// use channels or wrap in a mutex.
#[derive(Debug)]
pub struct WebSocketClient {
    write: SplitSink<WsStream, Message>,
    read: SplitStream<WsStream>,
    message_id: u64,
    /// Active subscriptions by sid
    subscriptions: HashMap<u64, SubscriptionInfo>,
    /// Pending subscription requests by message id
    pending_subscriptions: HashMap<u64, PendingSubscription>,
}

/// Information about a pending subscription request
#[derive(Debug, Clone)]
struct PendingSubscription {
    channel: String,
    market_tickers: Option<Vec<String>>,
}

impl WebSocketClient {
    /// Connect to the Kalshi WebSocket API
    ///
    /// # Arguments
    ///
    /// * `config` - Client configuration with credentials
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails or authentication headers
    /// cannot be generated.
    pub async fn connect(config: &Config) -> Result<Self, Error> {
        let signer = Signer::new(config.private_key_pem())?;
        let timestamp = Signer::current_timestamp_ms();
        let signature = signer.sign(timestamp, "GET", "/trade-api/ws/v2")?;

        // Build WebSocket request with auth headers
        let request = Request::builder()
            .uri(config.websocket_url())
            .header("KALSHI-ACCESS-KEY", config.api_key_id())
            .header("KALSHI-ACCESS-TIMESTAMP", timestamp.to_string())
            .header("KALSHI-ACCESS-SIGNATURE", signature)
            .header("Host", "api.elections.kalshi.com")
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header(
                "Sec-WebSocket-Key",
                tokio_tungstenite::tungstenite::handshake::client::generate_key(),
            )
            .body(())?;

        let (ws_stream, _response) = tokio_tungstenite::connect_async(request).await?;
        let (write, read) = ws_stream.split();

        Ok(Self {
            write,
            read,
            message_id: 1,
            subscriptions: HashMap::new(),
            pending_subscriptions: HashMap::new(),
        })
    }

    /// Send a command to the WebSocket server
    async fn send_command(&mut self, cmd: WsCommand) -> Result<u64, Error> {
        let msg_id = self.message_id;
        let json = serde_json::to_string(&cmd)?;
        self.write.send(Message::Text(json)).await?;
        self.message_id += 1;
        Ok(msg_id)
    }

    /// Get the next message ID without incrementing
    pub fn next_message_id(&self) -> u64 {
        self.message_id
    }

    /// Get all active subscriptions
    pub fn subscriptions(&self) -> &HashMap<u64, SubscriptionInfo> {
        &self.subscriptions
    }

    /// Get subscription info by sid
    pub fn get_subscription(&self, sid: u64) -> Option<&SubscriptionInfo> {
        self.subscriptions.get(&sid)
    }

    /// Subscribe to orderbook updates for the given markets
    ///
    /// # Arguments
    ///
    /// * `market_tickers` - Market tickers to subscribe to
    ///
    /// # Returns
    ///
    /// The message ID of the subscription request (use to correlate with response)
    pub async fn subscribe_orderbook(&mut self, market_tickers: &[&str]) -> Result<u64, Error> {
        let tickers: Vec<String> = market_tickers.iter().map(|s| s.to_string()).collect();
        let msg_id = self.message_id;
        
        self.pending_subscriptions.insert(
            msg_id,
            PendingSubscription {
                channel: "orderbook_delta".to_string(),
                market_tickers: Some(tickers.clone()),
            },
        );

        let cmd = WsCommand::Subscribe {
            id: msg_id,
            params: SubscribeParams {
                channels: vec!["orderbook_delta".to_string()],
                market_tickers: Some(tickers),
            },
        };
        self.send_command(cmd).await
    }

    /// Subscribe to ticker updates
    ///
    /// # Arguments
    ///
    /// * `market_tickers` - Optional market tickers (None for all markets)
    pub async fn subscribe_ticker(
        &mut self,
        market_tickers: Option<&[&str]>,
    ) -> Result<u64, Error> {
        let tickers = market_tickers.map(|t| t.iter().map(|s| s.to_string()).collect());
        let msg_id = self.message_id;

        self.pending_subscriptions.insert(
            msg_id,
            PendingSubscription {
                channel: "ticker".to_string(),
                market_tickers: tickers.clone(),
            },
        );

        let cmd = WsCommand::Subscribe {
            id: msg_id,
            params: SubscribeParams {
                channels: vec!["ticker".to_string()],
                market_tickers: tickers,
            },
        };
        self.send_command(cmd).await
    }

    /// Subscribe to trade updates
    pub async fn subscribe_trades(
        &mut self,
        market_tickers: Option<&[&str]>,
    ) -> Result<u64, Error> {
        let tickers = market_tickers.map(|t| t.iter().map(|s| s.to_string()).collect());
        let msg_id = self.message_id;

        self.pending_subscriptions.insert(
            msg_id,
            PendingSubscription {
                channel: "trade".to_string(),
                market_tickers: tickers.clone(),
            },
        );

        let cmd = WsCommand::Subscribe {
            id: msg_id,
            params: SubscribeParams {
                channels: vec!["trade".to_string()],
                market_tickers: tickers,
            },
        };
        self.send_command(cmd).await
    }

    /// Subscribe to fill notifications (your trades)
    pub async fn subscribe_fills(
        &mut self,
        market_tickers: Option<&[&str]>,
    ) -> Result<u64, Error> {
        let tickers = market_tickers.map(|t| t.iter().map(|s| s.to_string()).collect());
        let msg_id = self.message_id;

        self.pending_subscriptions.insert(
            msg_id,
            PendingSubscription {
                channel: "fill".to_string(),
                market_tickers: tickers.clone(),
            },
        );

        let cmd = WsCommand::Subscribe {
            id: msg_id,
            params: SubscribeParams {
                channels: vec!["fill".to_string()],
                market_tickers: tickers,
            },
        };
        self.send_command(cmd).await
    }

    /// Subscribe to user order updates
    ///
    /// Receives updates when your orders are placed, filled, cancelled, etc.
    pub async fn subscribe_user_orders(&mut self) -> Result<u64, Error> {
        let msg_id = self.message_id;

        self.pending_subscriptions.insert(
            msg_id,
            PendingSubscription {
                channel: "user_orders".to_string(),
                market_tickers: None,
            },
        );

        let cmd = WsCommand::Subscribe {
            id: msg_id,
            params: SubscribeParams {
                channels: vec!["user_orders".to_string()],
                market_tickers: None,
            },
        };
        self.send_command(cmd).await
    }

    /// Subscribe to market lifecycle events
    ///
    /// Receives updates when markets open, close, settle, etc.
    ///
    /// # Arguments
    ///
    /// * `market_tickers` - Optional market tickers (None for all markets)
    pub async fn subscribe_market_lifecycle(
        &mut self,
        market_tickers: Option<&[&str]>,
    ) -> Result<u64, Error> {
        let tickers = market_tickers.map(|t| t.iter().map(|s| s.to_string()).collect());
        let msg_id = self.message_id;

        self.pending_subscriptions.insert(
            msg_id,
            PendingSubscription {
                channel: "market_lifecycle_v2".to_string(),
                market_tickers: tickers.clone(),
            },
        );

        let cmd = WsCommand::Subscribe {
            id: msg_id,
            params: SubscribeParams {
                channels: vec!["market_lifecycle_v2".to_string()],
                market_tickers: tickers,
            },
        };
        self.send_command(cmd).await
    }

    /// Unsubscribe from channels by subscription ID
    ///
    /// # Arguments
    ///
    /// * `sids` - Subscription IDs to unsubscribe from
    pub async fn unsubscribe(&mut self, sids: &[u64]) -> Result<u64, Error> {
        let cmd = WsCommand::Unsubscribe {
            id: self.message_id,
            params: crate::types::messages::UnsubscribeParams {
                sids: sids.to_vec(),
            },
        };
        self.send_command(cmd).await
    }

    /// Update an existing subscription to add or remove markets
    ///
    /// # Arguments
    ///
    /// * `sid` - The subscription ID to update
    /// * `add_tickers` - Market tickers to add
    /// * `remove_tickers` - Market tickers to remove
    pub async fn update_subscription(
        &mut self,
        sid: u64,
        add_tickers: Option<&[&str]>,
        remove_tickers: Option<&[&str]>,
    ) -> Result<u64, Error> {
        let cmd = WsCommand::UpdateSubscription {
            id: self.message_id,
            params: UpdateSubscriptionParams {
                sid,
                market_tickers_add: add_tickers.map(|t| t.iter().map(|s| s.to_string()).collect()),
                market_tickers_delete: remove_tickers
                    .map(|t| t.iter().map(|s| s.to_string()).collect()),
            },
        };
        self.send_command(cmd).await
    }

    /// List current subscriptions
    pub async fn list_subscriptions(&mut self) -> Result<u64, Error> {
        let cmd = WsCommand::ListSubscriptions {
            id: self.message_id,
        };
        self.send_command(cmd).await
    }

    /// Receive the next message from the WebSocket
    ///
    /// This method also handles subscription tracking automatically:
    /// - When a `Subscribed` message is received, it adds to the subscriptions map
    /// - When an `Unsubscribed` message is received, it removes from the subscriptions map
    ///
    /// # Returns
    ///
    /// The next message, or `None` if the connection is closed.
    pub async fn next(&mut self) -> Option<Result<WsMessage, Error>> {
        loop {
            match self.read.next().await? {
                Ok(Message::Text(text)) => {
                    let result: Result<WsMessage, _> = serde_json::from_str(&text);
                    match result {
                        Ok(msg) => {
                            // Track subscription state
                            self.handle_subscription_tracking(&msg);
                            return Some(Ok(msg));
                        }
                        Err(e) => return Some(Err(Error::from(e))),
                    }
                }
                Ok(Message::Ping(data)) => {
                    // Respond to pings automatically
                    if let Err(e) = self.write.send(Message::Pong(data)).await {
                        return Some(Err(e.into()));
                    }
                }
                Ok(Message::Close(_)) => {
                    return Some(Err(Error::ConnectionClosed));
                }
                Ok(_) => {
                    // Ignore other message types (Binary, Pong, Frame)
                    continue;
                }
                Err(e) => {
                    return Some(Err(e.into()));
                }
            }
        }
    }

    /// Handle subscription tracking for incoming messages
    fn handle_subscription_tracking(&mut self, msg: &WsMessage) {
        match msg {
            WsMessage::Subscribed(subscribed) => {
                // Move pending subscription to active
                if let Some(id) = subscribed.id {
                    if let Some(pending) = self.pending_subscriptions.remove(&id) {
                        self.subscriptions.insert(
                            subscribed.msg.sid,
                            SubscriptionInfo {
                                sid: subscribed.msg.sid,
                                channel: pending.channel,
                                market_tickers: pending.market_tickers,
                            },
                        );
                    }
                }
            }
            WsMessage::Unsubscribed(unsubscribed) => {
                self.subscriptions.remove(&unsubscribed.sid);
            }
            _ => {}
        }
    }

    /// Close the WebSocket connection
    pub async fn close(&mut self) -> Result<(), Error> {
        self.write.close().await?;
        Ok(())
    }
}

impl From<tokio_tungstenite::tungstenite::http::Error> for Error {
    fn from(err: tokio_tungstenite::tungstenite::http::Error) -> Self {
        Error::Config(format!("HTTP error building WebSocket request: {}", err))
    }
}

/// Configuration for reconnection behavior
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Maximum number of reconnection attempts (0 = infinite)
    pub max_retries: u32,
    /// Initial delay between reconnection attempts
    pub initial_delay_ms: u64,
    /// Maximum delay between reconnection attempts
    pub max_delay_ms: u64,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            max_retries: 10,
            initial_delay_ms: 100,
            max_delay_ms: 30_000,
            backoff_multiplier: 2.0,
        }
    }
}

impl ReconnectConfig {
    /// Create a new reconnect config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum retries (0 = infinite)
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set initial delay in milliseconds
    pub fn initial_delay_ms(mut self, ms: u64) -> Self {
        self.initial_delay_ms = ms;
        self
    }

    /// Set maximum delay in milliseconds
    pub fn max_delay_ms(mut self, ms: u64) -> Self {
        self.max_delay_ms = ms;
        self
    }

    /// Set backoff multiplier
    pub fn backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// Calculate delay for a given retry attempt
    pub fn delay_for_attempt(&self, attempt: u32) -> std::time::Duration {
        let delay = self.initial_delay_ms as f64 * self.backoff_multiplier.powi(attempt as i32);
        let delay_ms = delay.min(self.max_delay_ms as f64) as u64;
        std::time::Duration::from_millis(delay_ms)
    }
}

/// A subscription request that can be replayed after reconnection
#[derive(Debug, Clone)]
pub enum SubscriptionRequest {
    /// Subscribe to orderbook deltas
    Orderbook(Vec<String>),
    /// Subscribe to ticker updates
    Ticker(Option<Vec<String>>),
    /// Subscribe to trades
    Trades(Option<Vec<String>>),
    /// Subscribe to fills
    Fills(Option<Vec<String>>),
    /// Subscribe to user orders
    UserOrders,
    /// Subscribe to market lifecycle
    MarketLifecycle(Option<Vec<String>>),
}

/// WebSocket client with automatic reconnection support.
///
/// This wrapper around [`WebSocketClient`] provides:
/// - Automatic reconnection with exponential backoff
/// - Subscription replay after reconnection
/// - Connection state tracking
///
/// # Example
///
/// ```rust,no_run
/// use kalshi_rs::Config;
/// use kalshi_rs::client::websocket::{ReconnectingWebSocket, ReconnectConfig};
///
/// # async fn example() -> kalshi_rs::Result<()> {
/// let config = Config::new("api-key", "private-key-pem");
/// let reconnect_config = ReconnectConfig::default();
///
/// let mut ws = ReconnectingWebSocket::connect(config, reconnect_config).await?;
///
/// // Subscribe - will be automatically replayed on reconnection
/// ws.subscribe_orderbook(&["KXBTC-25JAN"]).await?;
///
/// loop {
///     match ws.next().await {
///         Some(Ok(msg)) => {
///             // Handle message
///         }
///         Some(Err(e)) => {
///             // Error occurred - reconnection will be attempted automatically
///             eprintln!("Error: {}", e);
///         }
///         None => {
///             // Connection closed
///             break;
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct ReconnectingWebSocket {
    /// The underlying WebSocket client
    client: Option<WebSocketClient>,
    /// Configuration for API connection
    config: Config,
    /// Reconnection configuration
    reconnect_config: ReconnectConfig,
    /// Subscriptions to replay after reconnection
    subscription_requests: Vec<SubscriptionRequest>,
    /// Current reconnection attempt
    reconnect_attempt: u32,
    /// Whether we're currently trying to reconnect
    is_reconnecting: bool,
}

impl std::fmt::Debug for ReconnectingWebSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReconnectingWebSocket")
            .field("connected", &self.client.is_some())
            .field("reconnect_attempt", &self.reconnect_attempt)
            .field("is_reconnecting", &self.is_reconnecting)
            .field("subscription_count", &self.subscription_requests.len())
            .finish()
    }
}

impl ReconnectingWebSocket {
    /// Connect to the Kalshi WebSocket API with reconnection support
    pub async fn connect(config: Config, reconnect_config: ReconnectConfig) -> Result<Self, Error> {
        let client = WebSocketClient::connect(&config).await?;

        Ok(Self {
            client: Some(client),
            config,
            reconnect_config,
            subscription_requests: Vec::new(),
            reconnect_attempt: 0,
            is_reconnecting: false,
        })
    }

    /// Check if currently connected
    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    /// Check if currently reconnecting
    pub fn is_reconnecting(&self) -> bool {
        self.is_reconnecting
    }

    /// Get the current reconnection attempt number
    pub fn reconnect_attempt(&self) -> u32 {
        self.reconnect_attempt
    }

    /// Get active subscriptions (if connected)
    pub fn subscriptions(&self) -> Option<&HashMap<u64, SubscriptionInfo>> {
        self.client.as_ref().map(|c| c.subscriptions())
    }

    /// Subscribe to orderbook updates
    ///
    /// The subscription will be automatically replayed if the connection is lost.
    pub async fn subscribe_orderbook(&mut self, market_tickers: &[&str]) -> Result<u64, Error> {
        let tickers: Vec<String> = market_tickers.iter().map(|s| s.to_string()).collect();
        self.subscription_requests
            .push(SubscriptionRequest::Orderbook(tickers));

        if let Some(ref mut client) = self.client {
            client.subscribe_orderbook(market_tickers).await
        } else {
            Err(Error::ConnectionClosed)
        }
    }

    /// Subscribe to ticker updates
    pub async fn subscribe_ticker(
        &mut self,
        market_tickers: Option<&[&str]>,
    ) -> Result<u64, Error> {
        let tickers = market_tickers.map(|t| t.iter().map(|s| s.to_string()).collect());
        self.subscription_requests
            .push(SubscriptionRequest::Ticker(tickers));

        if let Some(ref mut client) = self.client {
            client.subscribe_ticker(market_tickers).await
        } else {
            Err(Error::ConnectionClosed)
        }
    }

    /// Subscribe to trade updates
    pub async fn subscribe_trades(
        &mut self,
        market_tickers: Option<&[&str]>,
    ) -> Result<u64, Error> {
        let tickers = market_tickers.map(|t| t.iter().map(|s| s.to_string()).collect());
        self.subscription_requests
            .push(SubscriptionRequest::Trades(tickers));

        if let Some(ref mut client) = self.client {
            client.subscribe_trades(market_tickers).await
        } else {
            Err(Error::ConnectionClosed)
        }
    }

    /// Subscribe to fill notifications
    pub async fn subscribe_fills(
        &mut self,
        market_tickers: Option<&[&str]>,
    ) -> Result<u64, Error> {
        let tickers = market_tickers.map(|t| t.iter().map(|s| s.to_string()).collect());
        self.subscription_requests
            .push(SubscriptionRequest::Fills(tickers));

        if let Some(ref mut client) = self.client {
            client.subscribe_fills(market_tickers).await
        } else {
            Err(Error::ConnectionClosed)
        }
    }

    /// Subscribe to user order updates
    pub async fn subscribe_user_orders(&mut self) -> Result<u64, Error> {
        self.subscription_requests
            .push(SubscriptionRequest::UserOrders);

        if let Some(ref mut client) = self.client {
            client.subscribe_user_orders().await
        } else {
            Err(Error::ConnectionClosed)
        }
    }

    /// Subscribe to market lifecycle events
    pub async fn subscribe_market_lifecycle(
        &mut self,
        market_tickers: Option<&[&str]>,
    ) -> Result<u64, Error> {
        let tickers = market_tickers.map(|t| t.iter().map(|s| s.to_string()).collect());
        self.subscription_requests
            .push(SubscriptionRequest::MarketLifecycle(tickers));

        if let Some(ref mut client) = self.client {
            client.subscribe_market_lifecycle(market_tickers).await
        } else {
            Err(Error::ConnectionClosed)
        }
    }

    /// Clear all saved subscriptions
    ///
    /// Subscriptions will no longer be replayed on reconnection.
    pub fn clear_subscriptions(&mut self) {
        self.subscription_requests.clear();
    }

    /// Receive the next message, reconnecting if necessary
    ///
    /// This method will automatically attempt to reconnect if the connection
    /// is lost, replaying all subscriptions after successful reconnection.
    pub async fn next(&mut self) -> Option<Result<WsMessage, Error>> {
        loop {
            if let Some(ref mut client) = self.client {
                match client.next().await {
                    Some(Ok(msg)) => {
                        self.reconnect_attempt = 0; // Reset on successful message
                        return Some(Ok(msg));
                    }
                    Some(Err(Error::ConnectionClosed)) | None => {
                        // Connection lost, attempt reconnection
                        self.client = None;
                        if let Err(e) = self.attempt_reconnect().await {
                            return Some(Err(e));
                        }
                        // Continue loop to receive from new connection
                        continue;
                    }
                    Some(Err(e)) => {
                        return Some(Err(e));
                    }
                }
            } else {
                // Not connected, attempt reconnection
                if let Err(e) = self.attempt_reconnect().await {
                    return Some(Err(e));
                }
            }
        }
    }

    /// Attempt to reconnect with exponential backoff
    async fn attempt_reconnect(&mut self) -> Result<(), Error> {
        self.is_reconnecting = true;

        loop {
            // Check max retries
            if self.reconnect_config.max_retries > 0
                && self.reconnect_attempt >= self.reconnect_config.max_retries
            {
                self.is_reconnecting = false;
                return Err(Error::ConnectionClosed);
            }

            // Calculate and wait for backoff delay
            let delay = self.reconnect_config.delay_for_attempt(self.reconnect_attempt);
            tokio::time::sleep(delay).await;

            self.reconnect_attempt += 1;

            // Attempt to connect
            match WebSocketClient::connect(&self.config).await {
                Ok(mut client) => {
                    // Replay subscriptions
                    if self.replay_subscriptions(&mut client).await.is_err() {
                        // Failed to replay, try again
                        continue;
                    }

                    self.client = Some(client);
                    self.is_reconnecting = false;
                    return Ok(());
                }
                Err(_) => {
                    // Connection failed, continue loop to retry
                    continue;
                }
            }
        }
    }

    /// Replay all saved subscriptions on a new connection
    async fn replay_subscriptions(&self, client: &mut WebSocketClient) -> Result<(), Error> {
        for request in &self.subscription_requests {
            match request {
                SubscriptionRequest::Orderbook(tickers) => {
                    let refs: Vec<&str> = tickers.iter().map(|s| s.as_str()).collect();
                    client.subscribe_orderbook(&refs).await?;
                }
                SubscriptionRequest::Ticker(tickers) => {
                    let refs = tickers.as_ref().map(|t| {
                        t.iter().map(|s| s.as_str()).collect::<Vec<_>>()
                    });
                    client.subscribe_ticker(refs.as_deref()).await?;
                }
                SubscriptionRequest::Trades(tickers) => {
                    let refs = tickers.as_ref().map(|t| {
                        t.iter().map(|s| s.as_str()).collect::<Vec<_>>()
                    });
                    client.subscribe_trades(refs.as_deref()).await?;
                }
                SubscriptionRequest::Fills(tickers) => {
                    let refs = tickers.as_ref().map(|t| {
                        t.iter().map(|s| s.as_str()).collect::<Vec<_>>()
                    });
                    client.subscribe_fills(refs.as_deref()).await?;
                }
                SubscriptionRequest::UserOrders => {
                    client.subscribe_user_orders().await?;
                }
                SubscriptionRequest::MarketLifecycle(tickers) => {
                    let refs = tickers.as_ref().map(|t| {
                        t.iter().map(|s| s.as_str()).collect::<Vec<_>>()
                    });
                    client.subscribe_market_lifecycle(refs.as_deref()).await?;
                }
            }
        }
        Ok(())
    }

    /// Manually trigger a reconnection
    ///
    /// Useful if you want to force a reconnect without waiting for an error.
    pub async fn reconnect(&mut self) -> Result<(), Error> {
        if let Some(ref mut client) = self.client {
            let _ = client.close().await;
        }
        self.client = None;
        self.reconnect_attempt = 0;
        self.attempt_reconnect().await
    }

    /// Close the WebSocket connection
    pub async fn close(&mut self) -> Result<(), Error> {
        if let Some(ref mut client) = self.client {
            client.close().await?;
        }
        self.client = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconnect_config_default() {
        let config = ReconnectConfig::default();
        assert_eq!(config.max_retries, 10);
        assert_eq!(config.initial_delay_ms, 100);
        assert_eq!(config.max_delay_ms, 30_000);
        assert!((config.backoff_multiplier - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_reconnect_config_builder() {
        let config = ReconnectConfig::new()
            .max_retries(5)
            .initial_delay_ms(50)
            .max_delay_ms(10_000)
            .backoff_multiplier(1.5);

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_delay_ms, 50);
        assert_eq!(config.max_delay_ms, 10_000);
        assert!((config.backoff_multiplier - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_delay_calculation() {
        let config = ReconnectConfig::new()
            .initial_delay_ms(100)
            .backoff_multiplier(2.0)
            .max_delay_ms(1000);

        assert_eq!(config.delay_for_attempt(0), std::time::Duration::from_millis(100));
        assert_eq!(config.delay_for_attempt(1), std::time::Duration::from_millis(200));
        assert_eq!(config.delay_for_attempt(2), std::time::Duration::from_millis(400));
        assert_eq!(config.delay_for_attempt(3), std::time::Duration::from_millis(800));
        // Should cap at max_delay_ms
        assert_eq!(config.delay_for_attempt(4), std::time::Duration::from_millis(1000));
        assert_eq!(config.delay_for_attempt(10), std::time::Duration::from_millis(1000));
    }
}
