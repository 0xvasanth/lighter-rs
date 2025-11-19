//! HTTP client for interacting with the Lighter API

use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

use crate::constants::*;
use crate::errors::{LighterError, Result};
use crate::signer::{PoseidonKeyManager, Signer};
use crate::types::*;

/// HTTP Client for Lighter API
#[derive(Clone)]
pub struct HTTPClient {
    client: Client,
    endpoint: String,
    fat_finger_protection: bool,
}

impl HTTPClient {
    /// Create a new HTTP client
    pub fn new(base_url: &str) -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        Ok(Self {
            client,
            endpoint: base_url.to_string(),
            fat_finger_protection: true,
        })
    }

    /// Enable or disable fat finger protection
    pub fn set_fat_finger_protection(&mut self, enabled: bool) {
        self.fat_finger_protection = enabled;
    }

    /// Get the next nonce for an account and API key
    pub async fn get_next_nonce(&self, account_index: i64, api_key_index: u8) -> Result<i64> {
        let url = format!(
            "{}/api/v1/nextNonce?account_index={}&api_key_index={}",
            self.endpoint, account_index, api_key_index
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(LighterError::ApiError(format!(
                "Failed to get nonce: {}",
                response.status()
            )));
        }

        #[derive(Deserialize)]
        struct NonceResponse {
            nonce: i64,
        }

        let nonce_response: NonceResponse = response.json().await?;
        Ok(nonce_response.nonce)
    }

    /// Send a transaction to the Lighter API
    ///
    /// # Arguments
    /// * `tx_type` - Transaction type identifier
    /// * `tx_info` - JSON-serialized transaction info
    pub async fn send_tx(&self, tx_type: u8, tx_info: &str) -> Result<TxResponse> {
        let url = format!("{}/api/v1/sendTx", self.endpoint);

        #[derive(serde::Serialize)]
        struct SendTxRequest {
            tx_type: u8,
            tx_info: String,
        }

        let request_body = SendTxRequest {
            tx_type,
            tx_info: tx_info.to_string(),
        };

        // Debug: print request
        let request_json = serde_json::to_string(&request_body)?;
        eprintln!("DEBUG - Sending request: {}", request_json);

        let response = self.client.post(&url).json(&request_body).send().await?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LighterError::ApiError(format!(
                "Failed to send transaction: {}",
                error_text
            )));
        }

        let tx_response: TxResponse = response.json().await?;
        Ok(tx_response)
    }
}

/// Response from send_tx API call
#[derive(Debug, Clone, Deserialize)]
pub struct TxResponse {
    pub code: u16,
    pub tx_hash: Option<String>,
    pub message: Option<String>,
}

/// Transaction Client for signing and submitting transactions
pub struct TxClient {
    api_client: Option<HTTPClient>,
    chain_id: u32,
    key_manager: PoseidonKeyManager,
    account_index: i64,
    api_key_index: u8,
}

impl TxClient {
    /// Create a new transaction client
    ///
    /// # Arguments
    /// * `api_client_url` - Base URL for the Lighter API (or empty string to disable API calls)
    /// * `api_key_private_key` - Hex-encoded private key (with or without 0x prefix)
    /// * `account_index` - Account index
    /// * `api_key_index` - API key index
    /// * `chain_id` - Chain ID
    pub fn new(
        api_client_url: &str,
        api_key_private_key: &str,
        account_index: i64,
        api_key_index: u8,
        chain_id: u32,
    ) -> Result<Self> {
        let key_manager = PoseidonKeyManager::from_hex(api_key_private_key)?;

        let api_client = if !api_client_url.is_empty() {
            Some(HTTPClient::new(api_client_url)?)
        } else {
            None
        };

        Ok(Self {
            api_client,
            chain_id,
            key_manager,
            account_index,
            api_key_index,
        })
    }

    /// Get the account index
    pub fn account_index(&self) -> i64 {
        self.account_index
    }

    /// Get the API key index
    pub fn api_key_index(&self) -> u8 {
        self.api_key_index
    }

