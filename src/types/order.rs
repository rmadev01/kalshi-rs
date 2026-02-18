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
#[non_exhaustive]
pub enum Side {
    /// Yes contracts - pay out $1 if the event happens
    Yes,
    /// No contracts - pay out $1 if the event doesn't happen
    No,
}

impl Side {
    /// Get the opposite side
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Side::Yes => Side::No,
            Side::No => Side::Yes,
        }
    }
}

/// Order action (buy or sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Action {
    /// Buy contracts
    Buy,
    /// Sell contracts
    Sell,
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OrderType {
    /// Limit order - specify price and quantity
    #[default]
    Limit,
    /// Market order - execute at best available price
    Market,
}

/// Self-trade prevention type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
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
    pub count: i64,

    /// Order type (limit or market)
    #[serde(rename = "type")]
    pub order_type: OrderType,

    /// Limit price in centi-cents (required for limit orders)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<i64>,

    /// No price in centi-cents (alternative to yes_price)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price: Option<i64>,

    /// Expiration time in seconds from now
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_ts: Option<i64>,

    /// Self-trade prevention type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_trade_prevention_type: Option<SelfTradePrevention>,

    /// Buy max cost in centi-cents (for market orders)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_max_cost: Option<i64>,

    /// Sell position floor (minimum position to maintain)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_position_floor: Option<i64>,

    /// Time in force
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,

    /// Order group ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_group_id: Option<String>,

    /// Subaccount ID (0 = primary, 1-32 = subaccounts)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
}

/// Time-in-force options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TimeInForce {
    /// Good till canceled
    Gtc,
    /// Good till date
    Gtd,
    /// Immediate or cancel
    Ioc,
    /// Fill or kill
    Fok,
}

impl CreateOrderRequest {
    /// Create a new limit order request
    ///
    /// # Arguments
    /// * `ticker` - Market ticker
    /// * `side` - Yes or No
    /// * `action` - Buy or Sell
    /// * `count` - Number of contracts
    /// * `price_centicents` - Price in centi-cents (e.g., 5000 = $0.50)
    #[must_use]
    pub fn limit(
        ticker: impl Into<String>,
        side: Side,
        action: Action,
        count: i64,
        price_centicents: i64,
    ) -> Self {
        Self {
            ticker: ticker.into(),
            client_order_id: None,
            side,
            action,
            count,
            order_type: OrderType::Limit,
            yes_price: Some(price_centicents),
            no_price: None,
            expiration_ts: None,
            self_trade_prevention_type: None,
            buy_max_cost: None,
            sell_position_floor: None,
            time_in_force: None,
            order_group_id: None,
            subaccount: None,
        }
    }

    /// Create a new market order request
    #[must_use]
    pub fn market(ticker: impl Into<String>, side: Side, action: Action, count: i64) -> Self {
        Self {
            ticker: ticker.into(),
            client_order_id: None,
            side,
            action,
            count,
            order_type: OrderType::Market,
            yes_price: None,
            no_price: None,
            expiration_ts: None,
            self_trade_prevention_type: None,
            buy_max_cost: None,
            sell_position_floor: None,
            time_in_force: None,
            order_group_id: None,
            subaccount: None,
        }
    }

    /// Set a client order ID for idempotency
    #[must_use]
    pub fn with_client_order_id(mut self, id: impl Into<String>) -> Self {
        self.client_order_id = Some(id.into());
        self
    }

    /// Set the order group ID
    #[must_use]
    pub fn with_order_group(mut self, group_id: impl Into<String>) -> Self {
        self.order_group_id = Some(group_id.into());
        self
    }

    /// Set time-in-force
    #[must_use]
    pub fn with_time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = Some(tif);
        self
    }

    /// Set expiration time in seconds from now
    #[must_use]
    pub fn with_expiration_ts(mut self, ts: i64) -> Self {
        self.expiration_ts = Some(ts);
        self
    }

    /// Set subaccount
    #[must_use]
    pub fn with_subaccount(mut self, subaccount: i32) -> Self {
        self.subaccount = Some(subaccount);
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

    /// Price in centi-cents (for yes side)
    pub yes_price: i64,

    /// Price in centi-cents (for no side, computed as 10000 - yes_price)
    pub no_price: i64,

    /// Number of contracts filled
    #[serde(default)]
    pub fill_count: i64,

    /// Number of contracts remaining
    #[serde(default)]
    pub remaining_count: i64,

    /// Total count initially requested
    #[serde(default)]
    pub initial_count: Option<i64>,

    /// Queue position (if resting)
    pub queue_position: Option<i64>,

    /// Expiration time (ISO 8601)
    pub expiration_time: Option<String>,

    /// Time-in-force
    pub time_in_force: Option<String>,

    /// When the order was created
    pub created_time: Option<String>,

    /// When the order was last updated  
    pub updated_time: Option<String>,

    /// Subaccount ID
    pub subaccount: Option<i32>,

    /// Order group ID
    pub order_group_id: Option<String>,

    /// Decrease count (for decrease operations)
    pub decrease_count: Option<i64>,

    /// Maker fills
    pub maker_fill_count: Option<i64>,

    /// Taker fills
    pub taker_fill_count: Option<i64>,

    /// Maker fees in centi-cents
    pub maker_fees: Option<i64>,

    /// Taker fees in centi-cents
    pub taker_fees: Option<i64>,

    /// Amount spent/received in centi-cents
    pub total_cost: Option<i64>,
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
    /// New price in centi-cents (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<i64>,

    /// New count (must be >= current fill_count)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,

    /// Subaccount ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
}

