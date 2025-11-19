//! WebSocket client for real-time Lighter Protocol data streams
//!
//! This module provides WebSocket connectivity to subscribe to:
//! - Order book updates
//! - Account updates
//! - Real-time trading data

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::errors::{LighterError, Result};

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessageType {
    #[serde(rename = "connected")]
    Connected,
    #[serde(rename = "subscribed/order_book")]
    SubscribedOrderBook,
    #[serde(rename = "update/order_book")]
    UpdateOrderBook,
    #[serde(rename = "subscribed/account_all")]
    SubscribedAccount,
    #[serde(rename = "update/account_all")]
    UpdateAccount,
}

/// Subscription request message
#[derive(Debug, Clone, Serialize)]
struct SubscribeMessage {
    #[serde(rename = "type")]
    msg_type: String,
    channel: String,
}

/// Order book data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub asks: Vec<PriceLevel>,
    pub bids: Vec<PriceLevel>,
}

/// Price level in order book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: String,
    pub size: String,
}

/// WebSocket client configuration
pub struct WsClientBuilder {
    host: Option<String>,
    path: String,
    order_book_ids: Vec<u32>,
    account_ids: Vec<i64>,
}

impl WsClientBuilder {
    /// Create a new WebSocket client builder
    pub fn new() -> Self {
        Self {
            host: None,
            path: "/stream".to_string(),
            order_book_ids: Vec::new(),
            account_ids: Vec::new(),
        }
    }

    /// Set the WebSocket host (defaults to testnet)
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Set the WebSocket path (defaults to "/stream")
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    /// Subscribe to order book updates for specific markets
    pub fn order_books(mut self, ids: Vec<u32>) -> Self {
        self.order_book_ids = ids;
        self
    }

    /// Subscribe to account updates for specific accounts
    pub fn accounts(mut self, ids: Vec<i64>) -> Self {
        self.account_ids = ids;
        self
    }