    /// Get a reference to the key manager
    pub fn key_manager(&self) -> &PoseidonKeyManager {
        &self.key_manager
    }

    /// Get a reference to the HTTP client
    pub fn http(&self) -> Option<&HTTPClient> {
        self.api_client.as_ref()
    }

    /// Switch to a different API key
    pub fn switch_api_key(&mut self, api_key: u8) {
        self.api_key_index = api_key;
    }

    /// Fill in default transaction options
    pub async fn fill_default_opts(&self, opts: Option<TransactOpts>) -> Result<TransactOpts> {
        let mut opts = opts.unwrap_or_default();

        if opts.expired_at == 0 {
            use chrono::Utc;
            // Default to 10 minutes from now
            opts.expired_at = (Utc::now().timestamp_millis() + 600_000) - 1000;
        }

        if opts.from_account_index.is_none() {
            opts.from_account_index = Some(self.account_index);
        }

        if opts.api_key_index.is_none() {
            opts.api_key_index = Some(self.api_key_index);
        }

        if opts.nonce.is_none() {
            if let Some(client) = &self.api_client {
                let nonce = client
                    .get_next_nonce(
                        opts.from_account_index.unwrap(),
                        opts.api_key_index.unwrap(),
                    )
                    .await?;
                opts.nonce = Some(nonce);
            } else {
                return Err(LighterError::MissingField(
                    "nonce was not provided and HTTPClient is not available".to_string(),
                ));
            }
        }

        Ok(opts)
    }

    /// Construct and sign a create order transaction
    pub async fn create_order(
        &self,
        req: &CreateOrderTxReq,
        opts: Option<TransactOpts>,
    ) -> Result<L2CreateOrderTxInfo> {
        let opts = self.fill_default_opts(opts).await?;

        let mut tx_info = L2CreateOrderTxInfo {
            account_index: opts.from_account_index.unwrap(),
            api_key_index: opts.api_key_index.unwrap(),
            order_info: OrderInfo {
                market_index: req.market_index,
                client_order_index: req.client_order_index,
                base_amount: req.base_amount,
                price: req.price,
                is_ask: req.is_ask,
                order_type: req.order_type,
                time_in_force: req.time_in_force,
                reduce_only: req.reduce_only,
                trigger_price: req.trigger_price,
                order_expiry: req.order_expiry,
            },
            expired_at: opts.expired_at,
            nonce: opts.nonce.unwrap(),
            sig: None,
            signed_hash: None,
        };

        // Validate
        tx_info.validate()?;

        // Hash and sign
        let msg_hash = tx_info.hash(self.chain_id)?;
        let signature = self.key_manager.sign(&msg_hash)?;

        tx_info.sig = Some(signature);
        tx_info.signed_hash = Some(hex::encode(&msg_hash));

        Ok(tx_info)
    }

    /// Construct and sign a cancel order transaction
    pub async fn cancel_order(
        &self,
        req: &CancelOrderTxReq,
        opts: Option<TransactOpts>,
    ) -> Result<L2CancelOrderTxInfo> {
        let opts = self.fill_default_opts(opts).await?;

        let mut tx_info = L2CancelOrderTxInfo {
            account_index: opts.from_account_index.unwrap(),
            api_key_index: opts.api_key_index.unwrap(),
            market_index: req.market_index,
            index: req.index,
            expired_at: opts.expired_at,
            nonce: opts.nonce.unwrap(),
            sig: None,
            signed_hash: None,
        };

        tx_info.validate()?;
        let msg_hash = tx_info.hash(self.chain_id)?;
        let signature = self.key_manager.sign(&msg_hash)?;
        tx_info.sig = Some(signature);
        tx_info.signed_hash = Some(hex::encode(&msg_hash));

        Ok(tx_info)
    }

