//! Example: Simple Trading Bot with WebSocket Monitoring
//!
//! This example demonstrates a complete trading bot that:
//! 1. Monitors order book via WebSocket
//! 2. Monitors account state via WebSocket
//! 3. Places orders based on market conditions
//!
//! This is a simple example for educational purposes.
//! DO NOT use in production without proper risk management!
//!
//! Prerequisites - .env file:
//! - Set LIGHTER_API_KEY
//! - Set LIGHTER_ACCOUNT_INDEX
//! - Set LIGHTER_API_KEY_INDEX
//!
//! Run with: cargo run --example trading_bot_simple

use lighter_rs::client::TxClient;
use lighter_rs::ws_client::{OrderBook, WsClient};
use serde_json::Value;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("╔═══════════════════════════════════════════════════╗");
    tracing::info!("║   Lighter RS - Simple Trading Bot Example        ║");
    tracing::info!("║   WARNING: Educational purposes only!             ║");
    tracing::info!("╚═══════════════════════════════════════════════════╝\n");

    // Load .env file
    dotenv::dotenv().ok();

    // Load configuration
    let api_key =
        env::var("LIGHTER_API_KEY").expect("LIGHTER_API_KEY environment variable not set");

    let account_index: i64 = env::var("LIGHTER_ACCOUNT_INDEX")
        .expect("LIGHTER_ACCOUNT_INDEX environment variable not set")
        .parse()
        .expect("LIGHTER_ACCOUNT_INDEX must be a valid number");

    let api_key_index: u8 = env::var("LIGHTER_API_KEY_INDEX")
        .expect("LIGHTER_ACCOUNT_INDEX environment variable not set")
        .parse()
        .expect("LIGHTER_ACCOUNT_INDEX must be a valid number");

    let market_index = 0u8; // Trading on market 0 -> ETH
    let url = "https://mainnet.zklighter.elliot.ai";
    let url_ws = "mainnet.zklighter.elliot.ai";

    tracing::info!("Bot Configuration:");
    tracing::info!("  Account: {}", account_index);
    tracing::info!("  Market: {}", market_index);
    tracing::info!("  Mode: Demo (Educational)\n");

    // Create trading client
    let tx_client = Arc::new(TxClient::new(
        url,
        &api_key,
        account_index,
        api_key_index, // api_key_index
        304,           // 300 Testnet; 304 Mainnet
    )?);

    // Flag to track if we've placed an order
    let order_placed = Arc::new(AtomicBool::new(false));

    tracing::info!("✓ Trading client initialized\n");

    // Create WebSocket client
    let ws_client = WsClient::builder()
        .host(url_ws)
        .order_books(vec![market_index as u32])
        .accounts(vec![account_index])
        .build()?;

    tracing::info!("✓ WebSocket client created");
    tracing::info!("  Monitoring market {}", market_index);
    tracing::info!("  Monitoring account {}\n", account_index);

    // Clone for callbacks
    let tx_client_clone = tx_client.clone();
    let order_placed_clone = order_placed.clone();

    // Order book callback - Simple trading logic
    let on_order_book_update = move |market_id: String, order_book: OrderBook| {
        tracing::info!("📊 Order Book Update - Market {}", market_id);

        if let (Some(best_ask), Some(best_bid)) = (order_book.asks.first(), order_book.bids.first())
        {
            tracing::info!("  Best Ask: {} @ {}", best_ask.size, best_ask.price);
            tracing::info!("  Best Bid: {} @ {}", best_bid.size, best_bid.price);

            // Parse prices
            if let (Ok(ask_price), Ok(bid_price)) =
                (best_ask.price.parse::<f64>(), best_bid.price.parse::<f64>())
            {
                let spread = ask_price - bid_price;
                let spread_bps = (spread / bid_price) * 10000.0;

                tracing::info!("  Spread: {:.2} ({:.2} bps)", spread, spread_bps);

                // Simple trading logic: Place order if spread > 10 bps
                if spread_bps > 10.0 && !order_placed_clone.load(Ordering::Relaxed) {
                    tracing::info!("\n  🎯 Spread > 10 bps detected! Placing order...");

                    let tx_client = tx_client_clone.clone();
                    let order_placed = order_placed_clone.clone();

                    // Spawn task to place order (non-blocking)
                    tokio::spawn(async move {
                        // Place a small buy order at mid price
                        let mid_price = ((ask_price + bid_price) / 2.0) as u32;

                        match tx_client
                            .create_market_order(
                                market_index,
                                chrono::Utc::now().timestamp_millis(),
                                100_000, // Small size for demo
                                mid_price,
                                0,     // BUY
                                false, // not reduce-only
                                None,
                            )
                            .await
                        {
                            Ok(order) => {
                                tracing::info!("  ✓ Order created and signed");
                                match tx_client.send_transaction(&order).await {
                                    Ok(response) => {
                                        if response.code == 200 {
                                            tracing::info!("  ✓ Order submitted successfully!");
                                            if let Some(hash) = response.tx_hash {
                                                tracing::info!("    Tx Hash: {}", hash);
                                            }
                                            order_placed.store(true, Ordering::Relaxed);
                                        } else {
                                            tracing::info!("  ✗ Order failed: {:?}", response.message);
                                        }
                                    }
                                    Err(e) => tracing::info!("  ✗ Submit error: {}", e),
                                }
                            }
                            Err(e) => tracing::info!("  ✗ Order creation error: {}", e),
                        }
                    });
                }
            }
        }
        tracing::info!();
    };

    // Account callback - Monitor our positions
    let on_account_update = move |account_id: String, account_data: Value| {
        tracing::info!("👤 Account Update - ID: {}", account_id);

        if let Some(obj) = account_data.as_object() {
            if let Some(balance) = obj.get("usdc_balance") {
                tracing::info!("  Balance: {} USDC", balance);
            }

            if let Some(orders) = obj.get("orders").and_then(|o| o.as_array()) {
                tracing::info!("  Active Orders: {}", orders.len());
            }

            if let Some(positions) = obj.get("positions").and_then(|p| p.as_array()) {
                if !positions.is_empty() {
                    tracing::info!("  Open Positions: {}", positions.len());
                }
            }

            if let Some(pnl) = obj.get("unrealized_pnl") {
                tracing::info!("  Unrealized PnL: {}", pnl);
            }
        }
        tracing::info!();
    };

    tracing::info!("╔═══════════════════════════════════════════════════╗");
    tracing::info!("║   Trading Bot Started                             ║");
    tracing::info!("╚═══════════════════════════════════════════════════╝");
    tracing::info!("\nStrategy: Place order when spread > 10 bps");
    tracing::info!("Press Ctrl+C to stop\n");
    tracing::info!("{}\n", "═".repeat(50));

    // Run the WebSocket client
    ws_client
        .run(on_order_book_update, on_account_update)
        .await?;

    Ok(())
}
