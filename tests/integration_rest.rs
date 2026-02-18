//! Integration tests for the REST API.
//!
//! These tests run against Kalshi's demo API.
//!
//! # Setup
//!
//! 1. Create a demo account at https://demo.kalshi.com
//! 2. Generate API credentials in the dashboard
//! 3. Set environment variables:
//!    - KALSHI_API_KEY: Your API key ID
//!    - KALSHI_PRIVATE_KEY_PATH: Path to your private key PEM file
//!
//! # Running
//!
//! ```bash
//! KALSHI_API_KEY=your-key KALSHI_PRIVATE_KEY_PATH=./private_key.pem cargo test --test integration_rest
//! ```

use kalshi_trading::config::Environment;
use kalshi_trading::types::{Action, CreateOrderRequest, Side};
use kalshi_trading::{Config, KalshiClient};

/// Helper to create a client from environment variables
fn create_client() -> Option<KalshiClient> {
    let api_key = std::env::var("KALSHI_API_KEY").ok()?;
    let key_path = std::env::var("KALSHI_PRIVATE_KEY_PATH").ok()?;
    let private_key = std::fs::read_to_string(&key_path).ok()?;

    let config = Config::new(&api_key, &private_key).with_environment(Environment::Demo);

    KalshiClient::new(config).ok()
}

/// Skip test if credentials not available
macro_rules! require_client {
    () => {
        match create_client() {
            Some(c) => c,
            None => {
                eprintln!("Skipping test: KALSHI_API_KEY and KALSHI_PRIVATE_KEY_PATH not set");
                return;
            }
        }
    };
}

#[tokio::test]
async fn test_get_exchange_status() {
    let client = require_client!();

    let status = client.rest().get_exchange_status().await;
    assert!(status.is_ok(), "Failed to get exchange status: {:?}", status);

    let status = status.unwrap();
    println!("Exchange status: {:?}", status);
    // Just verify we got a response with boolean fields
    let _ = status.exchange_active;
    let _ = status.trading_active;
}

#[tokio::test]
async fn test_get_exchange_schedule() {
    let client = require_client!();

    let schedule = client.rest().get_exchange_schedule().await;
    assert!(
        schedule.is_ok(),
        "Failed to get exchange schedule: {:?}",
        schedule
    );

    let schedule = schedule.unwrap();
    println!("Exchange schedule: {:?}", schedule.schedule);
}

#[tokio::test]
async fn test_get_markets() {
    let client = require_client!();

    // Get all markets
    let markets = client.rest().get_markets(None, None, None).await;
    assert!(markets.is_ok(), "Failed to get markets: {:?}", markets);

    let markets = markets.unwrap();
    println!("Found {} markets", markets.markets.len());

    // Get open markets only
    let open_markets = client.rest().get_markets(Some("open"), None, None).await;
    assert!(
        open_markets.is_ok(),
        "Failed to get open markets: {:?}",
        open_markets
    );

    let open_markets = open_markets.unwrap();
    println!("Found {} open markets", open_markets.markets.len());
}

#[tokio::test]
async fn test_get_single_market() {
    let client = require_client!();

    // First get a market ticker
    let markets = client.rest().get_markets(Some("open"), None, None).await;
    if markets.is_err() || markets.as_ref().unwrap().markets.is_empty() {
        eprintln!("No open markets available for testing");
        return;
    }

    let ticker = &markets.unwrap().markets[0].ticker;
    println!("Testing with market: {}", ticker);

    let market = client.rest().get_market(ticker).await;
    assert!(market.is_ok(), "Failed to get market: {:?}", market);

    let market = market.unwrap();
    println!("Market: {} - {}", market.market.ticker, market.market.title);
}