    /// Construct and sign a modify order transaction
    pub async fn modify_order(
        &self,
        req: &ModifyOrderTxReq,
        opts: Option<TransactOpts>,
    ) -> Result<L2ModifyOrderTxInfo> {
        let opts = self.fill_default_opts(opts).await?;

        let mut tx_info = L2ModifyOrderTxInfo {
            account_index: opts.from_account_index.unwrap(),
            api_key_index: opts.api_key_index.unwrap(),
            market_index: req.market_index,
            index: req.index,
            base_amount: req.base_amount,
            price: req.price,
            trigger_price: req.trigger_price,
            expired_at: opts.expired_at,
            nonce: opts.nonce.unwrap(),
            sig: None,
            signed_hash: None,
        };

        tx_info.validate()?;
        let msg_hash = tx_info.hash(self.chain_id)?;
        let signature = self.key_manager.sign(&msg_hash)?;
        tx_info.sig = Some(signature);
        tx_info.signed_hash = Some(hex::encode(&msg_hash));

        Ok(tx_info)
    }

    /// Construct and sign a cancel all orders transaction
    pub async fn cancel_all_orders(
        &self,
        req: &CancelAllOrdersTxReq,
        opts: Option<TransactOpts>,
    ) -> Result<L2CancelAllOrdersTxInfo> {
        let opts = self.fill_default_opts(opts).await?;

        let mut tx_info = L2CancelAllOrdersTxInfo {
            account_index: opts.from_account_index.unwrap(),
            api_key_index: opts.api_key_index.unwrap(),
            time_in_force: req.time_in_force,
            time: req.time,
            expired_at: opts.expired_at,
            nonce: opts.nonce.unwrap(),
            sig: None,
            signed_hash: None,
        };

        tx_info.validate()?;
        let msg_hash = tx_info.hash(self.chain_id)?;
        let signature = self.key_manager.sign(&msg_hash)?;
        tx_info.sig = Some(signature);
        tx_info.signed_hash = Some(hex::encode(&msg_hash));

        Ok(tx_info)
    }

    /// Construct and sign a create grouped orders transaction
    pub async fn create_grouped_orders(
        &self,
        req: &CreateGroupedOrdersTxReq,
        opts: Option<TransactOpts>,
    ) -> Result<L2CreateGroupedOrdersTxInfo> {
        let opts = self.fill_default_opts(opts).await?;

        let orders: Vec<OrderInfo> = req
            .orders
            .iter()
            .map(|o| OrderInfo {
                market_index: o.market_index,
                client_order_index: o.client_order_index,
                base_amount: o.base_amount,
                price: o.price,
                is_ask: o.is_ask,
                order_type: o.order_type,
                time_in_force: o.time_in_force,
                reduce_only: o.reduce_only,
                trigger_price: o.trigger_price,
                order_expiry: o.order_expiry,
            })
            .collect();

        let mut tx_info = L2CreateGroupedOrdersTxInfo {
            account_index: opts.from_account_index.unwrap(),
            api_key_index: opts.api_key_index.unwrap(),
            grouping_type: req.grouping_type,
            orders,
            expired_at: opts.expired_at,
            nonce: opts.nonce.unwrap(),
            sig: None,
            signed_hash: None,
        };

        tx_info.validate()?;
        let msg_hash = tx_info.hash(self.chain_id)?;
        let signature = self.key_manager.sign(&msg_hash)?;
        tx_info.sig = Some(signature);
        tx_info.signed_hash = Some(hex::encode(&msg_hash));

        Ok(tx_info)
    }

    /// Construct and sign a transfer transaction
    pub async fn transfer(
        &self,
        req: &TransferTxReq,
        opts: Option<TransactOpts>,
    ) -> Result<L2TransferTxInfo> {
        let opts = self.fill_default_opts(opts).await?;

        let mut tx_info = L2TransferTxInfo {
            from_account_index: opts.from_account_index.unwrap(),
            api_key_index: opts.api_key_index.unwrap(),
            to_account_index: req.to_account_index,
            usdc_amount: req.usdc_amount,
            fee: req.fee,
            memo: req.memo,
            expired_at: opts.expired_at,
            nonce: opts.nonce.unwrap(),
            sig: None,
            signed_hash: None,
        };

        tx_info.validate()?;
        let msg_hash = tx_info.hash(self.chain_id)?;
        let signature = self.key_manager.sign(&msg_hash)?;
        tx_info.sig = Some(signature);
        tx_info.signed_hash = Some(hex::encode(&msg_hash));

        Ok(tx_info)
    }

