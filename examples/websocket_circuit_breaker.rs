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
use tracing;

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
        tracing::info!("  ✓ Circuit Breaker: SUCCESS - Reset to CLOSED state");
    }

    async fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        *self.last_failure_time.write().await = Some(Instant::now());

        tracing::info!("  ✗ Circuit Breaker: FAILURE {}/{}", failures, MAX_FAILURES);

        if failures >= MAX_FAILURES {
            self.state.store(CIRCUIT_OPEN, Ordering::Relaxed);
            tracing::info!("  🔴 Circuit Breaker: OPENED (too many failures)");
            tracing::info!("     Will retry in {:?}", CIRCUIT_TIMEOUT);
        }
    }

    async fn check_and_update(&self) {
        if self.state.load(Ordering::Relaxed) == CIRCUIT_OPEN {
            if let Some(last_failure) = *self.last_failure_time.read().await {
                if last_failure.elapsed() > CIRCUIT_TIMEOUT {
                    self.state.store(CIRCUIT_HALF_OPEN, Ordering::Relaxed);
                    tracing::info!("  🟡 Circuit Breaker: HALF_OPEN (testing recovery)");
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
    tracing_subscriber::fmt::init();
    // Load .env file
    dotenv().ok();

    tracing::info!("╔═══════════════════════════════════════════════════╗");
    tracing::info!("║   Lighter RS - Circuit Breaker Trading Bot       ║");
    tracing::info!("║   Educational Example - Use at Your Own Risk!    ║");
    tracing::info!("╚═══════════════════════════════════════════════════╝\n");

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

    tracing::info!("✓ Configuration loaded from .env");
    tracing::info!("  API URL: {}", api_url);
    tracing::info!("  WebSocket: wss://{}/stream", ws_host);
    tracing::info!("  Account: {}", account_index);
    tracing::info!("  Chain ID: {}", chain_id);
    tracing::info!();

    // Create trading client
    let tx_client = Arc::new(TxClient::new(
        &api_url,
        &api_key,
        account_index,
        api_key_index,
        chain_id,
    )?);

    tracing::info!("✓ Trading client initialized");

    // Create circuit breaker
    let circuit_breaker = Arc::new(CircuitBreaker::new());

    tracing::info!("✓ Circuit breaker initialized");
    tracing::info!("  Max failures: {}", MAX_FAILURES);
    tracing::info!("  Timeout: {:?}", CIRCUIT_TIMEOUT);
    tracing::info!("  Min spread: {} bps\n", MIN_SPREAD_BPS);

    // Create WebSocket client
    let ws_client = WsClient::builder()
        .host(&ws_host)
        .order_books(vec![0]) // Monitor market 0
        .accounts(vec![account_index])
        .build()?;

    tracing::info!("✓ WebSocket client created");
    tracing::info!("  Monitoring market: 0");
    tracing::info!("  Monitoring account: {}\n", account_index);

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
            tracing::info!("📊 Market {} | Circuit: {}", market_id, state);

            if let (Some(best_ask), Some(best_bid)) =
                (order_book.asks.first(), order_book.bids.first())
            {
                if let (Ok(ask_price), Ok(bid_price)) =
                    (best_ask.price.parse::<f64>(), best_bid.price.parse::<f64>())
                {
                    let spread = ask_price - bid_price;
                    let spread_bps = (spread / bid_price) * 10000.0;
                    let mid_price = (ask_price + bid_price) / 2.0;

                    tracing::info!(
                        "  Ask: {:.2} | Bid: {:.2} | Mid: {:.2}",
                        ask_price, bid_price, mid_price
                    );
                    tracing::info!("  Spread: {:.4} ({:.2} bps)", spread, spread_bps);

                    // Trading logic: Only trade if circuit is closed or half-open
                    if (cb.is_closed() || cb.is_half_open()) && spread_bps >= MIN_SPREAD_BPS {
                        let count = order_count.load(Ordering::Relaxed);

                        // Limit total orders for demo
                        if count < 3 {
                            tracing::info!(
                                "\n  🎯 TRADING SIGNAL: Spread {:.2} bps >= {:.2} bps",
                                spread_bps, MIN_SPREAD_BPS
                            );
                            tracing::info!("     Placing order #{}", count + 1);

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
                                    tracing::info!("     ✓ Order signed (nonce: {})", order.nonce);

                                    // Submit to API
                                    match tx_client.send_transaction(&order).await {
                                        Ok(response) => {
                                            if response.code == 200 {
                                                tracing::info!("     ✓ Order submitted successfully!");
                                                if let Some(hash) = response.tx_hash {
                                                    tracing::info!("       Tx: {}", hash);
                                                }
                                                cb.record_success().await;
                                                order_count.fetch_add(1, Ordering::Relaxed);
                                            } else {
                                                tracing::info!(
                                                    "     ✗ Order rejected: {:?}",
                                                    response.message
                                                );
                                                cb.record_failure().await;
                                            }
                                        }
                                        Err(e) => {
                                            tracing::info!("     ✗ Submit failed: {}", e);
                                            cb.record_failure().await;
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::info!("     ✗ Order creation failed: {}", e);
                                    cb.record_failure().await;
                                }
                            }
                        } else {
                            tracing::info!("  ⚠ Demo limit reached (3 orders max)");
                        }
                    } else if !cb.is_closed() && !cb.is_half_open() {
                        tracing::info!("  ⛔ Circuit breaker is OPEN - not trading");
                    }
                }
            }
            tracing::info!();
        });
    };

    // Account callback - Monitor our state
    let on_account_update = move |account_id: String, account_data: Value| {
        tracing::info!("👤 Account {} Updated", account_id);

        if let Some(obj) = account_data.as_object() {
            if let Some(balance) = obj.get("usdc_balance") {
                tracing::info!("  💵 Balance: {} USDC", balance);
            }

            if let Some(orders) = obj.get("orders").and_then(|o| o.as_array()) {
                tracing::info!("  📋 Active Orders: {}", orders.len());

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
                        tracing::info!("    {}. {} {} @ {}", i + 1, side, size, price);
                    }
                }
            }

            if let Some(positions) = obj.get("positions").and_then(|p| p.as_array()) {
                if !positions.is_empty() {
                    tracing::info!("  📊 Positions: {}", positions.len());
                }
            }

            if let Some(pnl) = obj.get("unrealized_pnl") {
                tracing::info!("  💹 Unrealized PnL: {}", pnl);
            }
        }
        tracing::info!();
    };

    tracing::info!("╔═══════════════════════════════════════════════════╗");
    tracing::info!("║   Trading Bot Started with Circuit Breaker       ║");
    tracing::info!("╚═══════════════════════════════════════════════════╝\n");

    tracing::info!("Strategy:");
    tracing::info!("  • Monitor market 0 order book");
    tracing::info!("  • Place orders when spread >= {} bps", MIN_SPREAD_BPS);
    tracing::info!("  • Circuit breaker protects against failures");
    tracing::info!("  • Demo mode: Max 3 orders\n");

    tracing::info!("Safety Features:");
    tracing::info!("  ✓ Circuit breaker pattern");
    tracing::info!("  ✓ Order count limits");
    tracing::info!("  ✓ Spread threshold");
    tracing::info!("  ✓ Error handling\n");

    tracing::info!("Press Ctrl+C to stop");
    tracing::info!("{}\n", "═".repeat(50));

    // Run the WebSocket client
    match ws_client.run(on_order_book_update, on_account_update).await {
        Ok(_) => tracing::info!("\n✓ WebSocket connection closed normally"),
        Err(e) => tracing::warn!("\n✗ WebSocket error: {}", e),
    }

    tracing::info!("\n╔═══════════════════════════════════════════════════╗");
    tracing::info!("║   Trading Bot Stopped                             ║");
    tracing::info!("╚═══════════════════════════════════════════════════╝");
    tracing::info!("\nOrders placed: {}", order_count.load(Ordering::Relaxed));
    tracing::info!("Circuit state: {}", circuit_breaker.state_name());

    Ok(())
}
