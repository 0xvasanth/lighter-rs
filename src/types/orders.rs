//! Order-related transaction types

use super::{OrderInfo, TxInfo};
use crate::constants::*;
use crate::errors::{LighterError, Result};
use serde::{Deserialize, Serialize};

/// Create Order Transaction Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderTxReq {
    pub market_index: u8,
    pub client_order_index: i64,
    pub base_amount: i64,
    pub price: u32,
    pub is_ask: u8,
    pub order_type: u8,
    pub time_in_force: u8,
    pub reduce_only: u8,
    pub trigger_price: u32,
    pub order_expiry: i64,
}

/// L2 Create Order Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2CreateOrderTxInfo {
    pub account_index: i64,
    pub api_key_index: u8,
    pub order_info: OrderInfo,
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<Vec<u8>>,
    #[serde(skip)]
    pub signed_hash: Option<String>,
}

impl TxInfo for L2CreateOrderTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_CREATE_ORDER
    }

    fn get_tx_info(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    fn get_tx_hash(&self) -> Option<String> {
        self.signed_hash.clone()
    }

    fn validate(&self) -> Result<()> {
        // Validate account index
        if self.account_index < MIN_ACCOUNT_INDEX {
            return Err(LighterError::AccountIndexTooLow(self.account_index));
        }
        if self.account_index > MAX_ACCOUNT_INDEX {
            return Err(LighterError::AccountIndexTooHigh(self.account_index));
        }

        // Validate API key index
        if self.api_key_index > MAX_API_KEY_INDEX {
            return Err(LighterError::ApiKeyIndexTooHigh(self.api_key_index));
        }

        // Validate order info
        self.validate_order_info()?;

        // Validate nonce
        if self.nonce < MIN_NONCE {
            return Err(LighterError::NonceTooLow(self.nonce));
        }

        Ok(())
    }

    fn hash(&self, lighter_chain_id: u32) -> Result<Vec<u8>> {
        use poseidon_hash::{hash_to_quintic_extension, Goldilocks};

        // Field order matches lighter-go implementation
        // See: lighter-go/types/txtypes/create_order.go
        let mut elements = Vec::new();

        // 1. Chain ID
        elements.push(Goldilocks::from(lighter_chain_id as u64));

        // 2. Transaction type
        elements.push(Goldilocks::from(TX_TYPE_L2_CREATE_ORDER as u64));

        // 3-4. CRITICAL: Nonce and ExpiredAt come BEFORE account info!
        elements.push(Goldilocks::from(self.nonce as u64));
        elements.push(Goldilocks::from(self.expired_at as u64));

        // 5-6. Account info
        elements.push(Goldilocks::from(self.account_index as u64));
        elements.push(Goldilocks::from(self.api_key_index as u64));

        // 7-16. Order info fields
        elements.push(Goldilocks::from(self.order_info.market_index as u64));
        elements.push(Goldilocks::from(self.order_info.client_order_index as u64));
        elements.push(Goldilocks::from(self.order_info.base_amount as u64));
        elements.push(Goldilocks::from(self.order_info.price as u64));
        elements.push(Goldilocks::from(self.order_info.is_ask as u64));
        elements.push(Goldilocks::from(self.order_info.order_type as u64));
        elements.push(Goldilocks::from(self.order_info.time_in_force as u64));
        elements.push(Goldilocks::from(self.order_info.reduce_only as u64));
        elements.push(Goldilocks::from(self.order_info.trigger_price as u64));
        elements.push(Goldilocks::from(self.order_info.order_expiry as u64));

        // Hash using Poseidon2
        let hash_result = hash_to_quintic_extension(&elements);

        // Convert Fp5Element to bytes (5 field elements * 8 bytes = 40 bytes)
        Ok(hash_result.to_bytes_le().to_vec())
    }
}

impl L2CreateOrderTxInfo {
    fn validate_order_info(&self) -> Result<()> {
        let order = &self.order_info;

        // Market index
        if order.market_index > MAX_MARKET_INDEX {
            return Err(LighterError::MarketIndexTooHigh(order.market_index));
        }

        // Price
        if order.price < MIN_ORDER_PRICE {
            return Err(LighterError::PriceTooLow(order.price));
        }

        // IsAsk
        if order.is_ask != 0 && order.is_ask != 1 {
            return Err(LighterError::IsAskInvalid);
        }

        Ok(())
    }
}

