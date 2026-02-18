//! Core orderbook data structure.
//!
//! This implementation uses `BTreeMap` for sorted price levels, providing:
//!
//! - O(log n) insertion, deletion, and lookup
//! - O(1) access to best bid/ask (via `first_key_value` / `last_key_value`)
//! - Ordered iteration for depth-of-book queries

use std::collections::BTreeMap;

use crate::types::messages::{OrderbookDeltaData, OrderbookSnapshotData};
use crate::types::order::Side;
use crate::types::{Price, Quantity};

/// HFT-optimized orderbook for a single Kalshi market.
///
/// # Design Decisions
///
/// 1. **Integer prices**: Prices are stored as `u8` (1-99 cents), avoiding
///    floating-point arithmetic and enabling exact comparisons.
///
/// 2. **BTreeMap**: Provides sorted price levels with O(log n) operations.
///    Best bid/ask are O(1) via `last_key_value()` / `first_key_value()`.
///
/// 3. **Sequence tracking**: The `sequence` field tracks WebSocket message
///    order to detect gaps and trigger re-synchronization.
///
/// 4. **No allocations on update**: Delta updates modify existing maps
///    without allocating new memory in the common case.
///
/// # Thread Safety
///
/// This struct is `Send + Sync` but not internally synchronized. For
/// concurrent access, wrap in `parking_lot::RwLock` or `Arc<Mutex>`.
#[derive(Debug, Clone)]
pub struct Orderbook {
    /// Market ticker
    market_ticker: String,

    /// Yes side bid levels: price -> quantity
    /// Sorted ascending by price (best bid = highest = last)
    yes_bids: BTreeMap<Price, Quantity>,

    /// Yes side ask levels: price -> quantity
    /// Sorted ascending by price (best ask = lowest = first)
    yes_asks: BTreeMap<Price, Quantity>,

    /// Last sequence number received (for gap detection)
    sequence: u64,
}

impl Orderbook {
    /// Create a new empty orderbook for the given market
    #[must_use]
    pub fn new(market_ticker: impl Into<String>) -> Self {
        Self {
            market_ticker: market_ticker.into(),
            yes_bids: BTreeMap::new(),
            yes_asks: BTreeMap::new(),
            sequence: 0,
        }
    }

    /// Get the market ticker
    #[must_use]
    pub fn market_ticker(&self) -> &str {
        &self.market_ticker
    }

    /// Get the current sequence number
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Apply a snapshot from WebSocket
    ///
    /// This replaces the entire orderbook state.
    pub fn apply_snapshot(&mut self, snapshot: &OrderbookSnapshotData, sequence: u64) {
        self.yes_bids.clear();
        self.yes_asks.clear();

        // Yes side in snapshot contains bids
        for level in &snapshot.yes {
            let price = level[0] as Price;
            let quantity = level[1] as Quantity;
            if quantity > 0 {
                self.yes_bids.insert(price, quantity);
            }
        }

        // No side in snapshot - convert to yes asks
        // No bid at price P = Yes ask at price (100 - P)
        for level in &snapshot.no {
            let no_price = level[0] as Price;
            let quantity = level[1] as Quantity;
            if quantity > 0 {
                let yes_price = 100 - no_price;
                self.yes_asks.insert(yes_price, quantity);
            }
        }

        self.sequence = sequence;
    }

    /// Apply a delta update from WebSocket
    ///
    /// Returns `true` if the sequence was valid, `false` if there was a gap.
    pub fn apply_delta_msg(&mut self, delta: &OrderbookDeltaData, sequence: u64) -> bool {
        // Check for sequence gap
        if sequence != self.sequence + 1 && self.sequence != 0 {
            // Sequence gap detected - caller should request re-sync
            return false;
        }

        self.sequence = sequence;

        // Determine which side of the book to update
        let (book, price) = match delta.side {
            Side::Yes => (&mut self.yes_bids, delta.price),
            Side::No => {
                // No delta affects yes asks at inverted price
                let yes_price = 100 - delta.price;
                (&mut self.yes_asks, yes_price)
            }
        };

        // Apply the delta
        if delta.delta == 0 {
            // No change
        } else if delta.delta < 0 {
            // Quantity decreased
            let decrease = (-delta.delta) as Quantity;
            if let Some(current) = book.get_mut(&price) {
                if *current <= decrease {
                    book.remove(&price);
                } else {
                    *current -= decrease;
                }
            }
        } else {
            // Quantity increased
            let increase = delta.delta as Quantity;
            *book.entry(price).or_insert(0) += increase;
        }

        true
    }

    /// Apply a delta directly (for manual updates)
    ///
    /// # Arguments
    ///
    /// * `price` - Price level in cents
    /// * `delta` - Change in quantity (positive = add, negative = remove)
    /// * `side` - Which side of the book
    pub fn apply_delta(&mut self, price: Price, delta: i64, side: Side) {
        let book = match side {
            Side::Yes => &mut self.yes_bids,
            Side::No => &mut self.yes_asks,
        };

        if delta == 0 {
            return;
        }

        if delta < 0 {
            let decrease = (-delta) as Quantity;
            if let Some(current) = book.get_mut(&price) {
                if *current <= decrease {
                    book.remove(&price);
                } else {
                    *current -= decrease;
                }
            }
        } else {
            let increase = delta as Quantity;
            *book.entry(price).or_insert(0) += increase;
        }
    }

    /// Set a price level directly
    ///
    /// Use this for snapshot reconstruction. Sets quantity to 0 removes the level.
    pub fn set_level(&mut self, price: Price, quantity: Quantity, side: Side) {
        let book = match side {
            Side::Yes => &mut self.yes_bids,
            Side::No => &mut self.yes_asks,
        };

        if quantity == 0 {
            book.remove(&price);
        } else {
            book.insert(price, quantity);
        }
    }

