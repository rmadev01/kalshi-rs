//! High-performance orderbook implementation.
//!
//! This module provides an HFT-grade orderbook data structure optimized for:
//!
//! - Fast updates (O(log n) for price level operations)
//! - Cache efficiency (integer prices, minimal allocations)
//! - Sequence tracking (detect missed WebSocket messages)
//!
//! # Components
//!
//! - [`Orderbook`] - Single market orderbook with delta/snapshot support
//! - [`OrderbookManager`] - Thread-safe container for multiple orderbooks
//! - [`OrderbookState`] - State enum for tracking sync status
//!
//! # Example
//!
//! ```rust
//! use kalshi_trading::orderbook::Orderbook;
//! use kalshi_trading::types::order::Side;
//!
//! let mut book = Orderbook::new("KXBTC-25JAN");
//!
//! // Apply a delta
//! book.apply_delta(55, 100, Side::Yes);
//! book.apply_delta(45, 50, Side::Yes);
//!
//! // Get best bid
//! if let Some((price, qty)) = book.best_bid() {
//!     println!("Best bid: {} @ {}", qty, price);
//! }
//! ```

pub mod book;
pub mod manager;

pub use book::Orderbook;
pub use manager::{OrderbookManager, OrderbookState};