    /// Build the WebSocket client
    pub fn build(self) -> Result<WsClient> {
        if self.order_book_ids.is_empty() && self.account_ids.is_empty() {
            return Err(LighterError::ValidationError(
                "At least one subscription (order_book or account) is required".to_string(),
            ));
        }

        let host = self
            .host
            .unwrap_or_else(|| "api-testnet.lighter.xyz".to_string());
        let base_url = format!("wss://{}{}", host, self.path);

        Ok(WsClient {
            base_url,
            order_book_ids: self.order_book_ids,
            account_ids: self.account_ids,
            order_book_states: Arc::new(RwLock::new(HashMap::new())),
            account_states: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

impl Default for WsClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket client for Lighter Protocol
pub struct WsClient {
    base_url: String,
    order_book_ids: Vec<u32>,
    account_ids: Vec<i64>,
    order_book_states: Arc<RwLock<HashMap<String, OrderBook>>>,
    account_states: Arc<RwLock<HashMap<String, Value>>>,
}

impl std::fmt::Debug for WsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WsClient")
            .field("base_url", &self.base_url)
            .field("order_book_ids", &self.order_book_ids)
            .field("account_ids", &self.account_ids)
            .finish()
    }
}

impl WsClient {
    /// Create a new WebSocket client builder
    pub fn builder() -> WsClientBuilder {
        WsClientBuilder::new()
    }

    /// Run the WebSocket client with callbacks
    ///
    /// # Arguments
    /// * `on_order_book_update` - Callback for order book updates (market_id, order_book)
    /// * `on_account_update` - Callback for account updates (account_id, account_data)
    pub async fn run<F1, F2>(&self, on_order_book_update: F1, on_account_update: F2) -> Result<()>
    where
        F1: Fn(String, OrderBook) + Send + Sync + 'static,
        F2: Fn(String, Value) + Send + Sync + 'static,
    {
        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&self.base_url).await.map_err(|e| {
            LighterError::InvalidConfiguration(format!("WebSocket connection failed: {e}"))
        })?;

        tracing::info!(base_url = %self.base_url, "WebSocket connected");

        let (mut write, mut read) = ws_stream.split();

        // Clone states for message handler
        let order_book_states = self.order_book_states.clone();
        let account_states = self.account_states.clone();
        let order_book_ids = self.order_book_ids.clone();
        let account_ids = self.account_ids.clone();

        // Wrap callbacks in Arc for sharing
        let on_order_book_update = Arc::new(on_order_book_update);
        let on_account_update = Arc::new(on_account_update);

        // Message handling loop
        while let Some(message) = read.next().await {
            let message = message
                .map_err(|e| LighterError::InvalidResponse(format!("WebSocket error: {e}")))?;

            if let Message::Text(text) = message {
                let parsed: Value = serde_json::from_str(&text)?;
                let msg_type = parsed.get("type").and_then(|t| t.as_str());

                match msg_type {
                    Some("connected") => {
                        tracing::info!("WebSocket connection established");
                        // Send subscriptions
                        for market_id in &order_book_ids {
                            let sub_msg = SubscribeMessage {
                                msg_type: "subscribe".to_string(),
                                channel: format!("order_book/{market_id}"),
                            };
                            let json = serde_json::to_string(&sub_msg)?;
                            write.send(Message::Text(json)).await.map_err(|e| {
                                LighterError::InvalidResponse(format!("Send error: {e}"))
                            })?;
                            tracing::debug!(market_id = %market_id, "Subscribed to order_book");
                        }

                        for account_id in &account_ids {
                            let sub_msg = SubscribeMessage {
                                msg_type: "subscribe".to_string(),
                                channel: format!("account_all/{account_id}"),
                            };
                            let json = serde_json::to_string(&sub_msg)?;
                            write.send(Message::Text(json)).await.map_err(|e| {
                                LighterError::InvalidResponse(format!("Send error: {e}"))
                            })?;
                            tracing::debug!(account_id = %account_id, "Subscribed to account_all");
                        }
                    }
                    Some("subscribed/order_book") => {
                        if let Some(channel) = parsed.get("channel").and_then(|c| c.as_str()) {
                            let market_id = channel.split(':').nth(1).unwrap_or("unknown");
                            if let Some(order_book) = parsed.get("order_book") {
                                let ob: OrderBook = serde_json::from_value(order_book.clone())?;
                                order_book_states
                                    .write()
                                    .await
                                    .insert(market_id.to_string(), ob.clone());
                                on_order_book_update(market_id.to_string(), ob);
                            }
                        }
                    }
                    Some("update/order_book") => {
                        if let Some(channel) = parsed.get("channel").and_then(|c| c.as_str()) {
                            let market_id = channel.split(':').nth(1).unwrap_or("unknown");
                            if let Some(update) = parsed.get("order_book") {
                                let mut states = order_book_states.write().await;
                                if let Some(existing) = states.get_mut(market_id) {
                                    // Update order book state
                                    Self::update_order_book_state(existing, update)?;
                                    on_order_book_update(market_id.to_string(), existing.clone());
                                }
                            }
                        }
                    }
                    Some("subscribed/account_all") => {
                        if let Some(channel) = parsed.get("channel").and_then(|c| c.as_str()) {
                            let account_id = channel.split(':').nth(1).unwrap_or("unknown");
                            account_states
                                .write()
                                .await
                                .insert(account_id.to_string(), parsed.clone());
                            on_account_update(account_id.to_string(), parsed);
                        }
                    }
                    Some("update/account_all") => {
                        if let Some(channel) = parsed.get("channel").and_then(|c| c.as_str()) {
                            let account_id = channel.split(':').nth(1).unwrap_or("unknown");
                            account_states
                                .write()
                                .await
                                .insert(account_id.to_string(), parsed.clone());
                            on_account_update(account_id.to_string(), parsed);
                        }
                    }
                    _ => {
                        tracing::warn!(msg_type = ?msg_type, "Unhandled message type");
                    }
                }
            }
        }

        Ok(())
    }

    /// Update order book state with incremental updates
    fn update_order_book_state(existing: &mut OrderBook, update: &Value) -> Result<()> {
        if let Some(asks) = update.get("asks").and_then(|a| a.as_array()) {
            for ask in asks {
                Self::update_price_levels(&mut existing.asks, ask)?;
            }
        }

        if let Some(bids) = update.get("bids").and_then(|b| b.as_array()) {
            for bid in bids {
                Self::update_price_levels(&mut existing.bids, bid)?;
            }
        }

        // Remove zero-size levels
        existing
            .asks
            .retain(|level| level.size.parse::<f64>().unwrap_or(0.0) > 0.0);
        existing
            .bids
            .retain(|level| level.size.parse::<f64>().unwrap_or(0.0) > 0.0);

        Ok(())
    }

    /// Update a specific price level
    fn update_price_levels(levels: &mut Vec<PriceLevel>, update: &Value) -> Result<()> {
        let price = update.get("price").and_then(|p| p.as_str()).unwrap_or("");
        let size = update.get("size").and_then(|s| s.as_str()).unwrap_or("0");

        // Find existing level
        let mut found = false;
        for level in levels.iter_mut() {
            if level.price == price {
                level.size = size.to_string();
                found = true;
                break;
            }
        }

        // Add new level if not found and size > 0
        if !found && size.parse::<f64>().unwrap_or(0.0) > 0.0 {
            levels.push(PriceLevel {
                price: price.to_string(),
                size: size.to_string(),
            });
        }

        Ok(())
    }

    /// Get current order book state for a market
    pub async fn get_order_book(&self, market_id: &str) -> Option<OrderBook> {
        self.order_book_states.read().await.get(market_id).cloned()
    }

    /// Get current account state
    pub async fn get_account(&self, account_id: &str) -> Option<Value> {
        self.account_states.read().await.get(account_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_client_builder() {
        let client = WsClient::builder()
            .order_books(vec![0, 1])
            .accounts(vec![12345])
            .build();

        assert!(client.is_ok());
    }

    #[test]
    fn test_ws_client_builder_no_subscriptions() {
        let client = WsClient::builder().build();

        assert!(client.is_err());
        assert!(matches!(
            client.unwrap_err(),
            LighterError::ValidationError(_)
        ));
    }

    #[test]
    fn test_update_price_levels() {
        let mut levels = vec![
            PriceLevel {
                price: "100.0".to_string(),
                size: "10.0".to_string(),
            },
            PriceLevel {
                price: "101.0".to_string(),
                size: "5.0".to_string(),
            },
        ];

        let update = serde_json::json!({
            "price": "100.0",
            "size": "15.0"
        });

        WsClient::update_price_levels(&mut levels, &update).unwrap();

        assert_eq!(levels[0].size, "15.0");
        assert_eq!(levels.len(), 2);
    }

    #[test]
    fn test_update_price_levels_new_level() {
        let mut levels = vec![PriceLevel {
            price: "100.0".to_string(),
            size: "10.0".to_string(),
        }];

        let update = serde_json::json!({
            "price": "102.0",
            "size": "8.0"
        });

        WsClient::update_price_levels(&mut levels, &update).unwrap();

        assert_eq!(levels.len(), 2);
        assert_eq!(levels[1].price, "102.0");
        assert_eq!(levels[1].size, "8.0");
    }
}