/// Cancel Order Transaction Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderTxReq {
    pub market_index: u8,
    pub index: i64,
}

/// Modify Order Transaction Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifyOrderTxReq {
    pub market_index: u8,
    pub index: i64,
    pub base_amount: i64,
    pub price: u32,
    pub trigger_price: u32,
}

/// Cancel All Orders Transaction Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelAllOrdersTxReq {
    pub time_in_force: u8,
    pub time: i64,
}

/// Create Grouped Orders Transaction Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupedOrdersTxReq {
    pub grouping_type: u8,
    pub orders: Vec<CreateOrderTxReq>,
}

/// L2 Cancel Order Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2CancelOrderTxInfo {
    pub account_index: i64,
    pub api_key_index: u8,
    pub market_index: u8,
    pub index: i64,
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<Vec<u8>>,
    #[serde(skip)]
    pub signed_hash: Option<String>,
}

impl TxInfo for L2CancelOrderTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_CANCEL_ORDER
    }

    fn get_tx_info(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    fn get_tx_hash(&self) -> Option<String> {
        self.signed_hash.clone()
    }

    fn validate(&self) -> Result<()> {
        if self.account_index < MIN_ACCOUNT_INDEX || self.account_index > MAX_ACCOUNT_INDEX {
            return Err(LighterError::AccountIndexTooLow(self.account_index));
        }
        if self.market_index > MAX_MARKET_INDEX {
            return Err(LighterError::MarketIndexTooHigh(self.market_index));
        }
        if self.nonce < MIN_NONCE {
            return Err(LighterError::NonceTooLow(self.nonce));
        }
        Ok(())
    }

    fn hash(&self, lighter_chain_id: u32) -> Result<Vec<u8>> {
        use poseidon_hash::{hash_to_quintic_extension, Goldilocks};

        // Field order matches lighter-go implementation
        // See: lighter-go/types/txtypes/cancel_order.go
        let mut elements = Vec::new();

        // 1. Chain ID
        elements.push(Goldilocks::from(lighter_chain_id as u64));

        // 2. Transaction type
        elements.push(Goldilocks::from(TX_TYPE_L2_CANCEL_ORDER as u64));

        // 3-4. Nonce and ExpiredAt (BEFORE account info!)
        elements.push(Goldilocks::from(self.nonce as u64));
        elements.push(Goldilocks::from(self.expired_at as u64));

        // 5-6. Account info
        elements.push(Goldilocks::from(self.account_index as u64));
        elements.push(Goldilocks::from(self.api_key_index as u64));

        // 7-8. Cancel order fields
        elements.push(Goldilocks::from(self.market_index as u64));
        elements.push(Goldilocks::from(self.index as u64));

        // Hash using Poseidon2
        let hash_result = hash_to_quintic_extension(&elements);
        Ok(hash_result.to_bytes_le().to_vec())
    }
}

/// L2 Modify Order Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2ModifyOrderTxInfo {
    pub account_index: i64,
    pub api_key_index: u8,
    pub market_index: u8,
    pub index: i64,
    pub base_amount: i64,
    pub price: u32,
    pub trigger_price: u32,
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<Vec<u8>>,
    #[serde(skip)]
    pub signed_hash: Option<String>,
}

