use dotenv::dotenv;
use lighter_rs::client::TxClient;
use lighter_rs::types::TxInfo;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    // Load configuration from environment
    let private_key =
        env::var("LIGHTER_API_KEY").expect("LIGHTER_API_KEY must be set in .env file");
    let account_index: i64 = env::var("LIGHTER_ACCOUNT_INDEX")
        .expect("LIGHTER_ACCOUNT_INDEX must be set in .env file")
        .parse()
        .expect("LIGHTER_ACCOUNT_INDEX must be a valid number");
    let api_key_index: u8 = env::var("LIGHTER_API_KEY_INDEX")
        .unwrap_or_else(|_| "0".to_string())
        .parse()
        .expect("LIGHTER_API_KEY_INDEX must be a valid number");

    let api_url = env::var("LIGHTER_API_URL").expect("LIGHTER_API_URL must be set in .env file");

    // Create transaction client
    let tx_client = TxClient::new(
        &api_url,
        &private_key,
        account_index,
        api_key_index,
        304, // 304 = Mainnet, 300 = Testnet
    )?;

    let market_index = 0u8; // Market 0 = ETH
    let mid_price = 3000_00; // Price protection for market order

    tracing::info!("Creating market order...");

    // Create and submit market order
    match tx_client
        .create_market_order(
            market_index,
            chrono::Utc::now().timestamp_millis(),
            100_000, // Small size for demo
            mid_price,
            0,     // BUY (0 = buy, 1 = sell)
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
                    } else {
                        tracing::info!("  ✗ Order failed: {:?}", response.message);
                    }
                }
                Err(e) => tracing::info!("  ✗ Submit error: {}", e),
            }
        }
        Err(e) => tracing::info!("  ✗ Order creation error: {}", e),
    }

    Ok(())
}
