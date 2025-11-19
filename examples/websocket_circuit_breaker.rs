//! Example: WebSocket Trading with Circuit Breaker Pattern
//!
//! This example demonstrates a production-ready trading setup with:
//! 1. Environment variable configuration (.env file support)
//! 2. WebSocket monitoring for real-time data
//! 3. Circuit breaker pattern for risk management
//! 4. Automatic order placement based on market conditions
//! 5. Safety mechanisms and error handling
//!
//! Circuit Breaker States:
//! - CLOSED: Normal operation, orders can be placed
//! - OPEN: Too many failures, stop trading temporarily
//! - HALF_OPEN: Testing if system recovered
//!
//! Setup:
//! 1. Copy .env.example to .env
//! 2. Fill in your credentials in .env
//! 3. Run: cargo run --example websocket_circuit_breaker

use dotenv::dotenv;
use lighter_rs::client::TxClient;
use lighter_rs::ws_client::{OrderBook, WsClient};
use serde_json::Value;
use std::env;
use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// Circuit breaker states
const CIRCUIT_CLOSED: u8 = 0; // Normal operation
const CIRCUIT_OPEN: u8 = 1; // Too many failures, stop trading
const CIRCUIT_HALF_OPEN: u8 = 2; // Testing recovery

// Circuit breaker configuration
const MAX_FAILURES: u32 = 3; // Open circuit after 3 failures
const CIRCUIT_TIMEOUT: Duration = Duration::from_secs(60); // Wait 60s before half-open
const MIN_SPREAD_BPS: f64 = 5.0; // Minimum spread to trade (5 basis points)

#[derive(Clone)]
struct CircuitBreaker {
    state: Arc<AtomicU8>,
    failure_count: Arc<AtomicU32>,
    last_failure_time: Arc<tokio::sync::RwLock<Option<Instant>>>,
}

impl CircuitBreaker {
    fn new() -> Self {
        Self {
            state: Arc::new(AtomicU8::new(CIRCUIT_CLOSED)),
            failure_count: Arc::new(AtomicU32::new(0)),
            last_failure_time: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    fn is_closed(&self) -> bool {
        self.state.load(Ordering::Relaxed) == CIRCUIT_CLOSED
    }

    fn is_half_open(&self) -> bool {
        self.state.load(Ordering::Relaxed) == CIRCUIT_HALF_OPEN
    }

    async fn record_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        self.state.store(CIRCUIT_CLOSED, Ordering::Relaxed);
        println!("  ✓ Circuit Breaker: SUCCESS - Reset to CLOSED state");
    }

    async fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        *self.last_failure_time.write().await = Some(Instant::now());

        println!("  ✗ Circuit Breaker: FAILURE {}/{}", failures, MAX_FAILURES);

        if failures >= MAX_FAILURES {
            self.state.store(CIRCUIT_OPEN, Ordering::Relaxed);
            println!("  🔴 Circuit Breaker: OPENED (too many failures)");
            println!("     Will retry in {:?}", CIRCUIT_TIMEOUT);
        }
    }

    async fn check_and_update(&self) {
        if self.state.load(Ordering::Relaxed) == CIRCUIT_OPEN {
            if let Some(last_failure) = *self.last_failure_time.read().await {
                if last_failure.elapsed() > CIRCUIT_TIMEOUT {
                    self.state.store(CIRCUIT_HALF_OPEN, Ordering::Relaxed);
                    println!("  🟡 Circuit Breaker: HALF_OPEN (testing recovery)");
                }
            }
        }
    }

