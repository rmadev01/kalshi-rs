#![allow(missing_docs)]

//! Order-related types.

use serde::{Deserialize, Serialize};

use crate::types::{
    deserialize_count, deserialize_dollars, deserialize_optional_count, serialize_optional_count,
    serialize_optional_dollars,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Side {
    Yes,
    No,
}

impl Side {
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Side::Yes => Side::No,
            Side::No => Side::Yes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Action {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OrderStatus {
    Resting,
    Canceled,
    Executed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OrderType {
    #[default]
    Limit,
    Market,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SelfTradePrevention {
    TakerAtCross,
    Maker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TimeInForce {
    FillOrKill,
    GoodTillCanceled,
    ImmediateOrCancel,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateOrderRequest {
    pub ticker: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    pub side: Side,
    pub action: Action,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_optional_count"
    )]
    pub count_fp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price: Option<i64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_optional_dollars"
    )]
    pub yes_price_dollars: Option<i64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_optional_dollars"
    )]
    pub no_price_dollars: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_max_cost: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_position_floor: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_trade_prevention_type: Option<SelfTradePrevention>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_order_on_pause: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
}

impl CreateOrderRequest {
    #[must_use]
    pub fn limit(
        ticker: impl Into<String>,
        side: Side,
        action: Action,
        count: i64,
        price_ten_thousandths: i64,
    ) -> Self {
        Self {
            ticker: ticker.into(),
            client_order_id: None,
            side,
            action,
            count: Some(count),
            count_fp: Some(count * 100),
            yes_price: None,
            no_price: None,
            yes_price_dollars: Some(price_ten_thousandths),
            no_price_dollars: None,
            expiration_ts: None,
            time_in_force: None,
            buy_max_cost: None,
            post_only: None,
            reduce_only: None,
            sell_position_floor: None,
            self_trade_prevention_type: None,
            order_group_id: None,
            cancel_order_on_pause: None,
            subaccount: None,
        }
    }

    #[must_use]
    pub fn market(ticker: impl Into<String>, side: Side, action: Action, count: i64) -> Self {
        Self {
            ticker: ticker.into(),
            client_order_id: None,
            side,
            action,
            count: Some(count),
            count_fp: Some(count * 100),
            yes_price: None,
            no_price: None,
            yes_price_dollars: None,
            no_price_dollars: None,
            expiration_ts: None,
            time_in_force: None,
            buy_max_cost: None,
            post_only: None,
            reduce_only: None,
            sell_position_floor: None,
            self_trade_prevention_type: None,
            order_group_id: None,
            cancel_order_on_pause: None,
            subaccount: None,
        }
    }

    #[must_use]
    pub fn with_client_order_id(mut self, id: impl Into<String>) -> Self {
        self.client_order_id = Some(id.into());
        self
    }

    #[must_use]
    pub fn with_order_group(mut self, group_id: impl Into<String>) -> Self {
        self.order_group_id = Some(group_id.into());
        self
    }

    #[must_use]
    pub fn with_time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = Some(tif);
        self
    }

    #[must_use]
    pub fn with_expiration_ts(mut self, ts: i64) -> Self {
        self.expiration_ts = Some(ts);
        self
    }

    #[must_use]
    pub fn with_subaccount(mut self, subaccount: i32) -> Self {
        self.subaccount = Some(subaccount);
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Order {
    pub order_id: String,
    pub user_id: String,
    pub client_order_id: String,
    pub ticker: String,
    pub side: Side,
    pub action: Action,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub status: OrderStatus,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub yes_price_dollars: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub no_price_dollars: i64,
    #[serde(deserialize_with = "deserialize_count")]
    pub fill_count_fp: i64,
    #[serde(deserialize_with = "deserialize_count")]
    pub remaining_count_fp: i64,
    #[serde(deserialize_with = "deserialize_count")]
    pub initial_count_fp: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub taker_fill_cost_dollars: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub maker_fill_cost_dollars: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub taker_fees_dollars: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub maker_fees_dollars: i64,
    #[serde(default)]
    pub expiration_time: Option<String>,
    #[serde(default)]
    pub created_time: Option<String>,
    #[serde(default)]
    pub last_update_time: Option<String>,
    #[serde(default)]
    pub self_trade_prevention_type: Option<SelfTradePrevention>,
    #[serde(default)]
    pub order_group_id: Option<String>,
    #[serde(default)]
    pub cancel_order_on_pause: Option<bool>,
    #[serde(default)]
    pub subaccount_number: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrderResponse {
    pub order: Order,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CancelOrderResponse {
    pub order: Order,
    #[serde(deserialize_with = "deserialize_count")]
    pub reduced_by_fp: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AmendOrderRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
    pub ticker: String,
    pub side: Side,
    pub action: Action,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price: Option<i64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_optional_dollars"
    )]
    pub yes_price_dollars: Option<i64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_optional_dollars"
    )]
    pub no_price_dollars: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_optional_count"
    )]
    pub count_fp: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AmendOrderResponse {
    pub old_order: Order,
    pub order: Order,
}