#[tokio::test]
async fn test_get_orderbook() {
    let client = require_client!();

    // First get a market ticker
    let markets = client.rest().get_markets(Some("open"), None, None).await;
    if markets.is_err() || markets.as_ref().unwrap().markets.is_empty() {
        eprintln!("No open markets available for testing");
        return;
    }

    let ticker = &markets.unwrap().markets[0].ticker;
    println!("Getting orderbook for: {}", ticker);

    let orderbook = client.rest().get_orderbook(ticker).await;
    assert!(orderbook.is_ok(), "Failed to get orderbook: {:?}", orderbook);

    let orderbook = orderbook.unwrap();
    println!(
        "Orderbook has {} yes levels, {} no levels",
        orderbook.orderbook.yes.len(),
        orderbook.orderbook.no.len()
    );
}

#[tokio::test]
async fn test_get_events() {
    let client = require_client!();

    let events = client.rest().get_events(None, None, None).await;
    assert!(events.is_ok(), "Failed to get events: {:?}", events);

    let events = events.unwrap();
    println!("Found {} events", events.events.len());
}

#[tokio::test]
async fn test_get_balance() {
    let client = require_client!();

    let balance = client.rest().get_balance().await;
    assert!(balance.is_ok(), "Failed to get balance: {:?}", balance);

    let balance = balance.unwrap();
    println!(
        "Balance: ${:.2}",
        balance.balance as f64 / 10000.0
    );
}

#[tokio::test]
async fn test_get_positions() {
    let client = require_client!();

    let positions = client.rest().get_positions(None, None, None, None).await;
    assert!(positions.is_ok(), "Failed to get positions: {:?}", positions);

    let positions = positions.unwrap();
    println!("Found {} positions", positions.event_positions.len());
}

#[tokio::test]
async fn test_get_orders() {
    let client = require_client!();

    let orders = client.rest().get_orders(None, None, None).await;
    assert!(orders.is_ok(), "Failed to get orders: {:?}", orders);

    let orders = orders.unwrap();
    println!("Found {} orders", orders.orders.len());
}

#[tokio::test]
async fn test_get_fills() {
    let client = require_client!();

    let fills = client.rest().get_fills(None, None, None, None).await;
    assert!(fills.is_ok(), "Failed to get fills: {:?}", fills);

    let fills = fills.unwrap();
    println!("Found {} fills", fills.fills.len());
}

#[tokio::test]
async fn test_order_lifecycle() {
    let client = require_client!();

    // Find an open market
    let markets = client.rest().get_markets(Some("open"), None, None).await;
    if markets.is_err() || markets.as_ref().unwrap().markets.is_empty() {
        eprintln!("No open markets available for testing");
        return;
    }

    let market = &markets.unwrap().markets[0];
    let ticker = &market.ticker;
    println!("Testing order lifecycle on: {}", ticker);

    // Create a limit order at a low price (unlikely to fill)
    let order = CreateOrderRequest::limit(
        ticker.clone(),
        Side::Yes,
        Action::Buy,
        1,
        100, // $0.01 - very unlikely to fill
    );

    let create_result = client.rest().create_order(&order).await;
    assert!(
        create_result.is_ok(),
        "Failed to create order: {:?}",
        create_result
    );

    let created_order = create_result.unwrap();
    let order_id = &created_order.order.order_id;
    println!("Created order: {}", order_id);

    // Get the order
    let get_result = client.rest().get_order(order_id).await;
    assert!(get_result.is_ok(), "Failed to get order: {:?}", get_result);

    let order_details = get_result.unwrap();
    println!("Order status: {:?}", order_details.order.status);

    // Cancel the order
    let cancel_result = client.rest().cancel_order(order_id).await;
    assert!(
        cancel_result.is_ok(),
        "Failed to cancel order: {:?}",
        cancel_result
    );

    println!("Order cancelled successfully");

    // Verify it's cancelled
    let get_result = client.rest().get_order(order_id).await;
    if let Ok(order) = get_result {
        println!("Final order status: {:?}", order.order.status);
    }
}

#[tokio::test]
async fn test_get_trades() {
    let client = require_client!();

    // Get recent trades for any market
    let trades = client.rest().get_trades(None, None, None).await;
    assert!(trades.is_ok(), "Failed to get trades: {:?}", trades);

    let trades = trades.unwrap();
    println!("Found {} recent trades", trades.trades.len());
}
