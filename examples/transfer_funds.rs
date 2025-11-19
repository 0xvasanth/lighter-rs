//! Example: Transferring funds between accounts
//!
//! This example demonstrates how to transfer funds using the Lighter SDK.
//!
//! Run with: cargo run --example transfer_funds

use lighter_rs::client::TxClient;
use lighter_rs::types::{TransactOpts, TransferTxReq, TxInfo};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("=== Lighter RS: Transfer Funds Example ===\n");

    // Initialize the transaction client
    let tx_client = TxClient::new(
        "", // Offline mode
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        12345, // From account index
        0,     // API key index
        1,     // Chain ID
    )?;

    tracing::info!("✓ Transaction client initialized");
    tracing::info!("  From Account: {}", tx_client.account_index());

    // Create a transfer request
    let transfer_req = TransferTxReq {
        to_account_index: 54321,
        usdc_amount: 1000000, // 1 USDC (6 decimals)
        fee: 1000,            // 0.001 USDC
        memo: [0u8; 32],
    };

    tracing::info!("\nTransfer Details:");
    tracing::info!("  To Account: {}", transfer_req.to_account_index);
    tracing::info!(
        "  Amount: {} USDC",
        transfer_req.usdc_amount as f64 / 1_000_000.0
    );
    tracing::info!("  Fee: {} USDC", transfer_req.fee as f64 / 1_000_000.0);

    // Create transaction options
    let opts = TransactOpts {
        from_account_index: Some(tx_client.account_index()),
        api_key_index: Some(tx_client.api_key_index()),
        expired_at: 1000000000,
        nonce: Some(1),
        dry_run: false,
    };

    // Sign the transaction
    let tx = tx_client.transfer(&transfer_req, Some(opts)).await?;

    tracing::info!("\n✓ Transfer transaction signed successfully");
    tracing::info!("  Transaction Type: {}", tx.get_tx_type());
    tracing::info!("  From: {}", tx.from_account_index);
    tracing::info!("  To: {}", tx.to_account_index);
    tracing::info!("  Amount: {}", tx.usdc_amount);
    tracing::info!("  Nonce: {}", tx.nonce);
    if let Some(hash) = tx.get_tx_hash() {
        tracing::info!("  Transaction Hash: {}", hash);
    }

    tracing::info!("\n✓ Example completed successfully!");

    Ok(())
}
