//! Quick test to verify API authentication and endpoints
//!
//! Usage:
//!   KALSHI_API_KEY=xxx KALSHI_PRIVATE_KEY_PATH=path/to/key.pem cargo run --example demo_test
//!
//! Optional:
//!   KALSHI_ENV=demo  # Use demo environment (default: production)

use kalshi_rs::config::Environment;
use kalshi_rs::{Config, KalshiClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read credentials from environment
    let api_key =
        std::env::var("KALSHI_API_KEY").expect("Set KALSHI_API_KEY environment variable");
    let key_path = std::env::var("KALSHI_PRIVATE_KEY_PATH")
        .expect("Set KALSHI_PRIVATE_KEY_PATH environment variable");

    let private_key = std::fs::read_to_string(&key_path)?;

    // Determine environment (default to production)
    let env = match std::env::var("KALSHI_ENV")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "demo" => Environment::Demo,
        _ => Environment::Production,
    };

    println!("API Key: {}", api_key);
    println!(
        "Key format: {}",
        if private_key.contains("BEGIN RSA PRIVATE KEY") {
            "PKCS#1"
        } else {
            "PKCS#8"
        }
    );
    println!(
        "Environment: {}",
        match env {
            Environment::Demo => "Demo",
            Environment::Production => "Production",
        }
    );

    // Create config
    let config = Config::new(&api_key, &private_key).with_environment(env);

    println!("Base URL: {}", config.rest_base_url());

    // Create client
    let client = KalshiClient::new(config)?;
    println!("Client created successfully!\n");

    // Test unauthenticated endpoint first
    println!("=== Exchange Status ===");
    match client.rest().get_exchange_status().await {
        Ok(status) => println!(
            "trading_active={}, exchange_active={}",
            status.trading_active, status.exchange_active
        ),
        Err(e) => println!("Error: {}", e),
    }

    // Test authenticated endpoint
    println!("\n=== Balance ===");
    match client.rest().get_balance().await {
        Ok(balance) => println!(
            "Balance: {} centi-cents (${:.2})",
            balance.balance,
            balance.balance as f64 / 10000.0
        ),
        Err(e) => println!("Auth error: {}", e),
    }

    // Get markets
    println!("\n=== Markets ===");
    match client.rest().get_markets(Some("open"), None, None).await {
        Ok(response) => {
            println!("Found {} markets", response.markets.len());
            for market in response.markets.iter().take(3) {
                println!(
                    "  {} | {:?} | bid:{:?} ask:{:?}",
                    market.ticker, market.status, market.yes_bid, market.yes_ask
                );
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Get positions
    println!("\n=== Positions ===");
    match client.rest().get_positions(None, None, None, None).await {
        Ok(response) => {
            println!("Found {} positions", response.market_positions.len());
            for pos in response.market_positions.iter().take(5) {
                println!(
                    "  {} | position:{} | cost:{}",
                    pos.ticker, pos.position, pos.position_cost
                );
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Get orders
    println!("\n=== Orders ===");
    match client.rest().get_orders(None, None, None).await {
        Ok(response) => {
            println!("Found {} orders", response.orders.len());
            for order in response.orders.iter().take(5) {
                println!(
                    "  {} | {} {:?} {:?} | {} @ {} | status:{:?}",
                    order.order_id,
                    order.ticker,
                    order.side,
                    order.action,
                    order.remaining_count,
                    order.yes_price,
                    order.status
                );
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Get fills
    println!("\n=== Recent Fills ===");
    match client.rest().get_fills(None, None, None, None).await {
        Ok(response) => {
            println!("Found {} fills", response.fills.len());
            for fill in response.fills.iter().take(5) {
                println!(
                    "  {} | {} {} | {} @ {}",
                    fill.ticker, fill.side, fill.action, fill.count, fill.yes_price
                );
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Test Complete ===");
    Ok(())
}