#[derive(Debug, Clone, Serialize)]
pub struct DecreaseOrderRequest {
    pub reduce_by: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DecreaseOrderResponse {
    pub order: Order,
    #[serde(default, deserialize_with = "deserialize_optional_count")]
    pub reduced_by_fp: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetOrdersResponse {
    pub orders: Vec<Order>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetOrderResponse {
    pub order: Order,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchCreateOrdersRequest {
    pub orders: Vec<CreateOrderRequest>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchOrderResult {
    #[serde(default)]
    pub client_order_id: Option<String>,
    #[serde(default)]
    pub order: Option<Order>,
    #[serde(default)]
    pub error: Option<BatchOrderError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchOrderError {
    #[serde(default)]
    pub code: Option<String>,
    pub message: String,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub service: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchCreateOrdersResponse {
    pub orders: Vec<BatchOrderResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchCancelOrdersRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orders: Option<Vec<BatchCancelOrdersRequestOrder>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchCancelOrdersRequestOrder {
    pub order_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchCancelResult {
    pub order_id: String,
    #[serde(default)]
    pub order: Option<Order>,
    #[serde(deserialize_with = "deserialize_count")]
    pub reduced_by_fp: i64,
    #[serde(default)]
    pub error: Option<BatchOrderError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchCancelOrdersResponse {
    pub orders: Vec<BatchCancelResult>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueuePosition {
    pub order_id: String,
    pub market_ticker: String,
    #[serde(deserialize_with = "deserialize_count")]
    pub queue_position_fp: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetOrderQueuePositionsResponse {
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
        let order = CreateOrderRequest::limit("KXBTC-25JAN", Side::Yes, Action::Buy, 10, 5_500);
        assert_eq!(order.ticker, "KXBTC-25JAN");
        assert_eq!(order.side, Side::Yes);
        assert_eq!(order.action, Action::Buy);
        assert_eq!(order.count, Some(10));
        assert_eq!(order.count_fp, Some(1_000));
        assert_eq!(order.yes_price_dollars, Some(5_500));
    }

    #[test]
    fn test_create_market_order() {
        let order = CreateOrderRequest::market("KXBTC-25JAN", Side::No, Action::Sell, 5);
        assert_eq!(order.count_fp, Some(500));
        assert_eq!(order.yes_price_dollars, None);
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
        let order = CreateOrderRequest::limit("TEST", Side::Yes, Action::Buy, 10, 5_000)
            .with_client_order_id("my-order-123")
            .with_time_in_force(TimeInForce::GoodTillCanceled)
            .with_subaccount(1);

        assert_eq!(order.client_order_id, Some("my-order-123".to_string()));
        assert_eq!(order.time_in_force, Some(TimeInForce::GoodTillCanceled));
        assert_eq!(order.subaccount, Some(1));
    }
}
