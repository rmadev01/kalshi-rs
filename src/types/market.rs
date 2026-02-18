//! Market and event types.
//!
//! This module contains types representing Kalshi markets and events.

use serde::{Deserialize, Serialize};

/// Market status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketStatus {
    /// Market has not yet opened for trading
    Unopened,
    /// Market is open for trading
    Open,
    /// Market is active (alias for open)
    Active,
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

/// Deserialize helper that treats empty strings as None
fn deserialize_optional_settlement<'de, D>(
    deserializer: D,
) -> Result<Option<SettlementResult>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::IntoDeserializer;

    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => SettlementResult::deserialize(s.into_deserializer()).map(Some),
    }
}

/// A Kalshi market (binary contract)
#[derive(Debug, Clone, Deserialize)]
pub struct Market {
    /// Unique market ticker (e.g., "KXBTC-25JAN-T50000")
    pub ticker: String,

    /// Event ticker this market belongs to
    pub event_ticker: String,

    /// Series ticker this market belongs to
    #[serde(default)]
    pub series_ticker: Option<String>,

    /// Market title/question
    pub title: String,

    /// Subtitle (short description)
    pub subtitle: String,

    /// Market status
    pub status: MarketStatus,

    /// Yes bid price in cents (centi-cents in API, we convert)
    pub yes_bid: Option<i64>,

    /// Yes ask price in cents (centi-cents in API, we convert)
    pub yes_ask: Option<i64>,

    /// Last trade price in cents
    pub last_price: Option<i64>,

    /// Previous yes bid
    pub previous_yes_bid: Option<i64>,

    /// Previous yes ask
    pub previous_yes_ask: Option<i64>,

    /// Previous price
    pub previous_price: Option<i64>,

    /// 24h volume (number of contracts traded)
    #[serde(default)]
    pub volume: i64,

    /// 24h volume in centi-cents
    #[serde(default)]
    pub dollar_volume: i64,

    /// Open interest (contracts outstanding)
    #[serde(default)]
    pub open_interest: i64,

    /// When trading opens (ISO 8601)
    pub open_time: Option<String>,

    /// When trading closes (ISO 8601)
    pub close_time: Option<String>,

    /// Expected expiration (ISO 8601)
    pub expected_expiration_time: Option<String>,

    /// Settlement result (if settled)
    #[serde(default, deserialize_with = "deserialize_optional_settlement")]
    pub result: Option<SettlementResult>,

    /// Whether the market can close early
    #[serde(default)]
    pub can_close_early: bool,

    /// Cap strike (for ranged markets)
    pub cap_strike: Option<f64>,

    /// Floor strike (for ranged markets)
    pub floor_strike: Option<f64>,

    /// Yes sub-title
    pub yes_sub_title: Option<String>,

    /// No sub-title
    pub no_sub_title: Option<String>,

    /// Risk limit in cents
    pub risk_limit_cents: Option<i64>,

    /// Notional value
    pub notional_value: Option<i64>,

    /// Tick size in centi-cents
    pub tick_size: Option<i64>,

    /// Maker fee percentage (basis points)
    pub maker_fee_bps: Option<i64>,

    /// Taker fee percentage (basis points)
    pub taker_fee_bps: Option<i64>,

    /// Settlement timer in seconds
    pub settlement_timer_seconds: Option<i64>,

    /// Expiration value
    pub expiration_value: Option<String>,

    /// Category
    pub category: Option<String>,

    /// Rules primary
    pub rules_primary: Option<String>,

    /// Rules secondary
    pub rules_secondary: Option<String>,
}