    /// Get the best bid (highest yes bid)
    ///
    /// Returns `(price, quantity)` or `None` if no bids.
    #[must_use]
    pub fn best_bid(&self) -> Option<(Price, Quantity)> {
        self.yes_bids.last_key_value().map(|(&p, &q)| (p, q))
    }

    /// Get the best ask (lowest yes ask)
    ///
    /// Returns `(price, quantity)` or `None` if no asks.
    #[must_use]
    pub fn best_ask(&self) -> Option<(Price, Quantity)> {
        self.yes_asks.first_key_value().map(|(&p, &q)| (p, q))
    }

    /// Get the mid price
    ///
    /// Returns the average of best bid and best ask, or `None` if either is missing.
    #[must_use]
    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => Some((bid as f64 + ask as f64) / 2.0),
            _ => None,
        }
    }

    /// Get the spread in cents
    #[must_use]
    pub fn spread(&self) -> Option<Price> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => Some(ask.saturating_sub(bid)),
            _ => None,
        }
    }

    /// Check if the book is crossed (best bid >= best ask)
    ///
    /// This shouldn't happen in a healthy market but is useful for validation.
    #[must_use]
    pub fn is_crossed(&self) -> bool {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => bid >= ask,
            _ => false,
        }
    }

    /// Get all bid levels, sorted by price descending (best first)
    pub fn bids(&self) -> impl Iterator<Item = (Price, Quantity)> + '_ {
        self.yes_bids.iter().rev().map(|(&p, &q)| (p, q))
    }

    /// Get all ask levels, sorted by price ascending (best first)
    pub fn asks(&self) -> impl Iterator<Item = (Price, Quantity)> + '_ {
        self.yes_asks.iter().map(|(&p, &q)| (p, q))
    }

    /// Get the top N bid levels
    #[must_use]
    pub fn top_bids(&self, n: usize) -> Vec<(Price, Quantity)> {
        self.bids().take(n).collect()
    }

    /// Get the top N ask levels
    #[must_use]
    pub fn top_asks(&self, n: usize) -> Vec<(Price, Quantity)> {
        self.asks().take(n).collect()
    }

    /// Get total bid quantity
    #[must_use]
    pub fn total_bid_quantity(&self) -> Quantity {
        self.yes_bids.values().sum()
    }

    /// Get total ask quantity
    #[must_use]
    pub fn total_ask_quantity(&self) -> Quantity {
        self.yes_asks.values().sum()
    }

    /// Clear the orderbook
    pub fn clear(&mut self) {
        self.yes_bids.clear();
        self.yes_asks.clear();
        self.sequence = 0;
    }

    /// Check if the orderbook is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.yes_bids.is_empty() && self.yes_asks.is_empty()
    }

    /// Get the number of price levels
    #[must_use]
    pub fn num_levels(&self) -> (usize, usize) {
        (self.yes_bids.len(), self.yes_asks.len())
    }
}

impl Default for Orderbook {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_orderbook() {
        let book = Orderbook::new("KXBTC-25JAN");
        assert_eq!(book.market_ticker(), "KXBTC-25JAN");
        assert!(book.is_empty());
        assert_eq!(book.sequence(), 0);
    }

    #[test]
    fn test_set_level() {
        let mut book = Orderbook::new("TEST");

        book.set_level(50, 100, Side::Yes);
        book.set_level(45, 50, Side::Yes);
        book.set_level(55, 75, Side::No);

        assert_eq!(book.best_bid(), Some((50, 100)));
        assert_eq!(book.best_ask(), Some((55, 75)));
    }

    #[test]
    fn test_apply_delta() {
        let mut book = Orderbook::new("TEST");

        // Add quantity
        book.apply_delta(50, 100, Side::Yes);
        assert_eq!(book.best_bid(), Some((50, 100)));

        // Add more
        book.apply_delta(50, 50, Side::Yes);
        assert_eq!(book.best_bid(), Some((50, 150)));

        // Remove some
        book.apply_delta(50, -50, Side::Yes);
        assert_eq!(book.best_bid(), Some((50, 100)));

        // Remove all
        book.apply_delta(50, -100, Side::Yes);
        assert_eq!(book.best_bid(), None);
    }

    #[test]
    fn test_mid_price_and_spread() {
        let mut book = Orderbook::new("TEST");

        book.set_level(45, 100, Side::Yes); // Best bid
        book.set_level(55, 100, Side::No); // Best ask

        assert_eq!(book.mid_price(), Some(50.0));
        assert_eq!(book.spread(), Some(10));
    }

    #[test]
    fn test_top_levels() {
        let mut book = Orderbook::new("TEST");

        book.set_level(45, 100, Side::Yes);
        book.set_level(44, 200, Side::Yes);
        book.set_level(43, 300, Side::Yes);

        let top = book.top_bids(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0], (45, 100)); // Best bid first
        assert_eq!(top[1], (44, 200));
    }

    #[test]
    fn test_crossed_book() {
        let mut book = Orderbook::new("TEST");

        book.set_level(55, 100, Side::Yes); // Bid at 55
        book.set_level(50, 100, Side::No); // Ask at 50

        assert!(book.is_crossed());
    }

    #[test]
    fn test_clear() {
        let mut book = Orderbook::new("TEST");
        book.set_level(50, 100, Side::Yes);
        book.set_level(55, 100, Side::No);

        assert!(!book.is_empty());

        book.clear();

        assert!(book.is_empty());
        assert_eq!(book.sequence(), 0);
    }
}
