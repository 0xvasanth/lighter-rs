//! Example: Pool operations (create, mint, burn shares)
//!
//! This example demonstrates pool-related operations using the Lighter SDK.
//!
//! Run with: cargo run --example pool_operations

use lighter_rs::client::TxClient;
use lighter_rs::types::{BurnSharesTxReq, CreatePublicPoolTxReq, MintSharesTxReq, TransactOpts};
use tracing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("=== Lighter RS: Pool Operations Example ===\n");

    // Initialize the transaction client
    let tx_client = TxClient::new(
        "", // Offline mode
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        12345, // Account index
        0,     // API key index
        1,     // Chain ID
    )?;

    tracing::info!("✓ Transaction client initialized\n");

    // Example 1: Create a public pool
    tracing::info!("=== Creating Public Pool ===");
    let pool_req = CreatePublicPoolTxReq {
        operator_fee: 10000,              // 1% (10000 / 1000000)
        initial_total_shares: 1000000000, // 1B shares
        min_operator_share_rate: 5000,    // 0.5% (5000 / 1000000)
    };

    let opts = TransactOpts {
        from_account_index: Some(tx_client.account_index()),
        api_key_index: Some(tx_client.api_key_index()),
        expired_at: 1000000000,
        nonce: Some(1),
        dry_run: false,
    };

    let _create_pool_tx = tx_client
        .create_public_pool(&pool_req, Some(opts.clone()))
        .await?;

    tracing::info!("✓ Pool creation transaction signed");
    tracing::info!(
        "  Operator Fee: {}%",
        pool_req.operator_fee as f64 / 10000.0
    );
    tracing::info!("  Initial Shares: {}", pool_req.initial_total_shares);
    tracing::info!(
        "  Min Operator Share Rate: {}%\n",
        pool_req.min_operator_share_rate as f64 / 10000.0
    );

    // Example 2: Mint shares
    tracing::info!("=== Minting Shares ===");
    let mint_req = MintSharesTxReq {
        public_pool_index: 123,
        share_amount: 100000,
    };

    let mut opts2 = opts.clone();
    opts2.nonce = Some(2);

    let _mint_tx = tx_client.mint_shares(&mint_req, Some(opts2)).await?;

    tracing::info!("✓ Mint shares transaction signed");
    tracing::info!("  Pool Index: {}", mint_req.public_pool_index);
    tracing::info!("  Share Amount: {}\n", mint_req.share_amount);

    // Example 3: Burn shares
    tracing::info!("=== Burning Shares ===");
    let burn_req = BurnSharesTxReq {
        public_pool_index: 123,
        share_amount: 50000,
    };

    let mut opts3 = opts;
    opts3.nonce = Some(3);

    let _burn_tx = tx_client.burn_shares(&burn_req, Some(opts3)).await?;

    tracing::info!("✓ Burn shares transaction signed");
    tracing::info!("  Pool Index: {}", burn_req.public_pool_index);
    tracing::info!("  Share Amount: {}", burn_req.share_amount);

    tracing::info!("\n✓ All pool operations completed successfully!");

    Ok(())
}
