//! Find the most active market (by bid/ask spread)

use kalshi_trading::config::Environment;
use kalshi_trading::{Config, KalshiClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("KALSHI_API_KEY")?;
    let key_path = std::env::var("KALSHI_PRIVATE_KEY_PATH")?;
    let private_key = std::fs::read_to_string(&key_path)?;

    let config = Config::new(&api_key, &private_key).with_environment(Environment::Production);
    let client = KalshiClient::new(config)?;

    let markets = client.rest().get_markets(Some("open"), None, None).await?;

    // Find markets with both bid and ask (tightest spread = most liquid)
    let mut active_markets: Vec<_> = markets
        .markets
        .iter()
        .filter_map(|m| {
            let bid = m.yes_bid?;
            let ask = m.yes_ask?;
            if bid > 0 && ask > 0 && ask > bid {
                Some((m, ask - bid, bid, ask))
            } else {
                None
            }
        })
        .collect();

    // Sort by spread (tightest first)
    active_markets.sort_by_key(|(_, spread, _, _)| *spread);

    println!("Top 10 most liquid markets:\n");
    for (market, spread, bid, ask) in active_markets.iter().take(10) {
        println!(
            "{}\n  {} \n  bid: {} | ask: {} | spread: {}\n",
            market.ticker, market.title, bid, ask, spread
        );
    }

    if let Some((market, _, _, _)) = active_markets.first() {
        println!("\nMost liquid: {}", market.ticker);
    }

    Ok(())
}
