/// Close the 0.01 ETH position with reduce_only

use dotenv::dotenv;
use lighter_rs::client::TxClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let tx_client = TxClient::new(
        &env::var("LIGHTER_API_URL")?,
        &env::var("LIGHTER_API_KEY")?,
        env::var("LIGHTER_ACCOUNT_INDEX")?.parse()?,
        env::var("LIGHTER_API_KEY_INDEX")?.parse()?,
        304,
    )?;

    println!("Closing 0.01 ETH position with reduce_only=true...\n");

    let close = tx_client.create_market_order(
        0,
        chrono::Utc::now().timestamp_millis(),
        10_000,        // 0.01 ETH exactly
        3_000_000_000,
        1,             // SELL
        true,          // reduce_only!
        None,
    ).await?;

    match tx_client.send_transaction(&close).await {
        Ok(r) if r.code == 200 => {
            println!("✅ Position closed!");
            println!("Tx: {:?}", r.tx_hash);
        }
        Ok(r) => println!("Error {}: {:?}", r.code, r.message),
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}
