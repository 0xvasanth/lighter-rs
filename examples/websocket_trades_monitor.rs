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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    println!("╔═══════════════════════════════════════════════════╗");
    println!("║   Lighter RS - WebSocket Trades Monitor          ║");
    println!("╚═══════════════════════════════════════════════════╝\n");

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

    println!("Configuration:");
    println!("  API: {}", api_url);
    println!("  WebSocket: wss://{}/stream", ws_host);
    println!("  Account: {}", account_index);
    println!("  Chain ID: {}\n", chain_id);

    // Create trading client
    let tx_client = Arc::new(TxClient::new(
        &api_url,
        &api_key,
        account_index,
        0,
        chain_id,
    )?);

    println!("✓ Trading client initialized\n");

    // Create WebSocket client
    println!("Creating WebSocket connection...");
    println!("  Host: {}", ws_host);
    println!("  Path: /stream");
    println!("  Subscriptions: market 0, account {}\n", account_index);

    let ws_client = WsClient::builder()
        .host(&ws_host)
        .path("/stream")
        .order_books(vec![0]) // Monitor market 0
        .accounts(vec![account_index])
        .build()?;

    println!("✓ WebSocket client created\n");

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

        println!("═══ Update #{} - Market {} ═══", count, market_id);

        if let (Some(best_ask), Some(best_bid)) = (order_book.asks.first(), order_book.bids.first())
        {
            if let (Ok(ask_price), Ok(bid_price)) =
                (best_ask.price.parse::<f64>(), best_bid.price.parse::<f64>())
            {
                let mid_price = (ask_price + bid_price) / 2.0;
                let spread = ask_price - bid_price;
                let spread_bps = (spread / bid_price) * 10000.0;

                println!("📊 Market Data:");
                println!("  Best Ask: ${:.4} (Size: {})", ask_price, best_ask.size);
                println!("  Best Bid: ${:.4} (Size: {})", bid_price, best_bid.size);
                println!("  Mid Price: ${:.4}", mid_price);
                println!("  Spread: ${:.4} ({:.2} bps)", spread, spread_bps);

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

                println!(
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

                        println!(
                            "  📈 Price Change: ${:.4} ({:+.2}%)",
                            price_change, price_change_pct
                        );

                        // Simple trading logic: Trade on significant price moves
                        if price_change_pct.abs() > 0.1 && trade_count.load(Ordering::Relaxed) < 2 {
                            println!(
                                "\n  🎯 TRADING SIGNAL: Price moved {:+.2}%",
                                price_change_pct
                            );

                            let is_ask = if price_change_pct > 0.0 { 1 } else { 0 }; // Sell if price up, buy if down

                            println!(
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
                                    println!("     ✓ Order signed (nonce: {})", order.nonce);

                                    match tx_client.send_transaction(&order).await {
                                        Ok(response) => {
                                            if response.code == 200 {
                                                println!("     ✓ Order submitted!");
                                                if let Some(hash) = response.tx_hash {
                                                    println!("       Tx: {}...", &hash[..16]);
                                                }
                                                trade_count.fetch_add(1, Ordering::Relaxed);
                                            } else {
                                                println!("     ✗ Rejected: {:?}", response.message);
                                            }
                                        }
                                        Err(e) => println!("     ✗ Submit error: {}", e),
                                    }
                                }
                                Err(e) => println!("     ✗ Order error: {}", e),
                            }
                        }
                    }

                    *last_mid = mid_price;
                });

                println!();
            }
        }
    };

    // Account callback
    let on_account_update = move |account_id: String, account_data: Value| {
        println!("👤 Account {} Update", account_id);

        if let Some(obj) = account_data.as_object() {
            if let Some(balance) = obj.get("usdc_balance").and_then(|b| b.as_str()) {
                if let Ok(balance_num) = balance.parse::<f64>() {
                    println!("  💵 Balance: ${:.2} USDC", balance_num / 1_000_000.0);
                }
            }

            if let Some(orders) = obj.get("orders").and_then(|o| o.as_array()) {
                println!("  📋 Active Orders: {}", orders.len());
            }

            if let Some(positions) = obj.get("positions").and_then(|p| p.as_array()) {
                if !positions.is_empty() {
                    println!("  📊 Positions: {}", positions.len());
                    for (i, pos) in positions.iter().take(3).enumerate() {
                        if let Some(pos_obj) = pos.as_object() {
                            let size = pos_obj.get("size").and_then(|s| s.as_str()).unwrap_or("0");
                            let market = pos_obj
                                .get("market_index")
                                .and_then(|m| m.as_i64())
                                .unwrap_or(0);
                            println!("    {}. Market {}: Size {}", i + 1, market, size);
                        }
                    }
                }
            }

            if let Some(pnl) = obj.get("unrealized_pnl").and_then(|p| p.as_str()) {
                if let Ok(pnl_num) = pnl.parse::<f64>() {
                    let pnl_usdc = pnl_num / 1_000_000.0;
                    let emoji = if pnl_usdc >= 0.0 { "💹" } else { "📉" };
                    println!("  {} Unrealized PnL: ${:.2}", emoji, pnl_usdc);
                }
            }
        }
        println!();
    };

    println!("╔═══════════════════════════════════════════════════╗");
    println!("║   WebSocket Market Monitor Started               ║");
    println!("╚═══════════════════════════════════════════════════╝\n");

    println!("Monitoring:");
    println!("  ✓ Order book for market 0");
    println!("  ✓ Account {}", account_index);
    println!("  ✓ Price movements and spreads");
    println!("  ✓ Trading opportunities\n");

    println!("Trading Logic:");
    println!("  • Track mid price changes");
    println!("  • Trade on >0.1% price moves");
    println!("  • Demo limit: 2 trades max\n");

    println!("Press Ctrl+C to stop");
    println!("{}\n", "═".repeat(50));

    // Run WebSocket client
    match ws_client.run(on_order_book_update, on_account_update).await {
        Ok(_) => println!("\n✓ WebSocket closed normally"),
        Err(e) => {
            eprintln!("\n✗ WebSocket error: {}", e);
            eprintln!("\nTroubleshooting:");
            eprintln!("  1. Check your internet connection");
            eprintln!("  2. Verify API URL in .env: {}", api_url);
            eprintln!("  3. Ensure WebSocket endpoint is accessible");
            eprintln!("  4. Try: wss://{}/stream", ws_host);
        }
    }

    println!("\n═══ Session Summary ═══");
    println!(
        "  Order Book Updates: {}",
        update_count.load(Ordering::Relaxed)
    );
    println!("  Trades Placed: {}", trade_count.load(Ordering::Relaxed));

    Ok(())
}
