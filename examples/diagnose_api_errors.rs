/// Diagnostic tool to understand and fix API errors
///
/// This example helps diagnose:
/// 1. "invalid base amount" - Check market specifications
/// 2. "api key not found" - Verify API key registration
///
/// Solutions provided based on error analysis
use dotenv::dotenv;
use lighter_rs::client::TxClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    tracing::info!("=== Lighter API Error Diagnostic Tool ===\n");

    // Load configuration
    let private_key = env::var("LIGHTER_API_KEY")?;
    let account_index: i64 = env::var("LIGHTER_ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("LIGHTER_API_KEY_INDEX")?.parse()?;
    let chain_id: u32 = env::var("LIGHTER_CHAIN_ID")
        .unwrap_or_else(|_| "304".to_string())
        .parse()?;
    let api_url = env::var("LIGHTER_API_URL")?;

    tracing::info!("Configuration Check:");
    tracing::info!("  API URL: {}", api_url);
    tracing::info!("  Account Index: {}", account_index);
    tracing::info!("  API Key Index: {}", api_key_index);
    tracing::info!("  Chain ID: {}", chain_id);
    tracing::info!("  API Key Length: {} chars", private_key.len());
    tracing::info!("");

    // Initialize client
    let tx_client = match TxClient::new(
        &api_url,
        &private_key,
        account_index,
        api_key_index,
        chain_id,
    ) {
        Ok(client) => {
            tracing::info!("✅ Client initialized successfully");
            client
        }
        Err(e) => {
            tracing::info!("❌ Client initialization failed: {}", e);
            tracing::info!("\n🔧 Solution:");
            tracing::info!("   - Verify your API key format (should be 40-byte hex string)");
            tracing::info!("   - Check that the key doesn't have '0x' prefix");
            return Err(e.into());
        }
    };
    tracing::info!("");

    // Error 1: Check "invalid base amount"
    tracing::info!("=== Error 1: 'invalid base amount' ===\n");
    tracing::info!("This error typically means:");
    tracing::info!("  1. Base amount is below minimum order size for the market");
    tracing::info!("  2. Base amount doesn't match the market's step size");
    tracing::info!("  3. Decimals are incorrect for the market");
    tracing::info!("");

    tracing::info!("Testing different base amounts for market 0 (ETH)...\n");

    // Test various amounts
    let test_amounts = vec![
        (100_000i64, "$0.10", "Too small - likely below minimum"),
        (1_000_000, "$1.00", "Small - may be below minimum"),
        (
            10_000_000,
            "$10.00",
            "Medium - should work if above minimum",
        ),
        (100_000_000, "$100.00", "Large - should definitely work"),
    ];

    for (amount, description, note) in test_amounts {
        tracing::info!("  Testing base_amount = {} ({})", amount, description);
        tracing::info!("    {}", note);

        match tx_client
            .create_limit_order(
                0, // ETH market
                chrono::Utc::now().timestamp_millis(),
                amount,
                3_000_000_000, // $3000 price
                0,
                false,
                None,
            )
            .await
        {
            Ok(order) => match tx_client.send_transaction(&order).await {
                Ok(response) => {
                    if response.code == 200 {
                        tracing::info!("    ✅ SUCCESS! This amount works!");
                        tracing::info!(
                            "    Minimum working amount: {} ({})\\n",
                            amount,
                            description
                        );
                        break;
                    } else {
                        tracing::info!("    ❌ Error: {:?}", response.message);
                    }
                }
                Err(e) => {
                    tracing::info!("    ❌ Error: {}", e);
                }
            },
            Err(e) => {
                tracing::info!("    ❌ Order creation failed: {}", e);
            }
        }
        tracing::info!("");
    }

    tracing::info!("\n🔧 Solutions for 'invalid base amount':");
    tracing::info!("   1. Check the market's minimum order size (may vary by market)");
    tracing::info!("   2. For ETH (market 0): Try amounts >= $10 (10_000_000 with 6 decimals)");
    tracing::info!("   3. Verify you're using correct decimals (usually 6 for base_amount)");
    tracing::info!("   4. Check market specifications: amount_step, min_order_size");
    tracing::info!("");

    // Error 2: Check "api key not found"
    tracing::info!("\n=== Error 2: 'api key not found' ===\n");
    tracing::info!("This error means:");
    tracing::info!("  - The API key is not registered on Lighter");
    tracing::info!("  - The account_index or api_key_index doesn't match");
    tracing::info!("  - The API key format is incorrect");
    tracing::info!("");

    tracing::info!("🔧 Solutions for 'api key not found':");
    tracing::info!("   1. Verify your API key is registered at https://app.lighter.xyz");
    tracing::info!("   2. Check account_index matches your account");
    tracing::info!("   3. Verify api_key_index (usually 0, 1, or 2)");
    tracing::info!("   4. Ensure API key is 40-byte hex without '0x' prefix");
    tracing::info!("   5. Try regenerating your API key if unsure");
    tracing::info!("");

    tracing::info!("\n=== Additional Debugging ===\n");
    tracing::info!("Your current configuration:");
    tracing::info!("  LIGHTER_API_KEY length: {}", private_key.len());
    tracing::info!("  LIGHTER_ACCOUNT_INDEX: {}", account_index);
    tracing::info!("  LIGHTER_API_KEY_INDEX: {}", api_key_index);
    tracing::info!("");

    if private_key.starts_with("0x") || private_key.starts_with("0X") {
        tracing::info!("⚠️  WARNING: Your API key starts with '0x'");
        tracing::info!("   Remove the '0x' prefix from your API key in .env file");
        tracing::info!("");
    }

    if private_key.len() != 80 && private_key.len() != 64 {
        tracing::info!(
            "⚠️  WARNING: Unusual API key length ({} chars)",
            private_key.len()
        );
        tracing::info!("   Expected: 80 chars (40 bytes hex) or 64 chars (32 bytes hex)");
        tracing::info!("");
    }

    tracing::info!("\n=== Next Steps ===\n");
    tracing::info!("1. Verify your credentials at https://app.lighter.xyz");
    tracing::info!("2. Check your account has sufficient balance");
    tracing::info!("3. Try increasing the base_amount to meet minimum requirements");
    tracing::info!("4. Ensure your API key is correctly registered");
    tracing::info!("");
    tracing::info!("Need more help? Check the Lighter documentation:");
    tracing::info!("  - API Docs: https://apidocs.lighter.xyz");
    tracing::info!("  - Discord: https://discord.gg/lighter");

    Ok(())
}
