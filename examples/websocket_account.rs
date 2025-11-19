//! Example: Real-time Account Updates via WebSocket
//!
//! This example demonstrates how to:
//! 1. Connect to Lighter WebSocket
//! 2. Subscribe to account updates
//! 3. Monitor account changes in real-time
//!
//! Prerequisites:
//! Set LIGHTER_ACCOUNT_INDEX environment variable
//!
//! Run with: cargo run --example websocket_account

use lighter_rs::ws_client::WsClient;
use serde_json::Value;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("╔═══════════════════════════════════════════════════╗");
    tracing::info!("║   Lighter RS - WebSocket Account Monitor         ║");
    tracing::info!("╚═══════════════════════════════════════════════════╝\n");

    // Get account index from environment
    let account_index: i64 = env::var("LIGHTER_ACCOUNT_INDEX")
        .unwrap_or_else(|_| {
            tracing::info!("⚠ LIGHTER_ACCOUNT_INDEX not set, using example account 12345");
            "12345".to_string()
        })
        .parse()
        .expect("LIGHTER_ACCOUNT_INDEX must be a valid number");

    tracing::info!("Configuration:");
    tracing::info!("  Account Index: {}", account_index);
    tracing::info!("  WebSocket: wss://api-testnet.lighter.xyz/stream\n");

    // Create WebSocket client
    let client = WsClient::builder()
        .host("api-testnet.lighter.xyz")
        .accounts(vec![account_index])
        .build()?;

    // Placeholder for order book updates (not used in this example)
    let on_order_book_update =
        |_market_id: String, _order_book: lighter_rs::ws_client::OrderBook| {};

    // Define callback for account updates
    let on_account_update = move |account_id: String, account_data: Value| {
        tracing::info!("═══ Account Update: {} ═══", account_id);

        // Extract key account information
        if let Some(obj) = account_data.as_object() {
            // Display account balance
            if let Some(balance) = obj.get("usdc_balance") {
                tracing::info!("\n  💵 USDC Balance: {}", balance);
            }

            // Display positions
            if let Some(positions) = obj.get("positions").and_then(|p| p.as_array()) {
                tracing::info!("\n  📊 Open Positions:");
                for (i, position) in positions.iter().enumerate() {
                    if let Some(pos_obj) = position.as_object() {
                        let market = pos_obj
                            .get("market_index")
                            .and_then(|m| m.as_i64())
                            .unwrap_or(0);
                        let size = pos_obj.get("size").and_then(|s| s.as_str()).unwrap_or("0");
                        let entry_price = pos_obj
                            .get("entry_price")
                            .and_then(|p| p.as_str())
                            .unwrap_or("0");

                        tracing::info!(
                            "    {}. Market {}: Size = {}, Entry = {}",
                            i + 1,
                            market,
                            size,
                            entry_price
                        );
                    }
                }
            }

            // Display active orders
            if let Some(orders) = obj.get("orders").and_then(|o| o.as_array()) {
                tracing::info!("\n  📋 Active Orders: {}", orders.len());
                for (i, order) in orders.iter().take(5).enumerate() {
                    if let Some(order_obj) = order.as_object() {
                        let side = if order_obj
                            .get("is_ask")
                            .and_then(|a| a.as_i64())
                            .unwrap_or(0)
                            == 1
                        {
                            "SELL"
                        } else {
                            "BUY"
                        };
                        let price = order_obj
                            .get("price")
                            .and_then(|p| p.as_str())
                            .unwrap_or("0");
                        let size = order_obj
                            .get("size")
                            .and_then(|s| s.as_str())
                            .unwrap_or("0");

                        tracing::info!("    {}. {} {} @ {}", i + 1, side, size, price);
                    }
                }
            }

            // Display PnL
            if let Some(pnl) = obj.get("unrealized_pnl") {
                tracing::info!("\n  💹 Unrealized PnL: {}", pnl);
            }

            // Display margin info
            if let Some(margin) = obj.get("available_margin") {
                tracing::info!("  🔒 Available Margin: {}", margin);
            }
        }

        tracing::info!("\n{}\n", "─".repeat(50));
    };

    tracing::info!("Starting WebSocket stream...");
    tracing::info!("Press Ctrl+C to stop\n");
    tracing::info!("{}\n", "═".repeat(50));

    // Run the WebSocket client
    client.run(on_order_book_update, on_account_update).await?;

    Ok(())
}