impl Market {
    /// Get the mid price in centi-cents (average of bid and ask)
    pub fn mid_price(&self) -> Option<i64> {
        match (self.yes_bid, self.yes_ask) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2),
            _ => None,
        }
    }

    /// Get the spread in centi-cents
    pub fn spread(&self) -> Option<i64> {
        match (self.yes_bid, self.yes_ask) {
            (Some(bid), Some(ask)) => Some(ask.saturating_sub(bid)),
            _ => None,
        }
    }

    /// Check if the market is tradeable
    pub fn is_tradeable(&self) -> bool {
        matches!(self.status, MarketStatus::Open | MarketStatus::Active)
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
    #[serde(default)]
    pub subtitle: Option<String>,

    /// Category (e.g., "Crypto", "Economics")
    pub category: Option<String>,

    /// Sub-title for the event
    pub sub_title: Option<String>,

    /// Mutually exclusive flag
    #[serde(default)]
    pub mutually_exclusive: bool,

    /// Strike date (ISO 8601)
    pub strike_date: Option<String>,

    /// Markets in this event
    #[serde(default)]
    pub markets: Vec<Market>,
}

/// A Kalshi series (template for recurring events)
#[derive(Debug, Clone, Deserialize)]
pub struct Series {
    /// Unique series ticker
    pub ticker: String,

    /// Series title
    pub title: String,

    /// Series category
    pub category: Option<String>,

    /// Tags associated with this series
    #[serde(default)]
    pub tags: Vec<String>,

    /// Settlement sources
    #[serde(default)]
    pub settlement_sources: Vec<SettlementSource>,

    /// Contract URL
    pub contract_url: Option<String>,
}

/// Settlement source information
#[derive(Debug, Clone, Deserialize)]
pub struct SettlementSource {
    /// Source URL
    pub url: Option<String>,

    /// Source name
    pub name: Option<String>,
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

/// Response from GetEvent endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct GetEventResponse {
    /// The event
    pub event: Event,
}

/// Response from GetSeries endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct GetSeriesResponse {
    /// The series
    pub series: Series,
}

/// Response from GetSeriesList endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct GetSeriesListResponse {
    /// List of series
    pub series: Vec<Series>,

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
    #[serde(default)]
    pub market_positions: Vec<Position>,

    /// Cursor for pagination
    pub cursor: Option<String>,

    /// Event positions (aggregated by event)
    #[serde(default)]
    pub event_positions: Vec<EventPosition>,
}

/// Event-level position aggregation
#[derive(Debug, Clone, Deserialize)]
pub struct EventPosition {
    /// Event ticker
    pub event_ticker: String,

    /// Event exposure in centi-cents
    pub event_exposure: i64,

    /// Realized P&L in centi-cents
    pub realized_pnl: i64,

    /// Fees paid in centi-cents
    pub fees_paid: i64,

    /// Total cost in centi-cents
    pub total_cost: i64,

    /// Resting order count
    #[serde(default)]
    pub resting_order_count: i64,
}

/// A trade on the exchange
#[derive(Debug, Clone, Deserialize)]
pub struct Trade {
    /// Trade ID
    pub trade_id: Option<String>,

    /// Market ticker
    pub ticker: String,

    /// Number of contracts traded
    pub count: i64,

    /// Yes price in centi-cents
    pub yes_price: i64,

    /// No price in centi-cents
    pub no_price: i64,

    /// Taker side (yes or no)
    pub taker_side: Option<String>,

    /// Timestamp when trade occurred
    pub created_time: Option<String>,
}

/// Response from GetTrades endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct GetTradesResponse {
    /// List of trades
    pub trades: Vec<Trade>,

    /// Cursor for pagination
    pub cursor: Option<String>,
}

/// A fill (your order matched)
#[derive(Debug, Clone, Deserialize)]
pub struct Fill {
    /// Trade ID
    pub trade_id: Option<String>,

    /// Order ID
    pub order_id: String,

    /// Market ticker
    pub ticker: String,

    /// Side (yes or no)
    pub side: String,

    /// Action (buy or sell)
    pub action: String,

    /// Number of contracts filled
    pub count: i64,

    /// Yes price in centi-cents
    pub yes_price: i64,

    /// No price in centi-cents
    pub no_price: i64,

    /// Whether you were the taker
    pub is_taker: bool,

    /// Timestamp when fill occurred
    pub created_time: Option<String>,
}

