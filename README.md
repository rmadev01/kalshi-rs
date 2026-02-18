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
```

## Quick Start

```rust
use kalshi_rs::{KalshiClient, Config};

#[tokio::main]
async fn main() -> Result<(), kalshi_rs::Error> {
    // Create client with API credentials
    let config = Config::new(
        "your-api-key-id",
        include_str!("path/to/private_key.pem"),
    );
    
    let client = KalshiClient::new(config)?;
    
    // Get markets
    let markets = client.get_markets().await?;
    println!("Found {} markets", markets.len());
    
    // Subscribe to orderbook updates
    let mut ws = client.websocket().await?;
    ws.subscribe_orderbook(&["KXBTC-25JAN"]).await?;
    
    while let Some(msg) = ws.next().await {
        match msg? {
            Message::OrderbookSnapshot(snap) => {
                println!("Snapshot: {:?}", snap);
            }
            Message::OrderbookDelta(delta) => {
                println!("Delta: {:?}", delta);
            }
            _ => {}
        }
    }
    
    Ok(())
}
```

## Architecture

### Performance Optimizations

This crate is designed for low-latency trading:

| Component | Optimization |
|-----------|-------------|
| **Orderbook** | `BTreeMap` for sorted price levels, integer prices (no floats) |
| **Hashing** | `FxHashMap` (2-3x faster than std for small keys) |
| **Locking** | `parking_lot` mutexes (faster, no poisoning) |
| **Memory** | Pre-allocated buffers, minimal allocations in hot paths |
| **Parsing** | Serde with `#[serde(borrow)]` for zero-copy where possible |

### Module Structure

```
kalshi-rs/
├── client/       # REST and WebSocket clients
│   ├── rest      # HTTP client with retry logic
│   ├── websocket # Real-time data streaming  
│   └── auth      # RSA-PSS request signing
├── types/        # API types (orders, markets, messages)
├── orderbook/    # HFT orderbook implementation
└── error         # Error types
```

## API Coverage

### REST Endpoints

- [ ] Authentication (RSA-PSS signing)
- [ ] Markets (`GET /markets`, `GET /markets/{ticker}`)
- [ ] Events (`GET /events`, `GET /events/{ticker}`)
- [ ] Orders (`POST /portfolio/orders`, `DELETE /portfolio/orders/{id}`)
- [ ] Batch Orders (`POST /portfolio/orders/batched`)
- [ ] Portfolio (`GET /portfolio/balance`, `GET /portfolio/positions`)
- [ ] Fills (`GET /portfolio/fills`)

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

This client respects rate limits and provides backoff strategies.

## Examples

See the [`examples/`](examples/) directory:

- `simple_connection.rs` - Basic REST API usage
- `orderbook_viewer.rs` - Real-time orderbook display
- `place_order.rs` - Order placement example

## Testing

```bash
# Run tests
cargo test

# Run benchmarks
cargo bench

# Run with demo environment
KALSHI_ENV=demo cargo run --example simple_connection
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Disclaimer

This software is provided as-is. Trading on Kalshi involves financial risk. The authors are not responsible for any losses incurred through use of this software.

## Contributing

Contributions welcome! Please read the contributing guidelines and submit PRs.
