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

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::http::Request;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::client::auth::Signer;
use crate::config::Config;
use crate::error::Error;
use crate::types::messages::{WsCommand, WsMessage};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// WebSocket client for real-time market data
#[derive(Debug)]
pub struct WebSocketClient {
    write: SplitSink<WsStream, Message>,
    read: SplitStream<WsStream>,
    message_id: u64,
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
        })
    }

    /// Send a command to the WebSocket server
    async fn send_command(&mut self, cmd: WsCommand) -> Result<(), Error> {
        let json = serde_json::to_string(&cmd)?;
        self.write.send(Message::Text(json)).await?;
        self.message_id += 1;
        Ok(())
    }

    /// Subscribe to orderbook updates for the given markets
    ///
    /// # Arguments
    ///
    /// * `market_tickers` - Market tickers to subscribe to
    pub async fn subscribe_orderbook(&mut self, market_tickers: &[&str]) -> Result<(), Error> {
        let cmd = WsCommand::Subscribe {
            id: self.message_id,
            params: crate::types::messages::SubscribeParams {
                channels: vec!["orderbook_delta".to_string()],
                market_tickers: Some(market_tickers.iter().map(|s| s.to_string()).collect()),
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
    ) -> Result<(), Error> {
        let cmd = WsCommand::Subscribe {
            id: self.message_id,
            params: crate::types::messages::SubscribeParams {
                channels: vec!["ticker".to_string()],
                market_tickers: market_tickers
                    .map(|t| t.iter().map(|s| s.to_string()).collect()),
            },
        };
        self.send_command(cmd).await
    }

    /// Subscribe to trade updates
    pub async fn subscribe_trades(
        &mut self,
        market_tickers: Option<&[&str]>,
    ) -> Result<(), Error> {
        let cmd = WsCommand::Subscribe {
            id: self.message_id,
            params: crate::types::messages::SubscribeParams {
                channels: vec!["trade".to_string()],
                market_tickers: market_tickers
                    .map(|t| t.iter().map(|s| s.to_string()).collect()),
            },
        };
        self.send_command(cmd).await
    }

    /// Subscribe to fill notifications (your trades)
    pub async fn subscribe_fills(
        &mut self,
        market_tickers: Option<&[&str]>,
    ) -> Result<(), Error> {
        let cmd = WsCommand::Subscribe {
            id: self.message_id,
            params: crate::types::messages::SubscribeParams {
                channels: vec!["fill".to_string()],
                market_tickers: market_tickers
                    .map(|t| t.iter().map(|s| s.to_string()).collect()),
            },
        };
        self.send_command(cmd).await
    }

    /// Receive the next message from the WebSocket
    ///
    /// # Returns
    ///
    /// The next message, or `None` if the connection is closed.
    pub async fn next(&mut self) -> Option<Result<WsMessage, Error>> {
        loop {
            match self.read.next().await? {
                Ok(Message::Text(text)) => {
                    return Some(serde_json::from_str(&text).map_err(Error::from));
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

#[cfg(test)]
mod tests {
    // Integration tests would go here with mock WebSocket server
}
