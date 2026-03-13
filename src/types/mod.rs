//! API types for Kalshi requests and responses.
//!
//! This module contains Rust types that correspond to the Kalshi API's
//! JSON request and response bodies.
//!
//! - [`order`] - Order-related types (Side, Action, CreateOrderRequest, etc.)
//! - [`market`] - Market and event types
//! - [`messages`] - WebSocket message types

mod fixed_point;
pub mod market;
pub mod messages;
pub mod order;

pub(crate) use fixed_point::{
    deserialize_count, deserialize_dollars, deserialize_optional_count,
    deserialize_optional_dollars, serialize_optional_count, serialize_optional_dollars,
    DOLLAR_SCALE,
};
pub use fixed_point::{format_count, format_dollars, parse_count, parse_dollars};
pub use market::{
    Balance, Event, EventPosition, ExchangeSchedule, ExchangeStatus, Fill, GetBalanceResponse,
    GetEventResponse, GetEventsResponse, GetExchangeScheduleResponse, GetFillsResponse,
    GetMarketResponse, GetMarketsResponse, GetOrderbookResponse, GetPositionsResponse,
    GetSeriesListResponse, GetSeriesResponse, GetSettlementsResponse, GetTradesResponse, Market,
    MarketStatus, Orderbook, OrderbookLevel, Position, Series, Settlement, SettlementResult,
    SettlementSource, Trade,
};
pub use messages::WsMessage;
pub use order::{
    Action, AmendOrderRequest, AmendOrderResponse, BatchCancelOrdersRequest,
    BatchCancelOrdersResponse, BatchCancelResult, BatchCreateOrdersRequest,
    BatchCreateOrdersResponse, BatchOrderError, BatchOrderResult, CancelOrderResponse,
    CreateOrderRequest, CreateOrderResponse, DecreaseOrderRequest, DecreaseOrderResponse,
    GetOrderQueuePositionsResponse, GetOrderResponse, GetOrdersResponse, Order, OrderStatus,
    OrderType, QueuePosition, SelfTradePrevention, Side, TimeInForce,
};

/// Price in ten-thousandths of a dollar.
///
/// The current Kalshi v2 API commonly represents prices as fixed-point dollar
/// strings such as `"0.5600"`; this crate stores them as scaled integers where
/// `10_000 == $1.0000`.
pub type Price = i64;

/// Quantity of contracts scaled by 100.
///
/// Kalshi emits fixed-point count strings such as `"10.00"`; this crate stores
/// them as scaled integers where `100 == 1.00` contracts.
pub type Quantity = i64;

/// Unix timestamp in seconds.
pub type TimestampMs = i64;
