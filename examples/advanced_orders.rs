//! Example: Advanced order operations (cancel, modify, grouped orders)
//!
//! This example demonstrates advanced order operations using the Lighter SDK.
//!
//! Run with: cargo run --example advanced_orders

use lighter_rs::client::TxClient;
use lighter_rs::constants::*;
use lighter_rs::types::{
    CancelAllOrdersTxReq, CancelOrderTxReq, CreateGroupedOrdersTxReq, CreateOrderTxReq,
    ModifyOrderTxReq, TransactOpts,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("=== Lighter RS: Advanced Orders Example ===\n");

    // Initialize the transaction client
    let tx_client = TxClient::new(
        "", // Offline mode
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        12345,
        0,
        1,
    )?;

    tracing::info!("✓ Transaction client initialized\n");

    // Example 1: Cancel an order
    tracing::info!("=== Canceling Order ===");
    let cancel_req = CancelOrderTxReq {
        market_index: 0,
        index: 123456,
    };

    let opts = TransactOpts {
        from_account_index: Some(tx_client.account_index()),
        api_key_index: Some(tx_client.api_key_index()),
        expired_at: 1000000000,
        nonce: Some(1),
        dry_run: false,
    };

    let _cancel_tx = tx_client
        .cancel_order(&cancel_req, Some(opts.clone()))
        .await?;

    tracing::info!("✓ Cancel order transaction signed");
    tracing::info!("  Market: {}", cancel_req.market_index);
    tracing::info!("  Order Index: {}\n", cancel_req.index);

    // Example 2: Modify an order
    tracing::info!("=== Modifying Order ===");
    let modify_req = ModifyOrderTxReq {
        market_index: 0,
        index: 123456,
        base_amount: 2000000,
        price: 105000000,
        trigger_price: 0,
    };

    let mut opts2 = opts.clone();
    opts2.nonce = Some(2);

    let _modify_tx = tx_client.modify_order(&modify_req, Some(opts2)).await?;

    tracing::info!("✓ Modify order transaction signed");
    tracing::info!("  Order Index: {}", modify_req.index);
    tracing::info!("  New Amount: {}", modify_req.base_amount);
    tracing::info!("  New Price: {}\n", modify_req.price);

    // Example 3: Create grouped orders (OCO - One Cancels the Other)
    tracing::info!("=== Creating Grouped Orders (OCO) ===");
    let order1 = CreateOrderTxReq {
        market_index: 0,
        client_order_index: 1,
        base_amount: 1000000,
        price: 100000000,
        is_ask: 0,
        order_type: ORDER_TYPE_LIMIT,
        time_in_force: TIME_IN_FORCE_GOOD_TILL_TIME,
        reduce_only: 0,
        trigger_price: 0,
        order_expiry: 0,
    };

    let order2 = CreateOrderTxReq {
        market_index: 0,
        client_order_index: 2,
        base_amount: 1000000,
        price: 110000000,
        is_ask: 1,
        order_type: ORDER_TYPE_LIMIT,
        time_in_force: TIME_IN_FORCE_GOOD_TILL_TIME,
        reduce_only: 0,
        trigger_price: 0,
        order_expiry: 0,
    };

    let grouped_req = CreateGroupedOrdersTxReq {
        grouping_type: GROUPING_TYPE_ONE_CANCELS_THE_OTHER,
        orders: vec![order1, order2],
    };

    let mut opts3 = opts.clone();
    opts3.nonce = Some(3);

    let _grouped_tx = tx_client
        .create_grouped_orders(&grouped_req, Some(opts3))
        .await?;

    tracing::info!("✓ Grouped orders transaction signed");
    tracing::info!("  Grouping Type: ONE_CANCELS_THE_OTHER");
    tracing::info!("  Number of Orders: {}\n", grouped_req.orders.len());

    // Example 4: Cancel all orders
    tracing::info!("=== Cancel All Orders ===");
    let cancel_all_req = CancelAllOrdersTxReq {
        time_in_force: CANCEL_ALL_IMMEDIATE,
        time: 1000000,
    };

    let mut opts4 = opts;
    opts4.nonce = Some(4);

    let _cancel_all_tx = tx_client
        .cancel_all_orders(&cancel_all_req, Some(opts4))
        .await?;

    tracing::info!("✓ Cancel all orders transaction signed");
    tracing::info!("  Time in Force: IMMEDIATE");

    tracing::info!("\n✓ All advanced order operations completed successfully!");

    Ok(())
}
