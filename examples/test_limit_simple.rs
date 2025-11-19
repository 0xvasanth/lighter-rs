use dotenv::dotenv;
use lighter_rs::client::TxClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenv().ok();
    let tx_client = TxClient::new(
        &env::var("LIGHTER_API_URL")?,
        &env::var("LIGHTER_API_KEY")?,
        env::var("LIGHTER_ACCOUNT_INDEX")?.parse()?,
        env::var("LIGHTER_API_KEY_INDEX")?.parse()?,
        304,
    )?;

    tracing::info!("Testing limit order placement...\n");

    let limit = tx_client
        .create_limit_order(
            0,
            chrono::Utc::now().timestamp_millis(),
            50,            // Tiny: 0.00005 ETH
            2_998_000_000, // $2998
            0,
            false,
            None,
        )
        .await?;

    match tx_client.send_transaction(&limit).await {
        Ok(r) => {
            tracing::info!("Response code: {}", r.code);
            tracing::info!("Message: {:?}", r.message);
            if r.code == 200 {
                tracing::info!("\n✅ SUCCESS!");
            }
        }
        Err(e) => tracing::info!("Error: {}", e),
    }

    Ok(())
}
