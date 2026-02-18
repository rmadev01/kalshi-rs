//! Market and event types.
//!
//! This module contains types representing Kalshi markets and events.

use serde::{Deserialize, Serialize};

/// Market status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketStatus {
    /// Market is open for trading
    Open,
    /// Market is closed (no more trading)
    Closed,
    /// Market has been settled
    Settled,
}

/// Settlement result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SettlementResult {
    /// Yes contracts paid out
    Yes,
    /// No contracts paid out
    No,
}

/// A Kalshi market (binary contract)
#[derive(Debug, Clone, Deserialize)]
pub struct Market {
    /// Unique market ticker (e.g., "KXBTC-25JAN-T50000")
    pub ticker: String,

    /// Event ticker this market belongs to
    pub event_ticker: String,

    /// Market title/question
    pub title: String,

    /// Subtitle (short description)
    pub subtitle: String,

    /// Market status
    pub status: MarketStatus,

    /// Yes bid price in cents
    pub yes_bid: Option<u8>,

    /// Yes ask price in cents
    pub yes_ask: Option<u8>,

    /// Last trade price in cents
    pub last_price: Option<u8>,

    /// 24h volume (number of contracts traded)
    pub volume: u64,

    /// 24h volume in dollars (cents)
    pub dollar_volume: u64,

    /// Open interest (contracts outstanding)
    pub open_interest: u64,

    /// When trading opens (ISO 8601)
    pub open_time: Option<String>,

    /// When trading closes (ISO 8601)
    pub close_time: Option<String>,

    /// Expected expiration (ISO 8601)
    pub expected_expiration_time: Option<String>,

    /// Settlement result (if settled)
    pub result: Option<SettlementResult>,

    /// Whether the market can close early
    pub can_close_early: bool,
}

impl Market {
    /// Get the mid price (average of bid and ask)
    pub fn mid_price(&self) -> Option<u8> {
        match (self.yes_bid, self.yes_ask) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2),
            _ => None,
        }
    }

    /// Get the spread in cents
    pub fn spread(&self) -> Option<u8> {
        match (self.yes_bid, self.yes_ask) {
            (Some(bid), Some(ask)) => Some(ask.saturating_sub(bid)),
            _ => None,
        }
    }

    /// Check if the market is tradeable
    pub fn is_tradeable(&self) -> bool {
        self.status == MarketStatus::Open
    }
}

/// A Kalshi event (container for multiple markets)
#[derive(Debug, Clone, Deserialize)]
pub struct Event {
    /// Unique event ticker
    pub event_ticker: String,

    /// Series ticker this event belongs to
    pub series_ticker: String,

    /// Event title
    pub title: String,

    /// Event subtitle
    pub subtitle: String,

    /// Category (e.g., "Crypto", "Economics")
    pub category: Option<String>,

    /// Number of markets in this event
    pub market_count: u32,
}

/// Response from GetMarkets endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct GetMarketsResponse {
    /// List of markets
    pub markets: Vec<Market>,

    /// Cursor for pagination
    pub cursor: Option<String>,
}

/// Response from GetMarket endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct GetMarketResponse {
    /// The market
    pub market: Market,
}

/// Response from GetEvents endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct GetEventsResponse {
    /// List of events
    pub events: Vec<Event>,

    /// Cursor for pagination
    pub cursor: Option<String>,
}

/// Balance information
#[derive(Debug, Clone, Deserialize)]
pub struct Balance {
    /// Available balance in cents
    pub balance: i64,

    /// Portfolio value in cents
    pub portfolio_value: i64,
}

/// Response from GetBalance endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct GetBalanceResponse {
    /// Balance in cents
    pub balance: i64,

    /// Portfolio value in cents
    pub portfolio_value: i64,
}

/// Position in a market
#[derive(Debug, Clone, Deserialize)]
pub struct Position {
    /// Market ticker
    pub ticker: String,

    /// Event ticker
    pub event_ticker: String,

    /// Position (positive = long, negative = short)
    pub position: i64,

    /// Cost basis in cents
    pub position_cost: i64,

    /// Realized P&L in cents
    pub realized_pnl: i64,

    /// Fees paid in cents
    pub fees_paid: i64,
}

/// Response from GetPositions endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct GetPositionsResponse {
    /// List of positions
    pub positions: Vec<Position>,

    /// Cursor for pagination
    pub cursor: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_mid_price() {
        let market = Market {
            ticker: "TEST".to_string(),
            event_ticker: "TEST-EVENT".to_string(),
            title: "Test".to_string(),
            subtitle: "Test".to_string(),
            status: MarketStatus::Open,
            yes_bid: Some(45),
            yes_ask: Some(55),
            last_price: Some(50),
            volume: 1000,
            dollar_volume: 500,
            open_interest: 100,
            open_time: None,
            close_time: None,
            expected_expiration_time: None,
            result: None,
            can_close_early: false,
        };

        assert_eq!(market.mid_price(), Some(50));
        assert_eq!(market.spread(), Some(10));
        assert!(market.is_tradeable());
    }

    #[test]
    fn test_market_status_serde() {
        let json = serde_json::to_string(&MarketStatus::Open).unwrap();
        assert_eq!(json, "\"open\"");
    }
}
