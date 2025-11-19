//! Example: WebSocket Trades and Market Data Monitor
//!
//! This example demonstrates:
//! 1. Connecting to Lighter WebSocket with proper URL format
//! 2. Monitoring real-time order book updates
//! 3. Tracking market data and price action
//! 4. Simple trading logic based on market conditions
//!
//! Setup:
//! 1. Ensure .env file exists with your credentials
//! 2. Update LIGHTER_ACCOUNT_INDEX with your actual account
//! 3. Run: cargo run --example websocket_trades_monitor

use dotenv::dotenv;
use lighter_rs::client::TxClient;
use lighter_rs::ws_client::{OrderBook, WsClient};
use serde_json::Value;
use std::env;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tracing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    tracing::info!("╔═══════════════════════════════════════════════════╗");
    tracing::info!("║   Lighter RS - WebSocket Trades Monitor          ║");
    tracing::info!("╚═══════════════════════════════════════════════════╝\n");

    // Load configuration
    let api_key = env::var("LIGHTER_API_KEY").expect("LIGHTER_API_KEY not found in .env");

    let account_index: i64 = env::var("LIGHTER_ACCOUNT_INDEX")
        .unwrap_or_else(|_| "12345".to_string())
        .parse()
        .expect("LIGHTER_ACCOUNT_INDEX must be a number");

    let api_url =
        env::var("LIGHTER_API_URL").unwrap_or_else(|_| "https://api.lighter.xyz".to_string());

    let chain_id: u32 = env::var("LIGHTER_CHAIN_ID")
        .unwrap_or_else(|_| "304".to_string())
        .parse()
        .unwrap_or(304);

    // Use dedicated WebSocket host from environment
    let ws_host = env::var("LIGHTER_WS_HOST").unwrap_or_else(|_| "ws.lighter.xyz".to_string());

    tracing::info!("Configuration:");
    tracing::info!("  API: {}", api_url);
    tracing::info!("  WebSocket: wss://{}/stream", ws_host);
    tracing::info!("  Account: {}", account_index);
    tracing::info!("  Chain ID: {}\n", chain_id);

    // Create trading client
    let tx_client = Arc::new(TxClient::new(
        &api_url,
        &api_key,
        account_index,
        0,
        chain_id,
    )?);

    tracing::info!("✓ Trading client initialized\n");

    // Create WebSocket client
    tracing::info!("Creating WebSocket connection...");
    tracing::info!("  Host: {}", ws_host);
    tracing::info!("  Path: /stream");
    tracing::info!("  Subscriptions: market 0, account {}\n", account_index);

    let ws_client = WsClient::builder()
        .host(&ws_host)
        .path("/stream")
        .order_books(vec![0]) // Monitor market 0
        .accounts(vec![account_index])
        .build()?;

    tracing::info!("✓ WebSocket client created\n");

    // Price tracking
    let last_mid_price = Arc::new(tokio::sync::RwLock::new(0.0f64));
    let update_count = Arc::new(AtomicU32::new(0));
    let trade_count = Arc::new(AtomicU32::new(0));

    // Clone for callbacks
    let last_mid_clone = last_mid_price.clone();
    let update_count_clone = update_count.clone();
    let trade_count_clone = trade_count.clone();
    let tx_client_clone = tx_client.clone();

    // Order book callback - Market data analysis
    let on_order_book_update = move |market_id: String, order_book: OrderBook| {
        let count = update_count_clone.fetch_add(1, Ordering::Relaxed) + 1;

        tracing::info!("═══ Update #{} - Market {} ═══", count, market_id);

        if let (Some(best_ask), Some(best_bid)) = (order_book.asks.first(), order_book.bids.first())
        {
            if let (Ok(ask_price), Ok(bid_price)) =
                (best_ask.price.parse::<f64>(), best_bid.price.parse::<f64>())
            {
                let mid_price = (ask_price + bid_price) / 2.0;
                let spread = ask_price - bid_price;
                let spread_bps = (spread / bid_price) * 10000.0;

                tracing::info!("📊 Market Data:");
                tracing::info!("  Best Ask: ${:.4} (Size: {})", ask_price, best_ask.size);
                tracing::info!("  Best Bid: ${:.4} (Size: {})", bid_price, best_bid.size);
                tracing::info!("  Mid Price: ${:.4}", mid_price);
                tracing::info!("  Spread: ${:.4} ({:.2} bps)", spread, spread_bps);

                // Calculate order book depth
                let ask_depth: f64 = order_book
                    .asks
                    .iter()
                    .take(5)
                    .filter_map(|level| level.size.parse::<f64>().ok())
                    .sum();

                let bid_depth: f64 = order_book
                    .bids
                    .iter()
                    .take(5)
                    .filter_map(|level| level.size.parse::<f64>().ok())
                    .sum();

                tracing::info!(
                    "  Depth (top 5): Asks {:.2} | Bids {:.2}",
                    ask_depth, bid_depth
                );

                // Track price movement
                let last_mid_clone = last_mid_clone.clone();
                let trade_count = trade_count_clone.clone();
                let tx_client = tx_client_clone.clone();

                tokio::spawn(async move {
                    let mut last_mid = last_mid_clone.write().await;

                    if *last_mid > 0.0 {
                        let price_change = mid_price - *last_mid;
                        let price_change_pct = (price_change / *last_mid) * 100.0;

                        tracing::info!(
                            "  📈 Price Change: ${:.4} ({:+.2}%)",
                            price_change, price_change_pct
                        );

                        // Simple trading logic: Trade on significant price moves
                        if price_change_pct.abs() > 0.1 && trade_count.load(Ordering::Relaxed) < 2 {
                            tracing::info!(
                                "\n  🎯 TRADING SIGNAL: Price moved {:+.2}%",
                                price_change_pct
                            );

                            let is_ask = if price_change_pct > 0.0 { 1 } else { 0 }; // Sell if price up, buy if down

                            tracing::info!(
                                "     Action: {} at ${:.4}",
                                if is_ask == 1 { "SELL" } else { "BUY" },
                                mid_price
                            );

                            // Create small market order
                            match tx_client
                                .create_market_order(
                                    0,
                                    chrono::Utc::now().timestamp_millis(),
                                    50_000, // Very small size
                                    mid_price as u32,
                                    is_ask,
                                    false,
                                    None,
                                )
                                .await
                            {
                                Ok(order) => {
                                    tracing::info!("     ✓ Order signed (nonce: {})", order.nonce);

                                    match tx_client.send_transaction(&order).await {
                                        Ok(response) => {
                                            if response.code == 200 {
                                                tracing::info!("     ✓ Order submitted!");
                                                if let Some(hash) = response.tx_hash {
                                                    tracing::info!("       Tx: {}...", &hash[..16]);
                                                }
                                                trade_count.fetch_add(1, Ordering::Relaxed);
                                            } else {
                                                tracing::info!("     ✗ Rejected: {:?}", response.message);
                                            }
                                        }
                                        Err(e) => tracing::info!("     ✗ Submit error: {}", e),
                                    }
                                }
                                Err(e) => tracing::info!("     ✗ Order error: {}", e),
                            }
                        }
                    }

                    *last_mid = mid_price;
                });

                tracing::info!("");
            }
        }
    };

    // Account callback
    let on_account_update = move |account_id: String, account_data: Value| {
        tracing::info!("👤 Account {} Update", account_id);

        if let Some(obj) = account_data.as_object() {
            if let Some(balance) = obj.get("usdc_balance").and_then(|b| b.as_str()) {
                if let Ok(balance_num) = balance.parse::<f64>() {
                    tracing::info!("  💵 Balance: ${:.2} USDC", balance_num / 1_000_000.0);
                }
            }

            if let Some(orders) = obj.get("orders").and_then(|o| o.as_array()) {
                tracing::info!("  📋 Active Orders: {}", orders.len());
            }

            if let Some(positions) = obj.get("positions").and_then(|p| p.as_array()) {
                if !positions.is_empty() {
                    tracing::info!("  📊 Positions: {}", positions.len());
                    for (i, pos) in positions.iter().take(3).enumerate() {
                        if let Some(pos_obj) = pos.as_object() {
                            let size = pos_obj.get("size").and_then(|s| s.as_str()).unwrap_or("0");
                            let market = pos_obj
                                .get("market_index")
                                .and_then(|m| m.as_i64())
                                .unwrap_or(0);
                            tracing::info!("    {}. Market {}: Size {}", i + 1, market, size);
                        }
                    }
                }
            }

            if let Some(pnl) = obj.get("unrealized_pnl").and_then(|p| p.as_str()) {
                if let Ok(pnl_num) = pnl.parse::<f64>() {
                    let pnl_usdc = pnl_num / 1_000_000.0;
                    let emoji = if pnl_usdc >= 0.0 { "💹" } else { "📉" };
                    tracing::info!("  {} Unrealized PnL: ${:.2}", emoji, pnl_usdc);
                }
            }
        }
        tracing::info!("");
    };

    tracing::info!("╔═══════════════════════════════════════════════════╗");
    tracing::info!("║   WebSocket Market Monitor Started               ║");
    tracing::info!("╚═══════════════════════════════════════════════════╝\n");

    tracing::info!("Monitoring:");
    tracing::info!("  ✓ Order book for market 0");
    tracing::info!("  ✓ Account {}", account_index);
    tracing::info!("  ✓ Price movements and spreads");
    tracing::info!("  ✓ Trading opportunities\n");

    tracing::info!("Trading Logic:");
    tracing::info!("  • Track mid price changes");
    tracing::info!("  • Trade on >0.1% price moves");
    tracing::info!("  • Demo limit: 2 trades max\n");

    tracing::info!("Press Ctrl+C to stop");
    tracing::info!("{}\n", "═".repeat(50));

    // Run WebSocket client
    match ws_client.run(on_order_book_update, on_account_update).await {
        Ok(_) => tracing::info!("\n✓ WebSocket closed normally"),
        Err(e) => {
            etracing::info!("\n✗ WebSocket error: {}", e);
            etracing::info!("\nTroubleshooting:");
            etracing::info!("  1. Check your internet connection");
            etracing::info!("  2. Verify API URL in .env: {}", api_url);
            etracing::info!("  3. Ensure WebSocket endpoint is accessible");
            etracing::info!("  4. Try: wss://{}/stream", ws_host);
        }
    }

    tracing::info!("\n═══ Session Summary ═══");
    tracing::info!(
        "  Order Book Updates: {}",
        update_count.load(Ordering::Relaxed)
    );
    tracing::info!("  Trades Placed: {}", trade_count.load(Ordering::Relaxed));

    Ok(())
}
