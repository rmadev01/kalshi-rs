//! Order-related types.
//!
//! This module contains types for creating, managing, and representing orders
//! on the Kalshi exchange.

use serde::{Deserialize, Serialize};

/// Order side (Yes or No contract)
///
/// In Kalshi, every market is a binary contract where you can buy/sell
/// either YES or NO contracts. The prices always sum to 100 cents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    /// Yes contracts - pay out $1 if the event happens
    Yes,
    /// No contracts - pay out $1 if the event doesn't happen
    No,
}

impl Side {
    /// Get the opposite side
    pub fn opposite(self) -> Self {
        match self {
            Side::Yes => Side::No,
            Side::No => Side::Yes,
        }
    }
}

/// Order action (buy or sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    /// Buy contracts
    Buy,
    /// Sell contracts
    Sell,
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    /// Order is resting on the book
    Resting,
    /// Order has been canceled
    Canceled,
    /// Order has been fully executed
    Executed,
    /// Order is pending (being processed)
    Pending,
}

/// Order type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    /// Limit order - specify price and quantity
    Limit,
    /// Market order - execute at best available price
    Market,
}

impl Default for OrderType {
    fn default() -> Self {
        OrderType::Limit
    }
}

/// Self-trade prevention type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelfTradePrevention {
    /// Cancel the resting order, execute the new order
    CancelResting,
    /// Cancel the new (taker) order if it would self-trade
    CancelTaker,
}

/// Request to create a new order
#[derive(Debug, Clone, Serialize)]
pub struct CreateOrderRequest {
    /// Market ticker
    pub ticker: String,

    /// Client-generated order ID (optional, for idempotency)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,

    /// Order side (yes or no)
    pub side: Side,

    /// Order action (buy or sell)
    pub action: Action,

    /// Number of contracts
    pub count: u32,

    /// Order type (limit or market)
    #[serde(rename = "type")]
    pub order_type: OrderType,

    /// Limit price in cents (required for limit orders)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<u8>,

    /// Expiration time (ISO 8601, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_ts: Option<String>,

    /// Self-trade prevention type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_trade_prevention_type: Option<SelfTradePrevention>,
}

impl CreateOrderRequest {
    /// Create a new limit order request
    pub fn limit(
        ticker: impl Into<String>,
        side: Side,
        action: Action,
        count: u32,
        price_cents: u8,
    ) -> Self {
        Self {
            ticker: ticker.into(),
            client_order_id: None,
            side,
            action,
            count,
            order_type: OrderType::Limit,
            yes_price: Some(price_cents),
            expiration_ts: None,
            self_trade_prevention_type: None,
        }
    }

    /// Create a new market order request
    pub fn market(ticker: impl Into<String>, side: Side, action: Action, count: u32) -> Self {
        Self {
            ticker: ticker.into(),
            client_order_id: None,
            side,
            action,
            count,
            order_type: OrderType::Market,
            yes_price: None,
            expiration_ts: None,
            self_trade_prevention_type: None,
        }
    }

    /// Set a client order ID for idempotency
    pub fn with_client_order_id(mut self, id: impl Into<String>) -> Self {
        self.client_order_id = Some(id.into());
        self
    }
}

/// An order on the Kalshi exchange
#[derive(Debug, Clone, Deserialize)]
pub struct Order {
    /// Server-generated order ID
    pub order_id: String,

    /// Client-generated order ID (if provided)
    pub client_order_id: Option<String>,

    /// User ID that owns this order
    pub user_id: Option<String>,

    /// Market ticker
    pub ticker: String,

    /// Order status
    pub status: OrderStatus,

    /// Order side
    pub side: Side,

    /// Order action
    pub action: Action,

    /// Order type
    #[serde(rename = "type")]
    pub order_type: OrderType,

    /// Price in cents (for yes side)
    pub yes_price: u8,

    /// Price in cents (for no side, computed as 100 - yes_price)
    pub no_price: u8,

    /// Number of contracts filled
    pub fill_count: u32,

    /// Number of contracts remaining
    pub remaining_count: u32,

    /// Total count initially requested
    pub initial_count: u32,

    /// When the order was created
    pub created_time: Option<String>,

    /// When the order was last updated
    pub updated_time: Option<String>,

    /// When the order expires (if set)
    pub expiration_time: Option<String>,
}

/// Response from creating an order
#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrderResponse {
    /// The created order
    pub order: Order,
}

/// Response from canceling an order
#[derive(Debug, Clone, Deserialize)]
pub struct CancelOrderResponse {
    /// The canceled order
    pub order: Order,
}

/// Request to amend an existing order
#[derive(Debug, Clone, Serialize)]
pub struct AmendOrderRequest {
    /// New price in cents (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<u8>,

    /// New count (must be >= current fill_count)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_side_opposite() {
        assert_eq!(Side::Yes.opposite(), Side::No);
        assert_eq!(Side::No.opposite(), Side::Yes);
    }

    #[test]
    fn test_create_limit_order() {
        let order = CreateOrderRequest::limit("KXBTC-25JAN", Side::Yes, Action::Buy, 10, 55);
        assert_eq!(order.ticker, "KXBTC-25JAN");
        assert_eq!(order.side, Side::Yes);
        assert_eq!(order.action, Action::Buy);
        assert_eq!(order.count, 10);
        assert_eq!(order.yes_price, Some(55));
        assert_eq!(order.order_type, OrderType::Limit);
    }

    #[test]
    fn test_create_market_order() {
        let order = CreateOrderRequest::market("KXBTC-25JAN", Side::No, Action::Sell, 5);
        assert_eq!(order.order_type, OrderType::Market);
        assert_eq!(order.yes_price, None);
    }

    #[test]
    fn test_serde_side() {
        let json = serde_json::to_string(&Side::Yes).unwrap();
        assert_eq!(json, "\"yes\"");

        let side: Side = serde_json::from_str("\"no\"").unwrap();
        assert_eq!(side, Side::No);
    }
}