/// Response from GetFills endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct GetFillsResponse {
    /// List of fills
    pub fills: Vec<Fill>,

    /// Cursor for pagination
    pub cursor: Option<String>,
}

/// Settlement record
#[derive(Debug, Clone, Deserialize)]
pub struct Settlement {
    /// Market ticker
    pub ticker: String,

    /// Settlement result (yes or no)
    pub result: String,

    /// Number of contracts settled
    pub count: i64,

    /// Revenue from settlement in centi-cents
    pub revenue: i64,

    /// Timestamp when settled
    pub settled_time: Option<String>,
}

/// Response from GetSettlements endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct GetSettlementsResponse {
    /// List of settlements
    pub settlements: Vec<Settlement>,

    /// Cursor for pagination
    pub cursor: Option<String>,
}

/// Orderbook level (price and quantity)
#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookLevel {
    /// Price in centi-cents
    pub price: i64,

    /// Total quantity at this price level
    #[serde(default)]
    pub quantity: i64,
}

/// Market orderbook
#[derive(Debug, Clone, Deserialize)]
pub struct Orderbook {
    /// Market ticker
    pub ticker: String,

    /// Yes bids (sorted best to worst, highest price first)
    #[serde(default)]
    pub yes: Vec<Vec<i64>>,

    /// No bids (sorted best to worst, highest price first)
    #[serde(default)]
    pub no: Vec<Vec<i64>>,
}

/// Response from GetOrderbook endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct GetOrderbookResponse {
    /// The orderbook
    pub orderbook: Orderbook,
}

/// Exchange status
#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeStatus {
    /// Whether the exchange is in trading mode
    pub trading_active: bool,

    /// Whether the exchange is available
    pub exchange_active: bool,
}

/// Exchange schedule
#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeSchedule {
    /// Standard hours
    pub standard_hours: Option<ScheduleHours>,

    /// Next maintenance window
    pub maintenance_windows: Option<Vec<MaintenanceWindow>>,
}

/// Schedule hours
#[derive(Debug, Clone, Deserialize)]
pub struct ScheduleHours {
    /// Open time
    pub open_time: Option<String>,

    /// Close time
    pub close_time: Option<String>,
}

/// Maintenance window
#[derive(Debug, Clone, Deserialize)]
pub struct MaintenanceWindow {
    /// Start time
    pub start_time: Option<String>,

    /// End time
    pub end_time: Option<String>,
}

/// Response from GetExchangeSchedule endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct GetExchangeScheduleResponse {
    /// The schedule
    pub schedule: ExchangeSchedule,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_mid_price() {
        let market = Market {
            ticker: "TEST".to_string(),
            event_ticker: "TEST-EVENT".to_string(),
            series_ticker: None,
            title: "Test".to_string(),
            subtitle: "Test".to_string(),
            status: MarketStatus::Open,
            yes_bid: Some(4500),
            yes_ask: Some(5500),
            last_price: Some(5000),
            previous_yes_bid: None,
            previous_yes_ask: None,
            previous_price: None,
            volume: 1000,
            dollar_volume: 500,
            open_interest: 100,
            open_time: None,
            close_time: None,
            expected_expiration_time: None,
            result: None,
            can_close_early: false,
            cap_strike: None,
            floor_strike: None,
            yes_sub_title: None,
            no_sub_title: None,
            risk_limit_cents: None,
            notional_value: None,
            tick_size: None,
            maker_fee_bps: None,
            taker_fee_bps: None,
            settlement_timer_seconds: None,
            expiration_value: None,
            category: None,
            rules_primary: None,
            rules_secondary: None,
        };

        assert_eq!(market.mid_price(), Some(5000));
        assert_eq!(market.spread(), Some(1000));
        assert!(market.is_tradeable());
    }

    #[test]
    fn test_market_status_serde() {
        let json = serde_json::to_string(&MarketStatus::Open).unwrap();
        assert_eq!(json, "\"open\"");
    }
}
