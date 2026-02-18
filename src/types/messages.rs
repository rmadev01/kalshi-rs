//! WebSocket message types.
//!
//! This module contains types for WebSocket commands sent to Kalshi
//! and messages received from the WebSocket API.

use serde::{Deserialize, Serialize};

use super::order::Side;
use super::{Price, Quantity, TimestampMs};

/// WebSocket command sent to the server
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum WsCommand {
    /// Subscribe to channels
    Subscribe {
        /// Message ID
        id: u64,
        /// Subscription parameters
        params: SubscribeParams,
    },
    /// Unsubscribe from channels
    Unsubscribe {
        /// Message ID
        id: u64,
        /// Subscription IDs to unsubscribe
        params: UnsubscribeParams,
    },
    /// List current subscriptions
    ListSubscriptions {
        /// Message ID
        id: u64,
    },
}

/// Parameters for subscribe command
#[derive(Debug, Clone, Serialize)]
pub struct SubscribeParams {
    /// Channels to subscribe to
    pub channels: Vec<String>,
    /// Market tickers (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_tickers: Option<Vec<String>>,
}

/// Parameters for unsubscribe command
#[derive(Debug, Clone, Serialize)]
pub struct UnsubscribeParams {
    /// Subscription IDs to unsubscribe
    pub sids: Vec<u64>,
}

/// WebSocket message received from the server
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// Subscription confirmed
    Subscribed(SubscribedMsg),
    /// Unsubscription confirmed
    Unsubscribed(UnsubscribedMsg),
    /// Error response
    Error(ErrorMsg),
    /// Orderbook snapshot (full book state)
    OrderbookSnapshot(OrderbookSnapshotMsg),
    /// Orderbook delta (incremental update)
    OrderbookDelta(OrderbookDeltaMsg),
    /// Ticker update
    Ticker(TickerMsg),
    /// Trade occurred
    Trade(TradeMsg),
    /// Fill notification (your order was filled)
    Fill(FillMsg),
    /// User order update
    UserOrder(UserOrderMsg),
}

/// Subscription confirmed message
#[derive(Debug, Clone, Deserialize)]
pub struct SubscribedMsg {
    /// Message ID (matches the request)
    pub id: Option<u64>,
    /// Subscription details
    pub msg: SubscriptionInfo,
}

/// Subscription info
#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionInfo {
    /// Channel name
    pub channel: String,
    /// Subscription ID (use to unsubscribe)
    pub sid: u64,
}

/// Unsubscription confirmed message
#[derive(Debug, Clone, Deserialize)]
pub struct UnsubscribedMsg {
    /// Message ID
    pub id: Option<u64>,
    /// Subscription ID that was unsubscribed
    pub sid: u64,
}

/// Error message
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorMsg {
    /// Message ID
    pub id: Option<u64>,
    /// Error details
    pub msg: ErrorDetails,
}

/// Error details
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorDetails {
    /// Error code
    pub code: u32,
    /// Error message
    pub msg: String,
}

/// Orderbook snapshot message
///
/// Contains the full state of the orderbook for a market.
/// Price levels are represented as [price_cents, quantity] pairs.
#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookSnapshotMsg {
    /// Subscription ID
    pub sid: u64,
    /// Sequence number
    pub seq: u64,
    /// Snapshot data
    pub msg: OrderbookSnapshotData,
}

/// Orderbook snapshot data
#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookSnapshotData {
    /// Market ticker
    pub market_ticker: String,
    /// Yes side bids: [[price, quantity], ...]
    pub yes: Vec<[u64; 2]>,
    /// No side bids: [[price, quantity], ...]
    pub no: Vec<[u64; 2]>,
}

/// Orderbook delta message
///
/// Contains an incremental update to apply to the orderbook.
#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookDeltaMsg {
    /// Subscription ID
    pub sid: u64,
    /// Sequence number (use to detect gaps)
    pub seq: u64,
    /// Delta data
    pub msg: OrderbookDeltaData,
}

/// Orderbook delta data
#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookDeltaData {
    /// Market ticker
    pub market_ticker: String,
    /// Price level that changed (in cents)
    pub price: Price,
    /// Change in quantity (can be negative)
    pub delta: i64,
    /// Side that changed
    pub side: Side,
    /// Timestamp
    pub ts: Option<String>,
}