    /// Construct and sign a withdraw transaction
    pub async fn withdraw(
        &self,
        req: &WithdrawTxReq,
        opts: Option<TransactOpts>,
    ) -> Result<L2WithdrawTxInfo> {
        let opts = self.fill_default_opts(opts).await?;

        let mut tx_info = L2WithdrawTxInfo {
            from_account_index: opts.from_account_index.unwrap(),
            api_key_index: opts.api_key_index.unwrap(),
            usdc_amount: req.usdc_amount,
            expired_at: opts.expired_at,
            nonce: opts.nonce.unwrap(),
            sig: None,
            signed_hash: None,
        };

        tx_info.validate()?;
        let msg_hash = tx_info.hash(self.chain_id)?;
        let signature = self.key_manager.sign(&msg_hash)?;
        tx_info.sig = Some(signature);
        tx_info.signed_hash = Some(hex::encode(&msg_hash));

        Ok(tx_info)
    }

    /// Construct and sign a change public key transaction
    pub async fn change_pub_key(
        &self,
        req: &ChangePubKeyReq,
        opts: Option<TransactOpts>,
    ) -> Result<L2ChangePubKeyTxInfo> {
        let opts = self.fill_default_opts(opts).await?;

        let mut tx_info = L2ChangePubKeyTxInfo {
            account_index: opts.from_account_index.unwrap(),
            api_key_index: opts.api_key_index.unwrap(),
            pub_key: req.pub_key.clone(),
            expired_at: opts.expired_at,
            nonce: opts.nonce.unwrap(),
            sig: None,
            signed_hash: None,
        };

        tx_info.validate()?;
        let msg_hash = tx_info.hash(self.chain_id)?;
        let signature = self.key_manager.sign(&msg_hash)?;
        tx_info.sig = Some(signature);
        tx_info.signed_hash = Some(hex::encode(&msg_hash));

        Ok(tx_info)
    }

    /// Construct and sign an update leverage transaction
    pub async fn update_leverage(
        &self,
        req: &UpdateLeverageTxReq,
        opts: Option<TransactOpts>,
    ) -> Result<L2UpdateLeverageTxInfo> {
        let opts = self.fill_default_opts(opts).await?;

        let mut tx_info = L2UpdateLeverageTxInfo {
            account_index: opts.from_account_index.unwrap(),
            api_key_index: opts.api_key_index.unwrap(),
            market_index: req.market_index,
            initial_margin_fraction: req.initial_margin_fraction,
            expired_at: opts.expired_at,
            nonce: opts.nonce.unwrap(),
            sig: None,
            signed_hash: None,
        };

        tx_info.validate()?;
        let msg_hash = tx_info.hash(self.chain_id)?;
        let signature = self.key_manager.sign(&msg_hash)?;
        tx_info.sig = Some(signature);
        tx_info.signed_hash = Some(hex::encode(&msg_hash));

        Ok(tx_info)
    }

    /// Construct and sign an update margin transaction
    pub async fn update_margin(
        &self,
        req: &UpdateMarginTxReq,
        opts: Option<TransactOpts>,
    ) -> Result<L2UpdateMarginTxInfo> {
        let opts = self.fill_default_opts(opts).await?;

        let mut tx_info = L2UpdateMarginTxInfo {
            account_index: opts.from_account_index.unwrap(),
            api_key_index: opts.api_key_index.unwrap(),
            market_index: req.market_index,
            usdc_amount: req.usdc_amount,
            direction: req.direction,
            expired_at: opts.expired_at,
            nonce: opts.nonce.unwrap(),
            sig: None,
            signed_hash: None,
        };

        tx_info.validate()?;
        let msg_hash = tx_info.hash(self.chain_id)?;
        let signature = self.key_manager.sign(&msg_hash)?;
        tx_info.sig = Some(signature);
        tx_info.signed_hash = Some(hex::encode(&msg_hash));

        Ok(tx_info)
    }

