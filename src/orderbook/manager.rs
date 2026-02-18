//! Orderbook manager for handling multiple markets with WebSocket integration.
//!
//! This module provides [`OrderbookManager`], a thread-safe container for managing
//! multiple orderbooks that can be updated from WebSocket messages.
//!
//! # Design
//!
//! The manager uses `parking_lot::RwLock` for each orderbook, allowing concurrent
//! reads while ensuring exclusive write access during updates.
//!
//! # Sequence Tracking
//!
//! Each orderbook tracks its sequence number to detect gaps in WebSocket messages.
//! When a gap is detected, the orderbook is marked as stale and should be
//! re-synchronized via a snapshot request.

use std::collections::HashMap;

use parking_lot::RwLock;

use crate::error::Error;
use crate::types::messages::{OrderbookDeltaMsg, OrderbookSnapshotMsg, WsMessage};

use super::Orderbook;

/// State of an orderbook
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderbookState {
    /// Orderbook is synchronized and receiving updates
    Synchronized,
    /// Orderbook has detected a sequence gap and needs resync
    NeedsResync,
    /// Orderbook has not received initial snapshot
    WaitingForSnapshot,
}

/// Entry in the orderbook manager
#[derive(Debug)]
struct OrderbookEntry {
    book: Orderbook,
    state: OrderbookState,
    subscription_id: Option<u64>,
}

/// Manager for multiple orderbooks with WebSocket integration.
///
/// This struct provides thread-safe access to multiple orderbooks and handles
/// WebSocket message processing including:
///
/// - Applying snapshots and deltas
/// - Sequence gap detection
/// - State tracking per orderbook
///
/// # Thread Safety
///
/// The manager is safe to share across threads via `Arc<OrderbookManager>`.
/// Individual orderbooks are protected by `RwLock` for concurrent read access.
///
/// # Example
///
/// ```rust,no_run
/// use kalshi_rs::orderbook::OrderbookManager;
/// use std::sync::Arc;
///
/// # async fn example() {
/// let manager = Arc::new(OrderbookManager::new());
///
/// // Add markets to track
/// manager.add_market("KXBTC-25JAN");
/// manager.add_market("KXBTC-26JAN");
///
/// // In your WebSocket loop:
/// // manager.process_message(&ws_message);
///
/// // Read orderbook state
/// if let Some(book) = manager.get_orderbook("KXBTC-25JAN") {
///     println!("Best bid: {:?}", book.best_bid());
/// }
/// # }
/// ```
#[derive(Debug, Default)]
pub struct OrderbookManager {
    /// Orderbooks by market ticker
    books: RwLock<HashMap<String, RwLock<OrderbookEntry>>>,
}

impl OrderbookManager {
    /// Create a new orderbook manager
    pub fn new() -> Self {
        Self {
            books: RwLock::new(HashMap::new()),
        }
    }

    /// Add a market to track
    ///
    /// Creates an empty orderbook in `WaitingForSnapshot` state.
    pub fn add_market(&self, market_ticker: impl Into<String>) {
        let ticker = market_ticker.into();
        let mut books = self.books.write();
        books.entry(ticker.clone()).or_insert_with(|| {
            RwLock::new(OrderbookEntry {
                book: Orderbook::new(&ticker),
                state: OrderbookState::WaitingForSnapshot,
                subscription_id: None,
            })
        });
    }

    /// Remove a market from tracking
    pub fn remove_market(&self, market_ticker: &str) {
        let mut books = self.books.write();
        books.remove(market_ticker);
    }

    /// Set the subscription ID for a market
    ///
    /// Used to track which subscription is providing updates for this market.
    pub fn set_subscription_id(&self, market_ticker: &str, sid: u64) {
        let books = self.books.read();
        if let Some(entry) = books.get(market_ticker) {
            entry.write().subscription_id = Some(sid);
        }
    }

    /// Get the state of an orderbook
    pub fn get_state(&self, market_ticker: &str) -> Option<OrderbookState> {
        let books = self.books.read();
        books.get(market_ticker).map(|e| e.read().state)
    }

    /// Get all markets that need resync
    pub fn markets_needing_resync(&self) -> Vec<String> {
        let books = self.books.read();
        books
            .iter()
            .filter(|(_, entry)| {
                let e = entry.read();
                matches!(
                    e.state,
                    OrderbookState::NeedsResync | OrderbookState::WaitingForSnapshot
                )
            })
            .map(|(ticker, _)| ticker.clone())
            .collect()
    }