/// Ticker update message
#[derive(Debug, Clone, Deserialize)]
pub struct TickerMsg {
    /// Subscription ID
    pub sid: u64,
    /// Ticker data
    pub msg: TickerData,
}

/// Ticker data
#[derive(Debug, Clone, Deserialize)]
pub struct TickerData {
    /// Market ticker
    pub market_ticker: String,
    /// Last price in cents
    pub price: Option<Price>,
    /// Yes bid price
    pub yes_bid: Option<Price>,
    /// Yes ask price
    pub yes_ask: Option<Price>,
    /// 24h volume
    pub volume: Option<u64>,
    /// Open interest
    pub open_interest: Option<u64>,
    /// Timestamp (Unix ms)
    pub ts: Option<TimestampMs>,
}

/// Trade message
#[derive(Debug, Clone, Deserialize)]
pub struct TradeMsg {
    /// Subscription ID
    pub sid: u64,
    /// Trade data
    pub msg: TradeData,
}

/// Trade data
#[derive(Debug, Clone, Deserialize)]
pub struct TradeData {
    /// Trade ID
    pub trade_id: String,
    /// Market ticker
    pub market_ticker: String,
    /// Yes price in cents
    pub yes_price: Price,
    /// No price in cents (100 - yes_price)
    pub no_price: Price,
    /// Number of contracts
    pub count: Quantity,
    /// Which side was the taker
    pub taker_side: Side,
    /// Timestamp (Unix seconds)
    pub ts: TimestampMs,
}

/// Fill message (your order was filled)
#[derive(Debug, Clone, Deserialize)]
pub struct FillMsg {
    /// Subscription ID
    pub sid: u64,
    /// Fill data
    pub msg: FillData,
}

/// Fill data
#[derive(Debug, Clone, Deserialize)]
pub struct FillData {
    /// Trade ID
    pub trade_id: String,
    /// Order ID
    pub order_id: String,
    /// Market ticker
    pub market_ticker: String,
    /// Whether you were the taker
    pub is_taker: bool,
    /// Side (yes/no)
    pub side: Side,
    /// Yes price in cents
    pub yes_price: Price,
    /// Number of contracts filled
    pub count: Quantity,
    /// Action (buy/sell)
    pub action: String,
    /// Timestamp
    pub ts: TimestampMs,
}

/// User order update message
#[derive(Debug, Clone, Deserialize)]
pub struct UserOrderMsg {
    /// Subscription ID
    pub sid: u64,
    /// Order data
    pub msg: UserOrderData,
}

/// User order data
#[derive(Debug, Clone, Deserialize)]
pub struct UserOrderData {
    /// Order ID
    pub order_id: String,
    /// Market ticker
    pub ticker: String,
    /// Order status
    pub status: String,
    /// Side (yes/no)
    pub side: Side,
    /// Client order ID (if provided)
    pub client_order_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscribe_command_serialization() {
        let cmd = WsCommand::Subscribe {
            id: 1,
            params: SubscribeParams {
                channels: vec!["orderbook_delta".to_string()],
                market_tickers: Some(vec!["KXBTC-25JAN".to_string()]),
            },
        };

        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("subscribe"));
        assert!(json.contains("orderbook_delta"));
        assert!(json.contains("KXBTC-25JAN"));
    }

    #[test]
    fn test_orderbook_delta_deserialization() {
        let json = r#"{
            "type": "orderbook_delta",
            "sid": 1,
            "seq": 42,
            "msg": {
                "market_ticker": "KXBTC-25JAN",
                "price": 55,
                "delta": -10,
                "side": "yes",
                "ts": "2024-01-15T12:00:00Z"
            }
        }"#;

        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::OrderbookDelta(delta) => {
                assert_eq!(delta.seq, 42);
                assert_eq!(delta.msg.market_ticker, "KXBTC-25JAN");
                assert_eq!(delta.msg.price, 55);
                assert_eq!(delta.msg.delta, -10);
            }
            _ => panic!("Expected OrderbookDelta"),
        }
    }
}