impl TxInfo for L2ModifyOrderTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_MODIFY_ORDER
    }

    fn get_tx_info(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    fn get_tx_hash(&self) -> Option<String> {
        self.signed_hash.clone()
    }

    fn validate(&self) -> Result<()> {
        if self.account_index < MIN_ACCOUNT_INDEX || self.account_index > MAX_ACCOUNT_INDEX {
            return Err(LighterError::AccountIndexTooLow(self.account_index));
        }
        if self.nonce < MIN_NONCE {
            return Err(LighterError::NonceTooLow(self.nonce));
        }
        Ok(())
    }

    fn hash(&self, lighter_chain_id: u32) -> Result<Vec<u8>> {
        use poseidon_hash::{hash_to_quintic_extension, Goldilocks};

        // Field order matches lighter-go implementation
        // See: lighter-go/types/txtypes/modify_order.go
        let mut elements = Vec::new();

        // 1. Chain ID
        elements.push(Goldilocks::from(lighter_chain_id as u64));

        // 2. Transaction type
        elements.push(Goldilocks::from(TX_TYPE_L2_MODIFY_ORDER as u64));

        // 3-4. Nonce and ExpiredAt (BEFORE account info!)
        elements.push(Goldilocks::from(self.nonce as u64));
        elements.push(Goldilocks::from(self.expired_at as u64));

        // 5-6. Account info
        elements.push(Goldilocks::from(self.account_index as u64));
        elements.push(Goldilocks::from(self.api_key_index as u64));

        // 7-11. Modify order fields
        elements.push(Goldilocks::from(self.market_index as u64));
        elements.push(Goldilocks::from(self.index as u64));
        elements.push(Goldilocks::from(self.base_amount as u64));
        elements.push(Goldilocks::from(self.price as u64));
        elements.push(Goldilocks::from(self.trigger_price as u64));

        // Hash using Poseidon2
        let hash_result = hash_to_quintic_extension(&elements);
        Ok(hash_result.to_bytes_le().to_vec())
    }
}

/// L2 Cancel All Orders Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2CancelAllOrdersTxInfo {
    pub account_index: i64,
    pub api_key_index: u8,
    pub time_in_force: u8,
    pub time: i64,
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<Vec<u8>>,
    #[serde(skip)]
    pub signed_hash: Option<String>,
}

impl TxInfo for L2CancelAllOrdersTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_CANCEL_ALL_ORDERS
    }

    fn get_tx_info(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    fn get_tx_hash(&self) -> Option<String> {
        self.signed_hash.clone()
    }

    fn validate(&self) -> Result<()> {
        if self.account_index < MIN_ACCOUNT_INDEX || self.account_index > MAX_ACCOUNT_INDEX {
            return Err(LighterError::AccountIndexTooLow(self.account_index));
        }
        if self.nonce < MIN_NONCE {
            return Err(LighterError::NonceTooLow(self.nonce));
        }
        Ok(())
    }

    fn hash(&self, _lighter_chain_id: u32) -> Result<Vec<u8>> {
        // TODO: Implement Poseidon2 hashing
        Ok(vec![0u8; 40])
    }
}

/// L2 Create Grouped Orders Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2CreateGroupedOrdersTxInfo {
    pub account_index: i64,
    pub api_key_index: u8,
    pub grouping_type: u8,
    pub orders: Vec<OrderInfo>,
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<Vec<u8>>,
    #[serde(skip)]
    pub signed_hash: Option<String>,
}