    /// Get a snapshot of an orderbook
    ///
    /// Returns a cloned copy of the orderbook for safe reading without holding locks.
    pub fn get_orderbook(&self, market_ticker: &str) -> Option<Orderbook> {
        let books = self.books.read();
        books.get(market_ticker).map(|e| e.read().book.clone())
    }

    /// Get best bid for a market
    pub fn best_bid(&self, market_ticker: &str) -> Option<(i64, i64)> {
        let books = self.books.read();
        books
            .get(market_ticker)
            .and_then(|e| e.read().book.best_bid())
    }

    /// Get best ask for a market
    pub fn best_ask(&self, market_ticker: &str) -> Option<(i64, i64)> {
        let books = self.books.read();
        books
            .get(market_ticker)
            .and_then(|e| e.read().book.best_ask())
    }

    /// Get mid price for a market
    pub fn mid_price(&self, market_ticker: &str) -> Option<f64> {
        let books = self.books.read();
        books
            .get(market_ticker)
            .and_then(|e| e.read().book.mid_price())
    }

    /// Get spread for a market
    pub fn spread(&self, market_ticker: &str) -> Option<i64> {
        let books = self.books.read();
        books
            .get(market_ticker)
            .and_then(|e| e.read().book.spread())
    }

    /// Process a WebSocket message
    ///
    /// Automatically routes snapshots and deltas to the appropriate orderbook.
    /// Returns the market ticker if an orderbook was updated.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(ticker))` - An orderbook was updated
    /// - `Ok(None)` - Message was not an orderbook message
    /// - `Err(_)` - A sequence gap was detected
    pub fn process_message(&self, message: &WsMessage) -> Result<Option<String>, Error> {
        match message {
            WsMessage::OrderbookSnapshot(snapshot) => {
                self.apply_snapshot(snapshot);
                Ok(Some(snapshot.msg.market_ticker.clone()))
            }
            WsMessage::OrderbookDelta(delta) => {
                let ticker = delta.msg.market_ticker.clone();
                if self.apply_delta(delta)? {
                    Ok(Some(ticker))
                } else {
                    // Market not tracked
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    /// Apply an orderbook snapshot
    fn apply_snapshot(&self, snapshot: &OrderbookSnapshotMsg) {
        let ticker = &snapshot.msg.market_ticker;
        let books = self.books.read();

        // Auto-add market if not tracked
        if let Some(entry) = books.get(ticker) {
            let mut e = entry.write();
            e.book.apply_snapshot(&snapshot.msg, snapshot.seq);
            e.state = OrderbookState::Synchronized;
            e.subscription_id = Some(snapshot.sid);
        } else {
            drop(books);
            self.add_market(ticker);
            let books = self.books.read();
            if let Some(entry) = books.get(ticker) {
                let mut e = entry.write();
                e.book.apply_snapshot(&snapshot.msg, snapshot.seq);
                e.state = OrderbookState::Synchronized;
                e.subscription_id = Some(snapshot.sid);
            }
        }
    }

    /// Apply an orderbook delta
    ///
    /// Returns `Ok(true)` if delta was applied, `Ok(false)` if market not tracked,
    /// `Err` if there was a sequence gap.
    fn apply_delta(&self, delta: &OrderbookDeltaMsg) -> Result<bool, Error> {
        let ticker = &delta.msg.market_ticker;
        let books = self.books.read();

        if let Some(entry) = books.get(ticker) {
            let mut e = entry.write();

            // Skip deltas if we're not synchronized
            if e.state != OrderbookState::Synchronized {
                return Ok(true);
            }

            // Apply delta and check sequence
            if e.book.apply_delta_msg(&delta.msg, delta.seq) {
                Ok(true)
            } else {
                // Sequence gap detected
                let expected = e.book.sequence() + 1;
                e.state = OrderbookState::NeedsResync;
                Err(Error::SequenceGap {
                    expected,
                    got: delta.seq,
                })
            }
        } else {
            Ok(false)
        }
    }

    /// Mark an orderbook as needing resync
    pub fn mark_needs_resync(&self, market_ticker: &str) {
        let books = self.books.read();
        if let Some(entry) = books.get(market_ticker) {
            entry.write().state = OrderbookState::NeedsResync;
        }
    }

    /// Clear all orderbooks
    pub fn clear(&self) {
        let mut books = self.books.write();
        books.clear();
    }

    /// Get number of tracked markets
    pub fn len(&self) -> usize {
        self.books.read().len()
    }

    /// Check if manager has no markets
    pub fn is_empty(&self) -> bool {
        self.books.read().is_empty()
    }

    /// Get all tracked market tickers
    pub fn market_tickers(&self) -> Vec<String> {
        self.books.read().keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::messages::{OrderbookDeltaData, OrderbookSnapshotData};
    use crate::types::order::Side;

    #[test]
    fn test_add_market() {
        let manager = OrderbookManager::new();
        manager.add_market("TEST");

        assert_eq!(manager.len(), 1);
        assert_eq!(
            manager.get_state("TEST"),
            Some(OrderbookState::WaitingForSnapshot)
        );
    }

    #[test]
    fn test_apply_snapshot() {
        let manager = OrderbookManager::new();
        manager.add_market("TEST");

        let snapshot = OrderbookSnapshotMsg {
            sid: 1,
            seq: 1,
            msg: OrderbookSnapshotData {
                market_ticker: "TEST".to_string(),
                yes: vec![[50, 100], [45, 200]],
                no: vec![[55, 150]],
            },
        };

        manager.apply_snapshot(&snapshot);

        assert_eq!(
            manager.get_state("TEST"),
            Some(OrderbookState::Synchronized)
        );
        assert_eq!(manager.best_bid("TEST"), Some((50, 100)));
        assert_eq!(manager.best_ask("TEST"), Some((45, 150))); // 100 - 55 = 45
    }

    #[test]
    fn test_apply_delta() {
        let manager = OrderbookManager::new();

        // First apply a snapshot
        let snapshot = OrderbookSnapshotMsg {
            sid: 1,
            seq: 1,
            msg: OrderbookSnapshotData {
                market_ticker: "TEST".to_string(),
                yes: vec![[50, 100]],
                no: vec![],
            },
        };
        manager.apply_snapshot(&snapshot);

        // Then apply a delta
        let delta = OrderbookDeltaMsg {
            sid: 1,
            seq: 2,
            msg: OrderbookDeltaData {
                market_ticker: "TEST".to_string(),
                price: 50,
                delta: 50,
                side: Side::Yes,
                ts: None,
            },
        };

        let result = manager.apply_delta(&delta);
        assert!(result.is_ok());
        assert_eq!(manager.best_bid("TEST"), Some((50, 150)));
    }

    #[test]
    fn test_sequence_gap() {
        let manager = OrderbookManager::new();

        // Apply snapshot at seq 1
        let snapshot = OrderbookSnapshotMsg {
            sid: 1,
            seq: 1,
            msg: OrderbookSnapshotData {
                market_ticker: "TEST".to_string(),
                yes: vec![[50, 100]],
                no: vec![],
            },
        };
        manager.apply_snapshot(&snapshot);

        // Skip seq 2, apply seq 3 - should detect gap
        let delta = OrderbookDeltaMsg {
            sid: 1,
            seq: 3, // Gap!
            msg: OrderbookDeltaData {
                market_ticker: "TEST".to_string(),
                price: 50,
                delta: 50,
                side: Side::Yes,
                ts: None,
            },
        };

        let result = manager.apply_delta(&delta);
        assert!(result.is_err());
        assert_eq!(manager.get_state("TEST"), Some(OrderbookState::NeedsResync));
    }

    #[test]
    fn test_markets_needing_resync() {
        let manager = OrderbookManager::new();
        manager.add_market("TEST1");
        manager.add_market("TEST2");

        // Both start as WaitingForSnapshot
        let needing_resync = manager.markets_needing_resync();
        assert_eq!(needing_resync.len(), 2);

        // Sync one
        let snapshot = OrderbookSnapshotMsg {
            sid: 1,
            seq: 1,
            msg: OrderbookSnapshotData {
                market_ticker: "TEST1".to_string(),
                yes: vec![],
                no: vec![],
            },
        };
        manager.apply_snapshot(&snapshot);

        let needing_resync = manager.markets_needing_resync();
        assert_eq!(needing_resync.len(), 1);
        assert_eq!(needing_resync[0], "TEST2");
    }
}
