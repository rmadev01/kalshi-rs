# kalshi-rs

A high-performance Rust client for the [Kalshi](https://kalshi.com) prediction market API, designed with HFT (High-Frequency Trading) workloads in mind.

## Features

- **REST API Client** - Full coverage of Kalshi's trading and market data endpoints
- **WebSocket Client** - Real-time orderbook, trades, fills, and market lifecycle events
- **HFT-Grade Orderbook** - Lock-free, cache-optimized orderbook with O(log n) updates
- **Zero-Copy Parsing** - Minimal allocations in hot paths
- **Async/Await** - Built on Tokio for maximum concurrency

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
kalshi-rs = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use kalshi_rs::{KalshiClient, Config};
use kalshi_rs::types::{CreateOrderRequest, Side, Action};

#[tokio::main]
async fn main() -> Result<(), kalshi_rs::Error> {
    // Load your private key
    let private_key = std::fs::read_to_string("private_key.pem")?;
    
    // Create client with API credentials
    let config = Config::new("your-api-key-id", &private_key);
    let client = KalshiClient::new(config)?;
    
    // Get open markets
    let markets = client.rest().get_markets(Some("open"), None, None).await?;
    println!("Found {} open markets", markets.markets.len());
    
    // Get your balance (values in centi-cents: 10000 = $1.00)
    let balance = client.rest().get_balance().await?;
    println!("Balance: ${:.2}", balance.balance as f64 / 10000.0);
    
    // Place a limit order (buy 10 Yes contracts at $0.50)
    let order = CreateOrderRequest::limit(
        "MARKET-TICKER",
        Side::Yes,
        Action::Buy,
        10,     // count
        5000,   // price in centi-cents ($0.50)
    );
    let response = client.rest().create_order(&order).await?;
    println!("Order placed: {}", response.order.order_id);
    
    // Cancel the order
    client.rest().cancel_order(&response.order.order_id).await?;
    
    Ok(())
}
```

## Price Representation

Kalshi uses **centi-cents** for subpenny precision:

| Centi-cents | Cents | Dollars | Description |
|-------------|-------|---------|-------------|
| 100 | 1¢ | $0.01 | 1% implied probability |
| 5000 | 50¢ | $0.50 | 50% implied probability |
| 9900 | 99¢ | $0.99 | 99% implied probability |
| 5050 | 50.5¢ | $0.505 | Subpenny pricing |

## Architecture

### Performance Optimizations

This crate is designed for low-latency trading:

| Component | Optimization |
|-----------|-------------|
| **Orderbook** | `BTreeMap` for sorted price levels, integer prices (no floats) |
| **Hashing** | `FxHashMap` (2-3x faster than std for small keys) |
| **Locking** | `parking_lot` mutexes (faster, no poisoning) |
| **Memory** | Pre-allocated buffers, minimal allocations in hot paths |
| **Parsing** | Serde with efficient deserialization |
| **Errors** | Boxed large variants to keep `Result` small |

### Module Structure

```
kalshi-rs/
├── client/       # REST and WebSocket clients
│   ├── rest      # HTTP client with auth
│   ├── websocket # Real-time data streaming  
│   └── auth      # RSA-PSS request signing
├── types/        # API types (orders, markets, messages)
├── orderbook/    # HFT orderbook implementation
└── error         # Error types
```

## API Coverage

### REST Endpoints

**Market Data:**
- [x] `GET /markets` - List markets with filters
- [x] `GET /markets/{ticker}` - Get single market
- [x] `GET /markets/{ticker}/orderbook` - Get orderbook
- [x] `GET /markets/trades` - Get public trades
- [x] `GET /events` - List events
- [x] `GET /events/{ticker}` - Get single event
- [x] `GET /series/{ticker}` - Get series info

**Orders:**
- [x] `POST /portfolio/orders` - Create order
- [x] `GET /portfolio/orders` - List orders
- [x] `GET /portfolio/orders/{id}` - Get single order
- [x] `DELETE /portfolio/orders/{id}` - Cancel order
- [x] `POST /portfolio/orders/{id}/amend` - Amend order
- [x] `POST /portfolio/orders/{id}/decrease` - Decrease order
- [x] `POST /portfolio/orders/batched` - Batch create orders
- [x] `DELETE /portfolio/orders/batched` - Batch cancel orders
- [x] `GET /portfolio/orders/queue_positions` - Get queue positions

**Portfolio:**
- [x] `GET /portfolio/balance` - Get balance
- [x] `GET /portfolio/positions` - Get positions
- [x] `GET /portfolio/fills` - Get fills
- [x] `GET /portfolio/settlements` - Get settlements

**Exchange:**
- [x] `GET /exchange/status` - Exchange status
- [x] `GET /exchange/schedule` - Exchange schedule

### WebSocket Channels

- [x] `orderbook_delta` - Real-time orderbook updates
- [x] `ticker` - Price/volume updates  
- [x] `trade` - Public trade feed
- [x] `fill` - Your fill notifications
- [x] `user_orders` - Your order updates
- [x] `market_lifecycle_v2` - Market state changes

## WebSocket Usage

### Basic WebSocket

```rust
use kalshi_rs::{Config, KalshiClient};
use kalshi_rs::client::websocket::WebSocketClient;

#[tokio::main]
async fn main() -> Result<(), kalshi_rs::Error> {
    let private_key = std::fs::read_to_string("private_key.pem")?;
    let config = Config::new("your-api-key-id", &private_key);
    
    // Connect to WebSocket
    let mut ws = WebSocketClient::connect(&config).await?;
    
    // Subscribe to orderbook updates
    ws.subscribe_orderbook(&["KXBTC-25JAN"]).await?;
    
    // Subscribe to your fills
    ws.subscribe_fills(None).await?;
    
    // Process messages
    while let Some(result) = ws.next().await {
        match result {
            Ok(msg) => {
                println!("Received: {:?}", msg);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}
```

### Reconnecting WebSocket

For production use, use `ReconnectingWebSocket` which automatically reconnects and replays subscriptions:

```rust
use kalshi_rs::Config;
use kalshi_rs::client::websocket::{ReconnectingWebSocket, ReconnectConfig};

#[tokio::main]
async fn main() -> Result<(), kalshi_rs::Error> {
    let private_key = std::fs::read_to_string("private_key.pem")?;
    let config = Config::new("your-api-key-id", &private_key);
    
    // Configure reconnection behavior
    let reconnect_config = ReconnectConfig::new()
        .max_retries(10)
        .initial_delay_ms(100)
        .max_delay_ms(30_000)
        .backoff_multiplier(2.0);
    
    let mut ws = ReconnectingWebSocket::connect(config, reconnect_config).await?;
    
    // Subscriptions are automatically replayed on reconnection
    ws.subscribe_orderbook(&["KXBTC-25JAN"]).await?;
    
    loop {
        match ws.next().await {
            Some(Ok(msg)) => {
                // Handle message - reconnection happens automatically
                println!("Message: {:?}", msg);
            }
            Some(Err(kalshi_rs::Error::ConnectionClosed)) => {
                // Max retries exceeded
                break;
            }
            Some(Err(e)) => {
                eprintln!("Error: {}", e);
            }
            None => break,
        }
    }
    
    Ok(())
}
```

## Orderbook Manager

For tracking multiple orderbooks with automatic WebSocket integration:

```rust
use std::sync::Arc;
use kalshi_rs::Config;
use kalshi_rs::client::websocket::WebSocketClient;
use kalshi_rs::orderbook::{OrderbookManager, OrderbookState};
use kalshi_rs::types::WsMessage;

#[tokio::main]
async fn main() -> Result<(), kalshi_rs::Error> {
    let private_key = std::fs::read_to_string("private_key.pem")?;
    let config = Config::new("your-api-key-id", &private_key);
    
    // Create thread-safe orderbook manager
    let manager = Arc::new(OrderbookManager::new());
    
    // Add markets to track
    manager.add_market("KXBTC-25JAN");
    manager.add_market("KXBTC-26JAN");
    
    // Connect and subscribe
    let mut ws = WebSocketClient::connect(&config).await?;
    ws.subscribe_orderbook(&["KXBTC-25JAN", "KXBTC-26JAN"]).await?;
    
    // Process messages
    while let Some(Ok(msg)) = ws.next().await {
        // Manager automatically applies snapshots and deltas
        match manager.process_message(&msg) {
            Ok(Some(ticker)) => {
                // Orderbook was updated
                if let Some((bid_price, bid_qty)) = manager.best_bid(&ticker) {
                    if let Some((ask_price, ask_qty)) = manager.best_ask(&ticker) {
                        println!(
                            "{}: {} @ {} / {} @ {}",
                            ticker, bid_qty, bid_price, ask_qty, ask_price
                        );
                    }
                }
            }
            Err(kalshi_rs::Error::SequenceGap { expected, got }) => {
                // Sequence gap detected - need to resync
                eprintln!("Gap: expected {}, got {} - requesting resync", expected, got);
            }
            _ => {}
        }
        
        // Check for markets needing resync
        let stale = manager.markets_needing_resync();
        if !stale.is_empty() {
            println!("Markets needing resync: {:?}", stale);
        }
    }
    
    Ok(())
}
```

## Rate Limits

Kalshi has tiered rate limits:

| Tier | Read | Write |
|------|------|-------|
| Basic | 20/s | 10/s |
| Advanced | 30/s | 30/s |
| Premier | 100/s | 100/s |
| Prime | 400/s | 400/s |

The client handles 429 rate limit responses and provides `retry_after_ms` in the error.

## Demo Environment

For testing, use Kalshi's demo environment:

```rust
use kalshi_rs::{Config, config::Environment};

let config = Config::builder()
    .api_key_id("demo-api-key")
    .private_key_pem(&private_key)
    .environment(Environment::Demo)
    .build();
```

## Testing

```bash
# Run tests
cargo test

# Run benchmarks
cargo bench

# Check for issues
cargo clippy
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Disclaimer

This software is provided as-is. Trading on Kalshi involves financial risk. The authors are not responsible for any losses incurred through use of this software.

## Contributing

Contributions welcome! Please submit PRs with tests.
