//! API types for Kalshi requests and responses.
//!
//! This module contains Rust types that correspond to the Kalshi API's
//! JSON request and response bodies.
//!
//! - [`order`] - Order-related types (Side, Action, CreateOrderRequest, etc.)
//! - [`market`] - Market and event types
//! - [`messages`] - WebSocket message types

pub mod market;
pub mod messages;
pub mod order;

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

/// Price in centi-cents (100 centi-cents = 1 cent, 10000 centi-cents = $1)
///
/// Kalshi uses centi-cents for subpenny precision:
/// - 100 = $0.01 (1 cent, 1% implied probability)
/// - 9900 = $0.99 (99 cents, 99% implied probability)
/// - 5050 = $0.505 (50.5 cents, 50.5% implied probability)
///
/// Using `i64` for:
/// - Exact arithmetic (no floating point errors)
/// - Support for signed values (P&L can be negative)
/// - Compatibility with API responses
pub type Price = i64;

/// Quantity of contracts
///
/// Using `i64` for compatibility with API responses (positions can be negative for shorts).
pub type Quantity = i64;

/// Timestamp in milliseconds since Unix epoch
pub type TimestampMs = i64;