    /// Construct and sign a create sub account transaction
    pub async fn create_sub_account(
        &self,
        opts: Option<TransactOpts>,
    ) -> Result<L2CreateSubAccountTxInfo> {
        let opts = self.fill_default_opts(opts).await?;

        let mut tx_info = L2CreateSubAccountTxInfo {
            account_index: opts.from_account_index.unwrap(),
            api_key_index: opts.api_key_index.unwrap(),
            expired_at: opts.expired_at,
            nonce: opts.nonce.unwrap(),
            sig: None,
            signed_hash: None,
        };

        tx_info.validate()?;
        let msg_hash = tx_info.hash(self.chain_id)?;
        let signature = self.key_manager.sign(&msg_hash)?;
        tx_info.sig = Some(signature);
        tx_info.signed_hash = Some(hex::encode(&msg_hash));

        Ok(tx_info)
    }

    /// Construct and sign a create public pool transaction
    pub async fn create_public_pool(
        &self,
        req: &CreatePublicPoolTxReq,
        opts: Option<TransactOpts>,
    ) -> Result<L2CreatePublicPoolTxInfo> {
        let opts = self.fill_default_opts(opts).await?;

        let mut tx_info = L2CreatePublicPoolTxInfo {
            account_index: opts.from_account_index.unwrap(),
            api_key_index: opts.api_key_index.unwrap(),
            operator_fee: req.operator_fee,
            initial_total_shares: req.initial_total_shares,
            min_operator_share_rate: req.min_operator_share_rate,
            expired_at: opts.expired_at,
            nonce: opts.nonce.unwrap(),
            sig: None,
            signed_hash: None,
        };

        tx_info.validate()?;
        let msg_hash = tx_info.hash(self.chain_id)?;
        let signature = self.key_manager.sign(&msg_hash)?;
        tx_info.sig = Some(signature);
        tx_info.signed_hash = Some(hex::encode(&msg_hash));

        Ok(tx_info)
    }

    /// Construct and sign an update public pool transaction
    pub async fn update_public_pool(
        &self,
        req: &UpdatePublicPoolTxReq,
        opts: Option<TransactOpts>,
    ) -> Result<L2UpdatePublicPoolTxInfo> {
        let opts = self.fill_default_opts(opts).await?;

        let mut tx_info = L2UpdatePublicPoolTxInfo {
            account_index: opts.from_account_index.unwrap(),
            api_key_index: opts.api_key_index.unwrap(),
            public_pool_index: req.public_pool_index,
            status: req.status,
            operator_fee: req.operator_fee,
            min_operator_share_rate: req.min_operator_share_rate,
            expired_at: opts.expired_at,
            nonce: opts.nonce.unwrap(),
            sig: None,
            signed_hash: None,
        };

        tx_info.validate()?;
        let msg_hash = tx_info.hash(self.chain_id)?;
        let signature = self.key_manager.sign(&msg_hash)?;
        tx_info.sig = Some(signature);
        tx_info.signed_hash = Some(hex::encode(&msg_hash));

        Ok(tx_info)
    }

    /// Construct and sign a mint shares transaction
    pub async fn mint_shares(
        &self,
        req: &MintSharesTxReq,
        opts: Option<TransactOpts>,
    ) -> Result<L2MintSharesTxInfo> {
        let opts = self.fill_default_opts(opts).await?;

        let mut tx_info = L2MintSharesTxInfo {
            account_index: opts.from_account_index.unwrap(),
            api_key_index: opts.api_key_index.unwrap(),
            public_pool_index: req.public_pool_index,
            share_amount: req.share_amount,
            expired_at: opts.expired_at,
            nonce: opts.nonce.unwrap(),
            sig: None,
            signed_hash: None,
        };

        tx_info.validate()?;
        let msg_hash = tx_info.hash(self.chain_id)?;
        let signature = self.key_manager.sign(&msg_hash)?;
        tx_info.sig = Some(signature);
        tx_info.signed_hash = Some(hex::encode(&msg_hash));

        Ok(tx_info)
    }

