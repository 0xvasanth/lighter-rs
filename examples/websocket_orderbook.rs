//! Example: Real-time Order Book Updates via WebSocket
//!
//! This example demonstrates how to:
//! 1. Connect to Lighter WebSocket
//! 2. Subscribe to order book updates
//! 3. Handle real-time price updates
//!
//! Run with: cargo run --example websocket_orderbook

use lighter_rs::ws_client::{OrderBook, WsClient};
use tracing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("╔═══════════════════════════════════════════════════╗");
    tracing::info!("║   Lighter RS - WebSocket Order Book Example      ║");
    tracing::info!("╚═══════════════════════════════════════════════════╝\n");

    // Create WebSocket client
    let client = WsClient::builder()
        .host("api-testnet.lighter.xyz")
        .order_books(vec![0, 1]) // Subscribe to markets 0 and 1
        .build()?;

    tracing::info!("Connecting to WebSocket...");
    tracing::info!("Subscriptions: markets 0, 1\n");

    // Define callback for order book updates
    let on_order_book_update = |market_id: String, order_book: OrderBook| {
        tracing::info!("═══ Order Book Update: Market {} ═══", market_id);

        // Display top 5 asks
        tracing::info!("\n  📈 Top 5 Asks (Sell Orders):");
        for (i, ask) in order_book.asks.iter().take(5).enumerate() {
            tracing::info!(
                "    {}. Price: {:>12} | Size: {:>12}",
                i + 1,
                ask.price,
                ask.size
            );
        }

        // Display top 5 bids
        tracing::info!("\n  📉 Top 5 Bids (Buy Orders):");
        for (i, bid) in order_book.bids.iter().take(5).enumerate() {
            tracing::info!(
                "    {}. Price: {:>12} | Size: {:>12}",
                i + 1,
                bid.price,
                bid.size
            );
        }

        // Calculate spread
        if let (Some(best_ask), Some(best_bid)) = (order_book.asks.first(), order_book.bids.first())
        {
            if let (Ok(ask_price), Ok(bid_price)) =
                (best_ask.price.parse::<f64>(), best_bid.price.parse::<f64>())
            {
                let spread = ask_price - bid_price;
                let spread_bps = (spread / bid_price) * 10000.0;
                tracing::info!("\n  💰 Spread: {:.2} ({:.2} bps)", spread, spread_bps);
            }
        }

        tracing::info!("\n{}\n", "─".repeat(50));
    };

    // Placeholder for account updates (not used in this example)
    let on_account_update = |_account_id: String, _account_data: serde_json::Value| {};

    tracing::info!("Starting WebSocket stream...");
    tracing::info!("Press Ctrl+C to stop\n");
    tracing::info!("{}\n", "═".repeat(50));

    // Run the WebSocket client
    client.run(on_order_book_update, on_account_update).await?;

    Ok(())
}