/// Response from amending an order
#[derive(Debug, Clone, Deserialize)]
pub struct AmendOrderResponse {
    /// The amended order
    pub order: Order,
}

/// Request to decrease an order's quantity
#[derive(Debug, Clone, Serialize)]
pub struct DecreaseOrderRequest {
    /// Amount to reduce by
    pub reduce_by: i64,

    /// Subaccount ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
}

/// Response from decreasing an order
#[derive(Debug, Clone, Deserialize)]
pub struct DecreaseOrderResponse {
    /// The decreased order
    pub order: Order,
}

/// Response from getting orders
#[derive(Debug, Clone, Deserialize)]
pub struct GetOrdersResponse {
    /// List of orders
    pub orders: Vec<Order>,

    /// Cursor for pagination
    pub cursor: Option<String>,
}

/// Response from getting a single order
#[derive(Debug, Clone, Deserialize)]
pub struct GetOrderResponse {
    /// The order
    pub order: Order,
}

/// Request to batch create orders
#[derive(Debug, Clone, Serialize)]
pub struct BatchCreateOrdersRequest {
    /// List of order requests
    pub orders: Vec<CreateOrderRequest>,
}

/// Result of a single order in a batch
#[derive(Debug, Clone, Deserialize)]
pub struct BatchOrderResult {
    /// The order (if successful)
    pub order: Option<Order>,

    /// Error message (if failed)
    pub error: Option<BatchOrderError>,
}

/// Error in batch order
#[derive(Debug, Clone, Deserialize)]
pub struct BatchOrderError {
    /// Error code
    pub code: Option<String>,

    /// Error message
    pub message: String,
}

/// Response from batch creating orders
#[derive(Debug, Clone, Deserialize)]
pub struct BatchCreateOrdersResponse {
    /// Results for each order
    pub orders: Vec<BatchOrderResult>,
}

/// Request to batch cancel orders
#[derive(Debug, Clone, Serialize)]
pub struct BatchCancelOrdersRequest {
    /// List of order IDs to cancel
    pub order_ids: Vec<String>,

    /// Subaccount ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
}

/// Result of a batch cancel operation
#[derive(Debug, Clone, Deserialize)]
pub struct BatchCancelResult {
    /// Order ID
    pub order_id: String,

    /// The canceled order (if successful)
    pub order: Option<Order>,

    /// Error message (if failed)
    pub error: Option<BatchOrderError>,
}

/// Response from batch canceling orders
#[derive(Debug, Clone, Deserialize)]
pub struct BatchCancelOrdersResponse {
    /// Results for each order
    pub orders: Vec<BatchCancelResult>,
}

/// Order queue position
#[derive(Debug, Clone, Deserialize)]
pub struct QueuePosition {
    /// Order ID
    pub order_id: String,

    /// Queue position (contracts ahead)
    pub queue_position: i64,
}

/// Response from getting queue positions
#[derive(Debug, Clone, Deserialize)]
pub struct GetOrderQueuePositionsResponse {
    /// Queue positions for orders
    pub queue_positions: Vec<QueuePosition>,
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
        let order = CreateOrderRequest::limit("KXBTC-25JAN", Side::Yes, Action::Buy, 10, 5500);
        assert_eq!(order.ticker, "KXBTC-25JAN");
        assert_eq!(order.side, Side::Yes);
        assert_eq!(order.action, Action::Buy);
        assert_eq!(order.count, 10);
        assert_eq!(order.yes_price, Some(5500));
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

    #[test]
    fn test_order_builder() {
        let order = CreateOrderRequest::limit("TEST", Side::Yes, Action::Buy, 10, 5000)
            .with_client_order_id("my-order-123")
            .with_time_in_force(TimeInForce::Gtc)
            .with_subaccount(1);

        assert_eq!(order.client_order_id, Some("my-order-123".to_string()));
        assert_eq!(order.time_in_force, Some(TimeInForce::Gtc));
        assert_eq!(order.subaccount, Some(1));
    }
}