impl TxInfo for L2CreateGroupedOrdersTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_CREATE_GROUPED_ORDERS
    }

    fn get_tx_info(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    fn get_tx_hash(&self) -> Option<String> {
        self.signed_hash.clone()
    }

    fn validate(&self) -> Result<()> {
        if self.account_index < MIN_ACCOUNT_INDEX || self.account_index > MAX_ACCOUNT_INDEX {
            return Err(LighterError::AccountIndexTooLow(self.account_index));
        }
        if self.orders.len() > MAX_GROUPED_ORDER_COUNT as usize {
            return Err(LighterError::OrderGroupSizeInvalid);
        }
        if self.nonce < MIN_NONCE {
            return Err(LighterError::NonceTooLow(self.nonce));
        }
        Ok(())
    }

    fn hash(&self, _lighter_chain_id: u32) -> Result<Vec<u8>> {
        // TODO: Implement Poseidon2 hashing
        Ok(vec![0u8; 40])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_order_info() -> OrderInfo {
        OrderInfo {
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
        }
    }

    #[test]
    fn test_create_order_validation_success() {
        let tx_info = L2CreateOrderTxInfo {
            account_index: 12345,
            api_key_index: 0,
            order_info: create_valid_order_info(),
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
    }

    #[test]
    fn test_create_order_account_index_too_low() {
        let tx_info = L2CreateOrderTxInfo {
            account_index: -1,
            api_key_index: 0,
            order_info: create_valid_order_info(),
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LighterError::AccountIndexTooLow(_)
        ));
    }

    #[test]
    fn test_create_order_account_index_too_high() {
        let tx_info = L2CreateOrderTxInfo {
            account_index: MAX_ACCOUNT_INDEX + 1,
            api_key_index: 0,
            order_info: create_valid_order_info(),
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LighterError::AccountIndexTooHigh(_)
        ));
    }

    #[test]
    fn test_create_order_api_key_index_too_high() {
        let tx_info = L2CreateOrderTxInfo {
            account_index: 12345,
            api_key_index: 255,
            order_info: create_valid_order_info(),
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LighterError::ApiKeyIndexTooHigh(_)
        ));
    }

    #[test]
    fn test_create_order_price_too_low() {
        let mut order_info = create_valid_order_info();
        order_info.price = 0;

        let tx_info = L2CreateOrderTxInfo {
            account_index: 12345,
            api_key_index: 0,
            order_info,
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LighterError::PriceTooLow(_)));
    }

    #[test]
    fn test_create_order_is_ask_invalid() {
        let mut order_info = create_valid_order_info();
        order_info.is_ask = 2;

        let tx_info = L2CreateOrderTxInfo {
            account_index: 12345,
            api_key_index: 0,
            order_info,
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LighterError::IsAskInvalid));
    }

    #[test]
    fn test_create_order_nonce_too_low() {
        let tx_info = L2CreateOrderTxInfo {
            account_index: 12345,
            api_key_index: 0,
            order_info: create_valid_order_info(),
            expired_at: 1000000,
            nonce: -1,
            sig: None,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LighterError::NonceTooLow(_)));
    }

    #[test]
    fn test_create_order_tx_type() {
        let tx_info = L2CreateOrderTxInfo {
            account_index: 12345,
            api_key_index: 0,
            order_info: create_valid_order_info(),
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_CREATE_ORDER);
    }

    #[test]
    fn test_cancel_order_validation_success() {
        let tx_info = L2CancelOrderTxInfo {
            account_index: 12345,
            api_key_index: 0,
            market_index: 0,
            index: 123456,
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_CANCEL_ORDER);
    }

    #[test]
    fn test_cancel_order_market_index_too_high() {
        let tx_info = L2CancelOrderTxInfo {
            account_index: 12345,
            api_key_index: 0,
            market_index: 255,
            index: 123456,
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LighterError::MarketIndexTooHigh(_)
        ));
    }

    #[test]
    fn test_modify_order_validation_success() {
        let tx_info = L2ModifyOrderTxInfo {
            account_index: 12345,
            api_key_index: 0,
            market_index: 0,
            index: 123456,
            base_amount: 2000000,
            price: 105000000,
            trigger_price: 0,
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_MODIFY_ORDER);
    }

    #[test]
    fn test_cancel_all_orders_validation_success() {
        let tx_info = L2CancelAllOrdersTxInfo {
            account_index: 12345,
            api_key_index: 0,
            time_in_force: CANCEL_ALL_IMMEDIATE,
            time: 1000000,
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_CANCEL_ALL_ORDERS);
    }

    #[test]
    fn test_create_grouped_orders_validation_success() {
        let tx_info = L2CreateGroupedOrdersTxInfo {
            account_index: 12345,
            api_key_index: 0,
            grouping_type: GROUPING_TYPE_ONE_CANCELS_THE_OTHER,
            orders: vec![create_valid_order_info(), create_valid_order_info()],
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_CREATE_GROUPED_ORDERS);
    }

    #[test]
    fn test_create_grouped_orders_too_many_orders() {
        let tx_info = L2CreateGroupedOrdersTxInfo {
            account_index: 12345,
            api_key_index: 0,
            grouping_type: GROUPING_TYPE_ONE_CANCELS_THE_OTHER,
            orders: vec![
                create_valid_order_info(),
                create_valid_order_info(),
                create_valid_order_info(),
                create_valid_order_info(),
            ],
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LighterError::OrderGroupSizeInvalid
        ));
    }

    #[test]
    fn test_tx_info_serialization() {
        let tx_info = L2CreateOrderTxInfo {
            account_index: 12345,
            api_key_index: 0,
            order_info: create_valid_order_info(),
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        let json_result = tx_info.get_tx_info();
        assert!(json_result.is_ok());

        let json = json_result.unwrap();
        assert!(json.contains("account_index"));
        assert!(json.contains("12345"));
    }
}
