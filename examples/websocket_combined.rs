//! Example: Combined Order Book and Account Updates via WebSocket
//!
//! This example demonstrates monitoring both order books and accounts simultaneously.
//!
//! Prerequisites:
//! Set LIGHTER_ACCOUNT_INDEX environment variable
//!
//! Run with: cargo run --example websocket_combined

use lighter_rs::ws_client::{OrderBook, WsClient};
use serde_json::Value;
use std::env;
use tracing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("╔═══════════════════════════════════════════════════╗");
    tracing::info!("║   Lighter RS - Combined WebSocket Monitor        ║");
    tracing::info!("╚═══════════════════════════════════════════════════╝\n");

    // Get account index from environment
    let account_index: i64 = env::var("LIGHTER_ACCOUNT_INDEX")
        .unwrap_or_else(|_| "12345".to_string())
        .parse()
        .expect("LIGHTER_ACCOUNT_INDEX must be a valid number");

    tracing::info!("Configuration:");
    tracing::info!("  Markets: 0, 1");
    tracing::info!("  Account: {}", account_index);
    // tracing::info!("  WebSocket: wss://api-testnet.lighter.xyz/stream\n");

    // Create WebSocket client with both subscriptions
    let client = WsClient::builder()
        .host("mainnet.zklighter.elliot.ai")
        .order_books(vec![0, 1])
        .accounts(vec![account_index])
        .build()?;

    // Counter for updates
    let update_counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));

    // Order book update callback
    let ob_counter = update_counter.clone();
    let on_order_book_update = move |market_id: String, order_book: OrderBook| {
        let count = ob_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        tracing::info!("📊 Order Book #{} - Market {}", count + 1, market_id);

        if let (Some(best_ask), Some(best_bid)) = (order_book.asks.first(), order_book.bids.first())
        {
            tracing::info!("  Best Ask: {} @ {}", best_ask.size, best_ask.price);
            tracing::info!("  Best Bid: {} @ {}", best_bid.size, best_bid.price);

            if let (Ok(ask), Ok(bid)) =
                (best_ask.price.parse::<f64>(), best_bid.price.parse::<f64>())
            {
                let mid = (ask + bid) / 2.0;
                tracing::info!("  Mid Price: {:.2}", mid);
            }
        }
        tracing::info!();
    };

    // Account update callback
    let acc_counter = update_counter.clone();
    let on_account_update = move |account_id: String, account_data: Value| {
        let count = acc_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        tracing::info!("👤 Account #{} - ID: {}", count + 1, account_id);

        if let Some(obj) = account_data.as_object() {
            if let Some(balance) = obj.get("usdc_balance") {
                tracing::info!("  Balance: {} USDC", balance);
            }

            if let Some(orders) = obj.get("orders").and_then(|o| o.as_array()) {
                tracing::info!("  Active Orders: {}", orders.len());
            }

            if let Some(pnl) = obj.get("unrealized_pnl") {
                tracing::info!("  Unrealized PnL: {}", pnl);
            }
        }
        tracing::info!();
    };

    tracing::info!("Starting WebSocket monitor...");
    tracing::info!("Monitoring real-time updates for:");
    tracing::info!("  ✓ Order books (markets 0, 1)");
    tracing::info!("  ✓ Account {}", account_index);
    tracing::info!("\nPress Ctrl+C to stop\n");
    tracing::info!("{}\n", "═".repeat(50));

    // Run the WebSocket client
    client.run(on_order_book_update, on_account_update).await?;

    Ok(())
}
