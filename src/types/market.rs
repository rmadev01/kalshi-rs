#![allow(missing_docs)]

//! Market and portfolio types.

use serde::{Deserialize, Serialize};

use crate::types::{
    deserialize_count, deserialize_dollars, deserialize_optional_count,
    deserialize_optional_dollars,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum MarketStatus {
    Initialized,
    Inactive,
    Active,
    Closed,
    Determined,
    Disputed,
    Amended,
    Finalized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum MarketType {
    Binary,
    Scalar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum SettlementResult {
    Yes,
    No,
    Scalar,
    Void,
}

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

#[derive(Debug, Clone, Deserialize)]
pub struct Market {
    pub ticker: String,
    pub event_ticker: String,
    pub market_type: MarketType,
    pub title: String,
    pub subtitle: String,
    pub yes_sub_title: String,
    pub no_sub_title: String,
    pub status: MarketStatus,
    pub created_time: String,
    pub updated_time: String,
    pub open_time: String,
    pub close_time: String,
    pub expiration_time: String,
    pub latest_expiration_time: String,
    #[serde(default)]
    pub expected_expiration_time: Option<String>,
    pub settlement_timer_seconds: i64,
    #[serde(default)]
    pub series_ticker: Option<String>,
    #[serde(default)]
    pub response_price_units: Option<String>,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub notional_value_dollars: i64,
    #[serde(deserialize_with = "deserialize_optional_dollars")]
    pub yes_bid_dollars: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_count")]
    pub yes_bid_size_fp: Option<i64>,
    #[serde(deserialize_with = "deserialize_optional_dollars")]
    pub yes_ask_dollars: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_count")]
    pub yes_ask_size_fp: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_dollars")]
    pub no_bid_dollars: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_dollars")]
    pub no_ask_dollars: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_dollars")]
    pub last_price_dollars: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_dollars")]
    pub previous_yes_bid_dollars: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_dollars")]
    pub previous_yes_ask_dollars: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_dollars")]
    pub previous_price_dollars: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_count")]
    pub volume_fp: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_count")]
    pub volume_24h_fp: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_dollars")]
    pub liquidity_dollars: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_count")]
    pub open_interest_fp: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_settlement")]
    pub result: Option<SettlementResult>,
    pub can_close_early: bool,
    pub fractional_trading_enabled: bool,
    pub expiration_value: String,
    pub rules_primary: String,
    pub rules_secondary: String,
    #[serde(default)]
    pub tick_size: Option<i64>,
    #[serde(default)]
    pub strike_type: Option<String>,
    #[serde(default)]
    pub floor_strike: Option<f64>,
    #[serde(default)]
    pub cap_strike: Option<f64>,
    #[serde(default)]
    pub category: Option<String>,
}

impl Market {
    #[must_use]
    pub fn mid_price(&self) -> Option<i64> {
        match (self.yes_bid_dollars, self.yes_ask_dollars) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2),
            _ => None,
        }
    }

    #[must_use]
    pub fn spread(&self) -> Option<i64> {
        match (self.yes_bid_dollars, self.yes_ask_dollars) {
            (Some(bid), Some(ask)) => Some(ask.saturating_sub(bid)),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_tradeable(&self) -> bool {
        matches!(self.status, MarketStatus::Active)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Event {
    pub event_ticker: String,
    pub series_ticker: String,
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub sub_title: Option<String>,
    #[serde(default)]
    pub mutually_exclusive: bool,
    #[serde(default)]
    pub strike_date: Option<String>,
    #[serde(default)]
    pub markets: Vec<Market>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Series {
    pub ticker: String,
    pub title: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub settlement_sources: Vec<SettlementSource>,
    #[serde(default)]
    pub contract_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettlementSource {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMarketsResponse {
    pub markets: Vec<Market>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMarketResponse {
    pub market: Market,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetEventsResponse {
    pub events: Vec<Event>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetEventResponse {
    pub event: Event,
    #[serde(default)]
    pub markets: Vec<Market>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetSeriesResponse {
    pub series: Series,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetSeriesListResponse {
    pub series: Vec<Series>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Balance {
    pub balance: i64,
    pub portfolio_value: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetBalanceResponse {
    pub balance: i64,
    pub portfolio_value: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Position {
    pub ticker: String,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub total_traded_dollars: i64,
    #[serde(deserialize_with = "deserialize_count")]
    pub position_fp: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub market_exposure_dollars: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub realized_pnl_dollars: i64,
    pub resting_orders_count: i32,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub fees_paid_dollars: i64,
    #[serde(default)]
    pub last_updated_ts: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventPosition {
    pub event_ticker: String,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub total_cost_dollars: i64,
    #[serde(deserialize_with = "deserialize_count")]
    pub total_cost_shares_fp: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub event_exposure_dollars: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub realized_pnl_dollars: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub fees_paid_dollars: i64,
    #[serde(default)]
    pub resting_orders_count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetPositionsResponse {
    #[serde(default)]
    pub market_positions: Vec<Position>,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default)]
    pub event_positions: Vec<EventPosition>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Trade {
    pub trade_id: String,
    pub ticker: String,
    #[serde(default)]
    pub price: Option<i64>,
    #[serde(deserialize_with = "deserialize_count")]
    pub count_fp: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub yes_price_dollars: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub no_price_dollars: i64,
    pub taker_side: String,
    #[serde(default)]
    pub created_time: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetTradesResponse {
    pub trades: Vec<Trade>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Fill {
    pub fill_id: String,
    pub trade_id: String,
    pub order_id: String,
    #[serde(default)]
    pub client_order_id: Option<String>,
    pub ticker: String,
    pub market_ticker: String,
    pub side: String,
    pub action: String,
    #[serde(deserialize_with = "deserialize_count")]
    pub count_fp: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub yes_price_dollars: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub no_price_dollars: i64,
    pub is_taker: bool,
    #[serde(default)]
    pub created_time: Option<String>,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub fee_cost: i64,
    #[serde(default)]
    pub subaccount_number: Option<i32>,
    #[serde(default)]
    pub ts: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetFillsResponse {
    pub fills: Vec<Fill>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settlement {
    pub ticker: String,
    pub event_ticker: String,
    pub market_result: String,
    #[serde(deserialize_with = "deserialize_count")]
    pub yes_count_fp: i64,
    pub yes_total_cost: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub yes_total_cost_dollars: i64,
    #[serde(deserialize_with = "deserialize_count")]
    pub no_count_fp: i64,
    pub no_total_cost: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub no_total_cost_dollars: i64,
    pub revenue: i64,
    pub settled_time: String,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub fee_cost: i64,
    #[serde(default)]
    pub value: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetSettlementsResponse {
    pub settlements: Vec<Settlement>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookLevel {
    #[serde(deserialize_with = "deserialize_dollars")]
    pub price: i64,
    #[serde(deserialize_with = "deserialize_count")]
    pub quantity: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Orderbook {
    #[serde(default)]
    pub yes_dollars: Vec<[String; 2]>,
    #[serde(default)]
    pub no_dollars: Vec<[String; 2]>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetOrderbookResponse {
    pub orderbook_fp: Orderbook,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeStatus {
    pub trading_active: bool,
    pub exchange_active: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeSchedule {
    pub standard_hours: Vec<WeeklySchedule>,
    pub maintenance_windows: Vec<MaintenanceWindow>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WeeklySchedule {
    pub start_time: String,
    pub end_time: String,
    pub monday: Vec<DailySchedule>,
    pub tuesday: Vec<DailySchedule>,
    pub wednesday: Vec<DailySchedule>,
    pub thursday: Vec<DailySchedule>,
    pub friday: Vec<DailySchedule>,
    pub saturday: Vec<DailySchedule>,
    pub sunday: Vec<DailySchedule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DailySchedule {
    pub open_time: String,
    pub close_time: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MaintenanceWindow {
    pub start_datetime: String,
    pub end_datetime: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetExchangeScheduleResponse {
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
            market_type: MarketType::Binary,
            title: "Test".to_string(),
            subtitle: "Test".to_string(),
            yes_sub_title: "Yes".to_string(),
            no_sub_title: "No".to_string(),
            status: MarketStatus::Active,
            created_time: "2024-01-01T00:00:00Z".to_string(),
            updated_time: "2024-01-01T00:00:00Z".to_string(),
            open_time: "2024-01-01T00:00:00Z".to_string(),
            close_time: "2024-01-02T00:00:00Z".to_string(),
            expiration_time: "2024-01-02T00:00:00Z".to_string(),
            latest_expiration_time: "2024-01-02T00:00:00Z".to_string(),
            expected_expiration_time: None,
            settlement_timer_seconds: 60,
            series_ticker: None,
            response_price_units: None,
            notional_value_dollars: 10_000,
            yes_bid_dollars: Some(4_500),
            yes_bid_size_fp: Some(1_000),
            yes_ask_dollars: Some(5_500),
            yes_ask_size_fp: Some(1_000),
            no_bid_dollars: Some(4_500),
            no_ask_dollars: Some(5_500),
            last_price_dollars: Some(5_000),
            previous_yes_bid_dollars: None,
            previous_yes_ask_dollars: None,
            previous_price_dollars: None,
            volume_fp: Some(10_000),
            volume_24h_fp: Some(10_000),
            liquidity_dollars: Some(0),
            open_interest_fp: Some(5_000),
            result: None,
            can_close_early: false,
            fractional_trading_enabled: false,
            expiration_value: "".to_string(),
            rules_primary: "Primary".to_string(),
            rules_secondary: "Secondary".to_string(),
            tick_size: None,
            strike_type: None,
            floor_strike: None,
            cap_strike: None,
            category: None,
        };

        assert_eq!(market.mid_price(), Some(5_000));
        assert_eq!(market.spread(), Some(1_000));
        assert!(market.is_tradeable());
    }

    #[test]
    fn test_market_status_serde() {
        let json = serde_json::to_string(&MarketStatus::Active).unwrap();
        assert_eq!(json, "\"active\"");
    }
}
