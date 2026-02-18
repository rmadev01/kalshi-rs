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

- [ ] `orderbook_delta` - Real-time orderbook updates
- [ ] `ticker` - Price/volume updates  
- [ ] `trade` - Public trade feed
- [ ] `fill` - Your fill notifications
- [ ] `user_orders` - Your order updates
- [ ] `market_lifecycle_v2` - Market state changes

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
