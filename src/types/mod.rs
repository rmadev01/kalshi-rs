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

pub use market::{Event, Market, MarketStatus};
pub use messages::WsMessage;
pub use order::{Action, Order, OrderStatus, Side};

/// Price in cents (1-99 for Kalshi binary contracts)
///
/// Kalshi prices are always in cents, where:
/// - 1 = $0.01 (1% implied probability)
/// - 99 = $0.99 (99% implied probability)
///
/// Using `u8` instead of floating point for:
/// - Exact arithmetic (no floating point errors)
/// - Faster comparisons
/// - Cache efficiency
pub type Price = u8;

/// Quantity of contracts
///
/// Using `u32` as Kalshi has a 200,000 open order limit, which fits in u32.
pub type Quantity = u32;

/// Timestamp in milliseconds since Unix epoch
pub type TimestampMs = u64;
