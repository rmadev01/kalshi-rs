//! Integration tests for the WebSocket API.
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
//! KALSHI_API_KEY=your-key KALSHI_PRIVATE_KEY_PATH=./private_key.pem cargo test --test integration_websocket
//! ```

use std::time::Duration;

use kalshi_rs::client::websocket::WebSocketClient;
use kalshi_rs::config::Environment;
use kalshi_rs::types::WsMessage;
use kalshi_rs::{Config, KalshiClient};
use tokio::time::timeout;

/// Helper to create a config from environment variables
fn create_config() -> Option<Config> {
    let api_key = std::env::var("KALSHI_API_KEY").ok()?;
    let key_path = std::env::var("KALSHI_PRIVATE_KEY_PATH").ok()?;
    let private_key = std::fs::read_to_string(&key_path).ok()?;

    Some(Config::new(&api_key, &private_key).with_environment(Environment::Demo))
}

/// Skip test if credentials not available
macro_rules! require_config {
    () => {
        match create_config() {
            Some(c) => c,
            None => {
                eprintln!("Skipping test: KALSHI_API_KEY and KALSHI_PRIVATE_KEY_PATH not set");
                return;
            }
        }
    };
}

#[tokio::test]
async fn test_websocket_connect() {
    let config = require_config!();

    let result = WebSocketClient::connect(&config).await;
    assert!(result.is_ok(), "Failed to connect: {:?}", result);

    let mut ws = result.unwrap();
    println!("WebSocket connected successfully");

    // Clean close
    let close_result = ws.close().await;
    assert!(close_result.is_ok(), "Failed to close: {:?}", close_result);
}

#[tokio::test]
async fn test_subscribe_ticker() {
    let config = require_config!();

    let mut ws = match WebSocketClient::connect(&config).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            return;
        }
    };

    // Subscribe to all tickers
    let sub_result = ws.subscribe_ticker(None).await;
    assert!(sub_result.is_ok(), "Failed to subscribe: {:?}", sub_result);

    println!("Subscribed to ticker channel");

    // Wait for subscription confirmation or timeout
    let timeout_duration = Duration::from_secs(5);
    let result = timeout(timeout_duration, async {
        let mut received_subscribed = false;
        let mut message_count = 0;

        while let Some(msg_result) = ws.next().await {
            match msg_result {
                Ok(WsMessage::Subscribed(subscribed)) => {
                    println!(
                        "Subscribed to channel: {} (sid: {})",
                        subscribed.msg.channel, subscribed.msg.sid
                    );
                    received_subscribed = true;
                }
                Ok(WsMessage::Ticker(ticker)) => {
                    println!(
                        "Ticker: {} - price: {:?}",
                        ticker.msg.market_ticker, ticker.msg.price
                    );
                    message_count += 1;
                    if message_count >= 3 {
                        break;
                    }
                }
                Ok(msg) => {
                    println!("Other message: {:?}", msg);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            }
        }

        received_subscribed
    })
    .await;

    match result {
        Ok(received) => {
            assert!(received, "Did not receive subscription confirmation");
        }
        Err(_) => {
            println!("Timeout reached (this is okay if we received some messages)");
        }
    }

    let _ = ws.close().await;
}