    /// Construct and sign a burn shares transaction
    pub async fn burn_shares(
        &self,
        req: &BurnSharesTxReq,
        opts: Option<TransactOpts>,
    ) -> Result<L2BurnSharesTxInfo> {
        let opts = self.fill_default_opts(opts).await?;

        let mut tx_info = L2BurnSharesTxInfo {
            account_index: opts.from_account_index.unwrap(),
            api_key_index: opts.api_key_index.unwrap(),
            public_pool_index: req.public_pool_index,
            share_amount: req.share_amount,
            expired_at: opts.expired_at,
            nonce: opts.nonce.unwrap(),
            sig: None,
            signed_hash: None,
        };

        tx_info.validate()?;
        let msg_hash = tx_info.hash(self.chain_id)?;
        let signature = self.key_manager.sign(&msg_hash)?;
        tx_info.sig = Some(signature);
        tx_info.signed_hash = Some(hex::encode(&msg_hash));

        Ok(tx_info)
    }

    // ========== Helper Methods ==========

    /// Create a limit order (convenience wrapper around create_order)
    ///
    /// Limit orders are placed on the order book at a specific price
    #[allow(clippy::too_many_arguments)]
    pub async fn create_limit_order(
        &self,
        market_index: u8,
        client_order_index: i64,
        base_amount: i64,
        price: u32,
        is_ask: u8,
        reduce_only: bool,
        opts: Option<TransactOpts>,
    ) -> Result<L2CreateOrderTxInfo> {
        let req = CreateOrderTxReq {
            market_index,
            client_order_index,
            base_amount,
            price,
            is_ask,
            order_type: ORDER_TYPE_LIMIT,
            time_in_force: TIME_IN_FORCE_GOOD_TILL_TIME,
            reduce_only: if reduce_only { 1 } else { 0 },
            trigger_price: 0,
            order_expiry: 0,
        };

        self.create_order(&req, opts).await
    }

    /// Create a market order (convenience wrapper around create_order)
    ///
    /// Market orders execute immediately at the best available price
    #[allow(clippy::too_many_arguments)]
    pub async fn create_market_order(
        &self,
        market_index: u8,
        client_order_index: i64,
        base_amount: i64,
        price: u32,
        is_ask: u8,
        reduce_only: bool,
        opts: Option<TransactOpts>,
    ) -> Result<L2CreateOrderTxInfo> {
        let req = CreateOrderTxReq {
            market_index,
            client_order_index,
            base_amount,
            price,
            is_ask,
            order_type: ORDER_TYPE_MARKET,
            time_in_force: TIME_IN_FORCE_IMMEDIATE_OR_CANCEL,
            reduce_only: if reduce_only { 1 } else { 0 },
            trigger_price: 0,
            order_expiry: 0,
        };

        self.create_order(&req, opts).await
    }

    /// Create a take profit order
    #[allow(clippy::too_many_arguments)]
    pub async fn create_tp_order(
        &self,
        market_index: u8,
        client_order_index: i64,
        base_amount: i64,
        trigger_price: u32,
        price: u32,
        is_ask: u8,
        reduce_only: bool,
        opts: Option<TransactOpts>,
    ) -> Result<L2CreateOrderTxInfo> {
        let req = CreateOrderTxReq {
            market_index,
            client_order_index,
            base_amount,
            price,
            is_ask,
            order_type: ORDER_TYPE_TAKE_PROFIT,
            time_in_force: TIME_IN_FORCE_IMMEDIATE_OR_CANCEL,
            reduce_only: if reduce_only { 1 } else { 0 },
            trigger_price,
            order_expiry: 0,
        };

        self.create_order(&req, opts).await
    }

