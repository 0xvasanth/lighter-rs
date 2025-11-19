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
    dotenv().ok();

    println!("=== Lighter API Error Diagnostic Tool ===\n");

    // Load configuration
    let private_key = env::var("LIGHTER_API_KEY")?;
    let account_index: i64 = env::var("LIGHTER_ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("LIGHTER_API_KEY_INDEX")?.parse()?;
    let chain_id: u32 = env::var("LIGHTER_CHAIN_ID").unwrap_or_else(|_| "304".to_string()).parse()?;
    let api_url = env::var("LIGHTER_API_URL")?;

    println!("Configuration Check:");
    println!("  API URL: {}", api_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!("  Chain ID: {}", chain_id);
    println!("  API Key Length: {} chars", private_key.len());
    println!();

    // Initialize client
    let tx_client = match TxClient::new(&api_url, &private_key, account_index, api_key_index, chain_id) {
        Ok(client) => {
            println!("✅ Client initialized successfully");
            client
        }
        Err(e) => {
            println!("❌ Client initialization failed: {}", e);
            println!("\n🔧 Solution:");
            println!("   - Verify your API key format (should be 40-byte hex string)");
            println!("   - Check that the key doesn't have '0x' prefix");
            return Err(e.into());
        }
    };
    println!();

    // Error 1: Check "invalid base amount"
    println!("=== Error 1: 'invalid base amount' ===\n");
    println!("This error typically means:");
    println!("  1. Base amount is below minimum order size for the market");
    println!("  2. Base amount doesn't match the market's step size");
    println!("  3. Decimals are incorrect for the market");
    println!();

    println!("Testing different base amounts for market 0 (ETH)...\n");

    // Test various amounts
    let test_amounts = vec![
        (100_000i64, "$0.10", "Too small - likely below minimum"),
        (1_000_000, "$1.00", "Small - may be below minimum"),
        (10_000_000, "$10.00", "Medium - should work if above minimum"),
        (100_000_000, "$100.00", "Large - should definitely work"),
    ];

    for (amount, description, note) in test_amounts {
        println!("  Testing base_amount = {} ({})", amount, description);
        println!("    {}", note);

        match tx_client.create_limit_order(
            0, // ETH market
            chrono::Utc::now().timestamp_millis(),
            amount,
            3_000_000_000, // $3000 price
            0,
            false,
            None,
        ).await {
            Ok(order) => {
                match tx_client.send_transaction(&order).await {
                    Ok(response) => {
                        if response.code == 200 {
                            println!("    ✅ SUCCESS! This amount works!");
                            println!("    Minimum working amount: {} ({})\\n", amount, description);
                            break;
                        } else {
                            println!("    ❌ Error: {:?}", response.message);
                        }
                    }
                    Err(e) => {
                        println!("    ❌ Error: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("    ❌ Order creation failed: {}", e);
            }
        }
        println!();
    }

    println!("\n🔧 Solutions for 'invalid base amount':");
    println!("   1. Check the market's minimum order size (may vary by market)");
    println!("   2. For ETH (market 0): Try amounts >= $10 (10_000_000 with 6 decimals)");
    println!("   3. Verify you're using correct decimals (usually 6 for base_amount)");
    println!("   4. Check market specifications: amount_step, min_order_size");
    println!();

    // Error 2: Check "api key not found"
    println!("\n=== Error 2: 'api key not found' ===\n");
    println!("This error means:");
    println!("  - The API key is not registered on Lighter");
    println!("  - The account_index or api_key_index doesn't match");
    println!("  - The API key format is incorrect");
    println!();

    println!("🔧 Solutions for 'api key not found':");
    println!("   1. Verify your API key is registered at https://app.lighter.xyz");
    println!("   2. Check account_index matches your account");
    println!("   3. Verify api_key_index (usually 0, 1, or 2)");
    println!("   4. Ensure API key is 40-byte hex without '0x' prefix");
    println!("   5. Try regenerating your API key if unsure");
    println!();

    println!("\n=== Additional Debugging ===\n");
    println!("Your current configuration:");
    println!("  LIGHTER_API_KEY length: {}", private_key.len());
    println!("  LIGHTER_ACCOUNT_INDEX: {}", account_index);
    println!("  LIGHTER_API_KEY_INDEX: {}", api_key_index);
    println!();

    if private_key.starts_with("0x") || private_key.starts_with("0X") {
        println!("⚠️  WARNING: Your API key starts with '0x'");
        println!("   Remove the '0x' prefix from your API key in .env file");
        println!();
    }

    if private_key.len() != 80 && private_key.len() != 64 {
        println!("⚠️  WARNING: Unusual API key length ({} chars)", private_key.len());
        println!("   Expected: 80 chars (40 bytes hex) or 64 chars (32 bytes hex)");
        println!();
    }

    println!("\n=== Next Steps ===\n");
    println!("1. Verify your credentials at https://app.lighter.xyz");
    println!("2. Check your account has sufficient balance");
    println!("3. Try increasing the base_amount to meet minimum requirements");
    println!("4. Ensure your API key is correctly registered");
    println!();
    println!("Need more help? Check the Lighter documentation:");
    println!("  - API Docs: https://apidocs.lighter.xyz");
    println!("  - Discord: https://discord.gg/lighter");

    Ok(())
}