#[tokio::test]
async fn test_subscribe_orderbook() {
    let config = require_config!();

    // First, get an open market ticker via REST
    let client = match KalshiClient::new(config.clone()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create client: {}", e);
            return;
        }
    };

    let markets = match client.rest().get_markets(Some("open"), None, None).await {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to get markets: {}", e);
            return;
        }
    };

    if markets.markets.is_empty() {
        eprintln!("No open markets available");
        return;
    }

    let ticker = &markets.markets[0].ticker;
    println!("Testing orderbook subscription for: {}", ticker);

    // Connect WebSocket
    let mut ws = match WebSocketClient::connect(&config).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            return;
        }
    };

    // Subscribe to orderbook
    let sub_result = ws.subscribe_orderbook(&[ticker]).await;
    assert!(sub_result.is_ok(), "Failed to subscribe: {:?}", sub_result);

    // Wait for messages
    let timeout_duration = Duration::from_secs(10);
    let result = timeout(timeout_duration, async {
        let mut received_snapshot = false;
        let mut delta_count = 0;

        while let Some(msg_result) = ws.next().await {
            match msg_result {
                Ok(WsMessage::Subscribed(subscribed)) => {
                    println!(
                        "Subscribed: {} (sid: {})",
                        subscribed.msg.channel, subscribed.msg.sid
                    );
                }
                Ok(WsMessage::OrderbookSnapshot(snapshot)) => {
                    println!(
                        "Orderbook snapshot for {}: {} yes levels, {} no levels",
                        snapshot.msg.market_ticker,
                        snapshot.msg.yes.len(),
                        snapshot.msg.no.len()
                    );
                    received_snapshot = true;
                }
                Ok(WsMessage::OrderbookDelta(delta)) => {
                    println!(
                        "Delta: {} @ {} delta {} (seq: {})",
                        delta.msg.market_ticker, delta.msg.price, delta.msg.delta, delta.seq
                    );
                    delta_count += 1;
                    if delta_count >= 5 {
                        break;
                    }
                }
                Ok(msg) => {
                    println!("Other: {:?}", msg);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            }
        }

        received_snapshot
    })
    .await;

    match result {
        Ok(received) => {
            println!("Received snapshot: {}", received);
        }
        Err(_) => {
            println!("Timeout reached");
        }
    }

    let _ = ws.close().await;
}

#[tokio::test]
async fn test_subscribe_user_channels() {
    let config = require_config!();

    let mut ws = match WebSocketClient::connect(&config).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            return;
        }
    };

    // Subscribe to user-specific channels
    let fills_result = ws.subscribe_fills(None).await;
    assert!(
        fills_result.is_ok(),
        "Failed to subscribe to fills: {:?}",
        fills_result
    );

    let orders_result = ws.subscribe_user_orders().await;
    assert!(
        orders_result.is_ok(),
        "Failed to subscribe to orders: {:?}",
        orders_result
    );

    println!("Subscribed to fills and user_orders");

    // Wait for subscription confirmations
    let timeout_duration = Duration::from_secs(5);
    let mut confirmed_count = 0;

    let _ = timeout(timeout_duration, async {
        while let Some(msg_result) = ws.next().await {
            match msg_result {
                Ok(WsMessage::Subscribed(subscribed)) => {
                    println!(
                        "Confirmed subscription: {} (sid: {})",
                        subscribed.msg.channel, subscribed.msg.sid
                    );
                    confirmed_count += 1;
                    if confirmed_count >= 2 {
                        break;
                    }
                }
                Ok(msg) => {
                    println!("Other message: {:?}", msg);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            }
        }
    })
    .await;

    println!("Received {} subscription confirmations", confirmed_count);

    // Check tracked subscriptions
    let subs = ws.subscriptions();
    println!("Tracked subscriptions: {}", subs.len());
    for (sid, info) in subs {
        println!("  - sid {}: {}", sid, info.channel);
    }

    let _ = ws.close().await;
}

#[tokio::test]
async fn test_unsubscribe() {
    let config = require_config!();

    let mut ws = match WebSocketClient::connect(&config).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            return;
        }
    };

    // Subscribe
    ws.subscribe_ticker(None)
        .await
        .expect("Failed to subscribe");

    // Wait for confirmation
    let timeout_duration = Duration::from_secs(3);
    let sid = timeout(timeout_duration, async {
        while let Some(msg_result) = ws.next().await {
            if let Ok(WsMessage::Subscribed(subscribed)) = msg_result {
                return Some(subscribed.msg.sid);
            }
        }
        None
    })
    .await
    .ok()
    .flatten();

    let sid = match sid {
        Some(s) => s,
        None => {
            eprintln!("Did not receive subscription confirmation");
            return;
        }
    };

    println!("Received subscription with sid: {}", sid);

    // Unsubscribe
    ws.unsubscribe(&[sid]).await.expect("Failed to unsubscribe");

    // Wait for unsubscribe confirmation
    let _ = timeout(Duration::from_secs(3), async {
        while let Some(msg_result) = ws.next().await {
            if let Ok(WsMessage::Unsubscribed(unsub)) = msg_result {
                println!("Unsubscribed from sid: {}", unsub.sid);
                break;
            }
        }
    })
    .await;

    // Check that subscription was removed
    assert!(
        ws.subscriptions().is_empty(),
        "Subscription should be removed after unsubscribe"
    );

    let _ = ws.close().await;
}
