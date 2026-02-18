//! Live WebSocket test - streams real-time orderbook data
//!
//! Usage:
//!   KALSHI_API_KEY=xxx KALSHI_PRIVATE_KEY_PATH=path/to/key.pem cargo run --example websocket_live
//!
//! Optional:
//!   KALSHI_ENV=demo  # Use demo environment (default: production)
//!   KALSHI_TICKER=TICKER  # Specific market ticker (default: auto-selects active market)

use kalshi_trading::client::websocket::WebSocketClient;
use kalshi_trading::config::Environment;
use kalshi_trading::orderbook::Orderbook;
use kalshi_trading::types::messages::WsMessage;
use kalshi_trading::types::Side;
use kalshi_trading::{Config, KalshiClient};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for debug output
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("kalshi_trading=info".parse().unwrap()),
        )
        .init();

    // Read credentials from environment
    let api_key =
        std::env::var("KALSHI_API_KEY").expect("Set KALSHI_API_KEY environment variable");
    let key_path = std::env::var("KALSHI_PRIVATE_KEY_PATH")
        .expect("Set KALSHI_PRIVATE_KEY_PATH environment variable");

    let private_key = std::fs::read_to_string(&key_path)?;

    // Determine environment
    let env = match std::env::var("KALSHI_ENV")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "demo" => Environment::Demo,
        _ => Environment::Production,
    };

    println!("=== Kalshi WebSocket Live Test ===\n");

    // Create REST client to find an active market
    let config = Config::new(&api_key, &private_key).with_environment(env.clone());
    let rest_client = KalshiClient::new(config)?;

    // Get ticker from env or find an active market
    let ticker = match std::env::var("KALSHI_TICKER") {
        Ok(t) => t,
        Err(_) => {
            println!("Finding an active market...");
            let markets = rest_client
                .rest()
                .get_markets(Some("open"), None, None)
                .await?;

            // Find a market with some activity (has volume or bids/asks)
            let active_market = markets
                .markets
                .iter()
                .filter(|m| m.volume > 0 || (m.yes_bid.is_some() && m.yes_ask.is_some()))
                .max_by_key(|m| m.volume)
                .or(markets.markets.first())
                .ok_or("No markets found")?;

            println!(
                "Selected: {} - {} (volume: {})",
                active_market.ticker, active_market.title, active_market.volume
            );
            active_market.ticker.clone()
        }
    };

    println!("\nConnecting to WebSocket...");

    // Create WebSocket config
    let ws_config = Config::new(&api_key, &private_key).with_environment(env);

    // Connect to WebSocket
    let mut ws_client = WebSocketClient::connect(&ws_config).await?;
    println!("Connected!\n");

    // Subscribe to orderbook deltas for the selected market
    println!("Subscribing to orderbook for {}...", ticker);
    let sid = ws_client.subscribe_orderbook(&[&ticker]).await?;
    println!("Subscribed with sid: {}\n", sid);

    // Also subscribe to ticker updates
    println!("Subscribing to ticker...");
    let ticker_sid = ws_client.subscribe_ticker(Some(&[&ticker])).await?;
    println!("Subscribed with sid: {}\n", ticker_sid);

    // Track orderbooks
    let mut orderbooks: HashMap<String, Orderbook> = HashMap::new();
    orderbooks.insert(ticker.clone(), Orderbook::new(&ticker));

    println!("=== Streaming Live Data ===");
    println!("(Press Ctrl+C to stop)\n");

    let mut message_count = 0u64;
    let start_time = std::time::Instant::now();

    // Process messages
    while let Some(msg_result) = ws_client.next().await {
        match msg_result {
            Ok(msg) => {
                message_count += 1;

                match msg {
                    WsMessage::OrderbookSnapshot(snapshot) => {
                        let data = &snapshot.msg;
                        println!(
                            "[SNAPSHOT] {} | {} yes levels, {} no levels | seq: {}",
                            data.market_ticker,
                            data.yes.len(),
                            data.no.len(),
                            snapshot.seq
                        );

                        // Initialize orderbook from snapshot
                        if let Some(book) = orderbooks.get_mut(&data.market_ticker) {
                            book.clear();
                            for level in &data.yes {
                                book.set_level(level[0] as i64, level[1] as i64, Side::Yes);
                            }
                            for level in &data.no {
                                book.set_level(level[0] as i64, level[1] as i64, Side::No);
                            }
                            print_book_summary(book);
                        }
                    }

                    WsMessage::OrderbookDelta(delta) => {
                        let data = &delta.msg;
                        if let Some(book) = orderbooks.get_mut(&data.market_ticker) {
                            // Apply delta
                            book.apply_delta(data.price, data.delta, data.side);

                            println!(
                                "[DELTA] {} | {:?} @ {} (delta: {:+}) | seq: {}",
                                data.market_ticker, data.side, data.price, data.delta, delta.seq
                            );
                            print_book_summary(book);
                        }
                    }

                    WsMessage::Ticker(ticker_msg) => {
                        let data = &ticker_msg.msg;
                        println!(
                            "[TICKER] {} | yes_bid: {:?} yes_ask: {:?} | volume: {:?}",
                            data.market_ticker, data.yes_bid, data.yes_ask, data.volume
                        );
                    }

                    WsMessage::Trade(trade) => {
                        let data = &trade.msg;
                        println!(
                            "[TRADE] {} | {} contracts @ {} | taker: {:?}",
                            data.market_ticker, data.count, data.yes_price, data.taker_side
                        );
                    }

                    WsMessage::Fill(fill) => {
                        let data = &fill.msg;
                        println!(
                            "[FILL] {} | {:?} {} @ {} | order: {}",
                            data.market_ticker, data.side, data.count, data.yes_price, data.order_id
                        );
                    }

                    WsMessage::Error(err) => {
                        println!("[ERROR] code: {}, msg: {}", err.msg.code, err.msg.msg);
                    }

                    WsMessage::Subscribed(sub) => {
                        println!(
                            "[SUBSCRIBED] sid: {}, channel: {}",
                            sub.msg.sid, sub.msg.channel
                        );
                    }

                    WsMessage::Unsubscribed(unsub) => {
                        println!("[UNSUBSCRIBED] sid: {}", unsub.sid);
                    }

                    _ => {
                        // Other message types (SubscriptionUpdated, SubscriptionsList, etc.)
                    }
                }

                // Print stats every 50 messages
                if message_count % 50 == 0 {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    println!(
                        "\n--- {} messages in {:.1}s ({:.1} msg/s) ---\n",
                        message_count,
                        elapsed,
                        message_count as f64 / elapsed
                    );
                }
            }
            Err(e) => {
                println!("[ERROR] WebSocket error: {}", e);
            }
        }
    }

    println!("\nWebSocket closed");
    Ok(())
}

fn print_book_summary(book: &Orderbook) {
    let (best_bid, bid_qty) = book.best_bid().unwrap_or((0, 0));
    let (best_ask, ask_qty) = book.best_ask().unwrap_or((0, 0));
    let spread = book.spread().unwrap_or(0);
    let mid = book.mid_price().unwrap_or(0.0);

    println!(
        "         BID: {} @ {} | ASK: {} @ {} | spread: {} | mid: ${:.2}",
        bid_qty,
        best_bid,
        ask_qty,
        best_ask,
        spread,
        mid / 100.0
    );
}
