//! Quick test to verify API authentication works with PKCS#1 keys
use kalshi_rs::{Config, KalshiClient};
use kalshi_rs::config::Environment;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read credentials from environment
    let api_key = std::env::var("KALSHI_API_KEY")
        .expect("Set KALSHI_API_KEY environment variable");
    let key_path = std::env::var("KALSHI_PRIVATE_KEY_PATH")
        .expect("Set KALSHI_PRIVATE_KEY_PATH environment variable");
    
    let private_key = std::fs::read_to_string(&key_path)?;
    
    println!("API Key: {}", api_key);
    println!("Key format: {}", if private_key.contains("BEGIN RSA PRIVATE KEY") {
        "PKCS#1"
    } else {
        "PKCS#8"
    });
    
    // Create config for demo environment
    let config = Config::new(&api_key, &private_key)
        .with_environment(Environment::Demo);
    
    println!("Config created successfully!");
    println!("Base URL: {}", config.rest_base_url());
    
    // Create client
    let client = KalshiClient::new(config)?;
    println!("Client created successfully!");
    
    // Test unauthenticated endpoint first
    println!("\nTesting unauthenticated endpoint (exchange status)...");
    match client.rest().get_exchange_status().await {
        Ok(status) => println!("Exchange status: trading_active={}", status.trading_active),
        Err(e) => println!("Error: {}", e),
    }
    
    // Test authenticated endpoint
    println!("\nTesting authenticated endpoint (get balance)...");
    match client.rest().get_balance().await {
        Ok(balance) => println!("Balance: {} centi-cents (${:.2})", balance.balance, balance.balance as f64 / 10000.0),
        Err(e) => println!("Auth error: {}", e),
    }
    
    // Get some markets
    println!("\nFetching markets...");
    match client.rest().get_markets(Some("open"), None, None).await {
        Ok(response) => {
            println!("Found {} markets", response.markets.len());
            if let Some(market) = response.markets.first() {
                println!("  First market: {} - {}", market.ticker, market.title);
            }
        },
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\nTest complete!");
    Ok(())
}