    fn state_name(&self) -> &str {
        match self.state.load(Ordering::Relaxed) {
            CIRCUIT_CLOSED => "CLOSED",
            CIRCUIT_OPEN => "OPEN",
            CIRCUIT_HALF_OPEN => "HALF_OPEN",
            _ => "UNKNOWN",
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenv().ok();

    println!("╔═══════════════════════════════════════════════════╗");
    println!("║   Lighter RS - Circuit Breaker Trading Bot       ║");
    println!("║   Educational Example - Use at Your Own Risk!    ║");
    println!("╚═══════════════════════════════════════════════════╝\n");

    // Load configuration from environment
    let api_key =
        env::var("LIGHTER_API_KEY").expect("LIGHTER_API_KEY not found. Did you create .env file?");

    let account_index: i64 = env::var("LIGHTER_ACCOUNT_INDEX")
        .expect("LIGHTER_ACCOUNT_INDEX not set")
        .parse()
        .expect("LIGHTER_ACCOUNT_INDEX must be a number");

    let api_key_index: u8 = env::var("LIGHTER_API_KEY_INDEX")
        .unwrap_or_else(|_| "0".to_string())
        .parse()
        .unwrap_or(0);

    let api_url = env::var("LIGHTER_API_URL")
        .unwrap_or_else(|_| "https://api-testnet.lighter.xyz".to_string());

    let chain_id: u32 = env::var("LIGHTER_CHAIN_ID")
        .unwrap_or_else(|_| "300".to_string())
        .parse()
        .unwrap_or(300);

    let ws_host =
        env::var("LIGHTER_WS_HOST").unwrap_or_else(|_| "api-testnet.lighter.xyz".to_string());

    println!("✓ Configuration loaded from .env");
    println!("  API URL: {}", api_url);
    println!("  WebSocket: wss://{}/stream", ws_host);
    println!("  Account: {}", account_index);
    println!("  Chain ID: {}", chain_id);
    println!();

    // Create trading client
    let tx_client = Arc::new(TxClient::new(
        &api_url,
        &api_key,
        account_index,
        api_key_index,
        chain_id,
    )?);

    println!("✓ Trading client initialized");

    // Create circuit breaker
    let circuit_breaker = Arc::new(CircuitBreaker::new());

    println!("✓ Circuit breaker initialized");
    println!("  Max failures: {}", MAX_FAILURES);
    println!("  Timeout: {:?}", CIRCUIT_TIMEOUT);
    println!("  Min spread: {} bps\n", MIN_SPREAD_BPS);

    // Create WebSocket client
    let ws_client = WsClient::builder()
        .host(&ws_host)
        .order_books(vec![0]) // Monitor market 0
        .accounts(vec![account_index])
        .build()?;

    println!("✓ WebSocket client created");
    println!("  Monitoring market: 0");
    println!("  Monitoring account: {}\n", account_index);

    // Order counter
    let order_count = Arc::new(AtomicU32::new(0));

    // Clone for callbacks
    let tx_client_clone = tx_client.clone();
    let circuit_breaker_clone = circuit_breaker.clone();
    let order_count_clone = order_count.clone();

    // Order book callback with trading logic
    let on_order_book_update = move |market_id: String, order_book: OrderBook| {
        let market_id_num: u8 = market_id.parse().unwrap_or(0);

        // Check circuit breaker
        let cb = circuit_breaker_clone.clone();
        let tx_client = tx_client_clone.clone();
        let order_count = order_count_clone.clone();

        tokio::spawn(async move {
            // Update circuit breaker state
            cb.check_and_update().await;

            let state = cb.state_name();
            println!("📊 Market {} | Circuit: {}", market_id, state);

            if let (Some(best_ask), Some(best_bid)) =
                (order_book.asks.first(), order_book.bids.first())
            {
                if let (Ok(ask_price), Ok(bid_price)) =
                    (best_ask.price.parse::<f64>(), best_bid.price.parse::<f64>())
                {
                    let spread = ask_price - bid_price;
                    let spread_bps = (spread / bid_price) * 10000.0;
                    let mid_price = (ask_price + bid_price) / 2.0;

                    println!(
                        "  Ask: {:.2} | Bid: {:.2} | Mid: {:.2}",
                        ask_price, bid_price, mid_price
                    );
                    println!("  Spread: {:.4} ({:.2} bps)", spread, spread_bps);

                    // Trading logic: Only trade if circuit is closed or half-open
                    if (cb.is_closed() || cb.is_half_open()) && spread_bps >= MIN_SPREAD_BPS {
                        let count = order_count.load(Ordering::Relaxed);

                        // Limit total orders for demo
                        if count < 3 {
                            println!(
                                "\n  🎯 TRADING SIGNAL: Spread {:.2} bps >= {:.2} bps",
                                spread_bps, MIN_SPREAD_BPS
                            );
                            println!("     Placing order #{}", count + 1);

                            // Place a small market buy order
                            let result = tx_client
                                .create_market_order(
                                    market_id_num,
                                    chrono::Utc::now().timestamp_millis(),
                                    100_000,                   // Small size for demo
                                    (mid_price * 1.01) as u32, // 1% slippage tolerance
                                    0,                         // BUY
                                    false,
                                    None,
                                )
                                .await;

                            match result {
                                Ok(order) => {
                                    println!("     ✓ Order signed (nonce: {})", order.nonce);

                                    // Submit to API
                                    match tx_client.send_transaction(&order).await {
                                        Ok(response) => {
                                            if response.code == 200 {
                                                println!("     ✓ Order submitted successfully!");
                                                if let Some(hash) = response.tx_hash {
                                                    println!("       Tx: {}", hash);
                                                }
                                                cb.record_success().await;
                                                order_count.fetch_add(1, Ordering::Relaxed);
                                            } else {
                                                println!(
                                                    "     ✗ Order rejected: {:?}",
                                                    response.message
                                                );
                                                cb.record_failure().await;
                                            }
                                        }
                                        Err(e) => {
                                            println!("     ✗ Submit failed: {}", e);
                                            cb.record_failure().await;
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("     ✗ Order creation failed: {}", e);
                                    cb.record_failure().await;
                                }
                            }
                        } else {
                            println!("  ⚠ Demo limit reached (3 orders max)");
                        }
                    } else if !cb.is_closed() && !cb.is_half_open() {
                        println!("  ⛔ Circuit breaker is OPEN - not trading");
                    }
                }
            }
            println!();
        });
    };

    // Account callback - Monitor our state
    let on_account_update = move |account_id: String, account_data: Value| {
        println!("👤 Account {} Updated", account_id);

        if let Some(obj) = account_data.as_object() {
            if let Some(balance) = obj.get("usdc_balance") {
                println!("  💵 Balance: {} USDC", balance);
            }

            if let Some(orders) = obj.get("orders").and_then(|o| o.as_array()) {
                println!("  📋 Active Orders: {}", orders.len());

                for (i, order) in orders.iter().take(3).enumerate() {
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
                            .unwrap_or("?");
                        let size = order_obj
                            .get("size")
                            .and_then(|s| s.as_str())
                            .unwrap_or("?");
                        println!("    {}. {} {} @ {}", i + 1, side, size, price);
                    }
                }
            }

            if let Some(positions) = obj.get("positions").and_then(|p| p.as_array()) {
                if !positions.is_empty() {
                    println!("  📊 Positions: {}", positions.len());
                }
            }

            if let Some(pnl) = obj.get("unrealized_pnl") {
                println!("  💹 Unrealized PnL: {}", pnl);
            }
        }
        println!();
    };

    println!("╔═══════════════════════════════════════════════════╗");
    println!("║   Trading Bot Started with Circuit Breaker       ║");
    println!("╚═══════════════════════════════════════════════════╝\n");

    println!("Strategy:");
    println!("  • Monitor market 0 order book");
    println!("  • Place orders when spread >= {} bps", MIN_SPREAD_BPS);
    println!("  • Circuit breaker protects against failures");
    println!("  • Demo mode: Max 3 orders\n");

    println!("Safety Features:");
    println!("  ✓ Circuit breaker pattern");
    println!("  ✓ Order count limits");
    println!("  ✓ Spread threshold");
    println!("  ✓ Error handling\n");

    println!("Press Ctrl+C to stop");
    println!("{}\n", "═".repeat(50));

    // Run the WebSocket client
    match ws_client.run(on_order_book_update, on_account_update).await {
        Ok(_) => println!("\n✓ WebSocket connection closed normally"),
        Err(e) => eprintln!("\n✗ WebSocket error: {}", e),
    }

    println!("\n╔═══════════════════════════════════════════════════╗");
    println!("║   Trading Bot Stopped                             ║");
    println!("╚═══════════════════════════════════════════════════╝");
    println!("\nOrders placed: {}", order_count.load(Ordering::Relaxed));
    println!("Circuit state: {}", circuit_breaker.state_name());

    Ok(())
}