    /// Create a take profit limit order
    #[allow(clippy::too_many_arguments)]
    pub async fn create_tp_limit_order(
        &self,
        market_index: u8,
        client_order_index: i64,
        base_amount: i64,
        trigger_price: u32,
        price: u32,
        is_ask: u8,
        reduce_only: bool,
        opts: Option<TransactOpts>,
    ) -> Result<L2CreateOrderTxInfo> {
        let req = CreateOrderTxReq {
            market_index,
            client_order_index,
            base_amount,
            price,
            is_ask,
            order_type: ORDER_TYPE_TAKE_PROFIT_LIMIT,
            time_in_force: TIME_IN_FORCE_GOOD_TILL_TIME,
            reduce_only: if reduce_only { 1 } else { 0 },
            trigger_price,
            order_expiry: 0,
        };

        self.create_order(&req, opts).await
    }

    /// Create a stop loss order
    #[allow(clippy::too_many_arguments)]
    pub async fn create_sl_order(
        &self,
        market_index: u8,
        client_order_index: i64,
        base_amount: i64,
        trigger_price: u32,
        price: u32,
        is_ask: u8,
        reduce_only: bool,
        opts: Option<TransactOpts>,
    ) -> Result<L2CreateOrderTxInfo> {
        let req = CreateOrderTxReq {
            market_index,
            client_order_index,
            base_amount,
            price,
            is_ask,
            order_type: ORDER_TYPE_STOP_LOSS,
            time_in_force: TIME_IN_FORCE_IMMEDIATE_OR_CANCEL,
            reduce_only: if reduce_only { 1 } else { 0 },
            trigger_price,
            order_expiry: 0,
        };

        self.create_order(&req, opts).await
    }

    /// Create a stop loss limit order
    #[allow(clippy::too_many_arguments)]
    pub async fn create_sl_limit_order(
        &self,
        market_index: u8,
        client_order_index: i64,
        base_amount: i64,
        trigger_price: u32,
        price: u32,
        is_ask: u8,
        reduce_only: bool,
        opts: Option<TransactOpts>,
    ) -> Result<L2CreateOrderTxInfo> {
        let req = CreateOrderTxReq {
            market_index,
            client_order_index,
            base_amount,
            price,
            is_ask,
            order_type: ORDER_TYPE_STOP_LOSS_LIMIT,
            time_in_force: TIME_IN_FORCE_GOOD_TILL_TIME,
            reduce_only: if reduce_only { 1 } else { 0 },
            trigger_price,
            order_expiry: 0,
        };

        self.create_order(&req, opts).await
    }

    /// Update leverage with a user-friendly leverage parameter
    ///
    /// # Arguments
    /// * `market_index` - The market to update leverage for
    /// * `leverage` - Leverage multiplier (e.g., 5 for 5x, 10 for 10x)
    /// * `margin_mode` - MARGIN_MODE_CROSS or MARGIN_MODE_ISOLATED
    /// * `opts` - Optional transaction options
    pub async fn update_leverage_with_multiplier(
        &self,
        market_index: u8,
        leverage: u16,
        margin_mode: u8,
        opts: Option<TransactOpts>,
    ) -> Result<L2UpdateLeverageTxInfo> {
        if leverage == 0 {
            return Err(LighterError::ValidationError(
                "Leverage must be greater than 0".to_string(),
            ));
        }

        // Convert leverage to initial margin fraction
        // IMF = 10,000 / leverage (Python: imf = int(10_000 / leverage))
        let initial_margin_fraction = 10_000 / leverage;

        let req = UpdateLeverageTxReq {
            market_index,
            initial_margin_fraction,
            margin_mode,
        };

        self.update_leverage(&req, opts).await
    }

    /// Send a signed transaction to the API
    ///
    /// # Arguments
    /// * `tx_info` - Any type implementing TxInfo trait
    pub async fn send_transaction<T: TxInfo>(&self, tx_info: &T) -> Result<TxResponse> {
        if let Some(client) = &self.api_client {
            let tx_type = tx_info.get_tx_type();
            let tx_json = tx_info.get_tx_info()?;
            client.send_tx(tx_type, &tx_json).await
        } else {
            Err(LighterError::InvalidConfiguration(
                "HTTPClient is not configured. Provide a valid API URL when creating TxClient."
                    .to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_creation() {
        let client = HTTPClient::new("https://api.lighter.xyz");
        assert!(client.is_ok());
    }
}
