//! Transfer and withdrawal transaction types

use serde::{Deserialize, Serialize};

/// Transfer Transaction Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTxReq {
    pub to_account_index: i64,
    pub usdc_amount: i64,
    pub fee: i64,
    pub memo: [u8; 32],
}

/// Withdraw Transaction Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawTxReq {
    pub usdc_amount: u64,
}

/// Change Public Key Transaction Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePubKeyReq {
    pub pub_key: Vec<u8>,
}

/// Update Leverage Transaction Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLeverageTxReq {
    pub market_index: u8,
    pub initial_margin_fraction: u16,
    pub margin_mode: u8,
}

/// Update Margin Transaction Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMarginTxReq {
    pub market_index: u8,
    pub usdc_amount: i64,
    pub direction: u8,
}

use super::TxInfo;
use crate::constants::*;
use crate::errors::{LighterError, Result};

/// L2 Transfer Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2TransferTxInfo {
    pub from_account_index: i64,
    pub api_key_index: u8,
    pub to_account_index: i64,
    pub usdc_amount: i64,
    pub fee: i64,
    pub memo: [u8; 32],
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<Vec<u8>>,
    #[serde(skip)]
    pub signed_hash: Option<String>,
}

impl TxInfo for L2TransferTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_TRANSFER
    }

    fn get_tx_info(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    fn get_tx_hash(&self) -> Option<String> {
        self.signed_hash.clone()
    }

    fn validate(&self) -> Result<()> {
        if self.from_account_index < MIN_ACCOUNT_INDEX
            || self.from_account_index > MAX_ACCOUNT_INDEX
        {
            return Err(LighterError::FromAccountIndexTooLow(
                self.from_account_index,
            ));
        }
        if self.to_account_index < MIN_ACCOUNT_INDEX || self.to_account_index > MAX_ACCOUNT_INDEX {
            return Err(LighterError::ToAccountIndexTooLow(self.to_account_index));
        }
        if self.usdc_amount < MIN_TRANSFER_AMOUNT || self.usdc_amount > MAX_TRANSFER_AMOUNT {
            return Err(LighterError::TransferAmountTooLow(self.usdc_amount));
        }
        if self.fee < 0 {
            return Err(LighterError::TransferFeeNegative);
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

/// L2 Withdraw Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2WithdrawTxInfo {
    pub from_account_index: i64,
    pub api_key_index: u8,
    pub usdc_amount: u64,
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<Vec<u8>>,
    #[serde(skip)]
    pub signed_hash: Option<String>,
}

impl TxInfo for L2WithdrawTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_WITHDRAW
    }

    fn get_tx_info(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    fn get_tx_hash(&self) -> Option<String> {
        self.signed_hash.clone()
    }

    fn validate(&self) -> Result<()> {
        if self.from_account_index < MIN_ACCOUNT_INDEX
            || self.from_account_index > MAX_ACCOUNT_INDEX
        {
            return Err(LighterError::FromAccountIndexTooLow(
                self.from_account_index,
            ));
        }
        if self.usdc_amount < MIN_WITHDRAWAL_AMOUNT || self.usdc_amount > MAX_WITHDRAWAL_AMOUNT {
            return Err(LighterError::WithdrawalAmountTooLow(self.usdc_amount));
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

/// L2 Change Public Key Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2ChangePubKeyTxInfo {
    pub account_index: i64,
    pub api_key_index: u8,
    pub pub_key: Vec<u8>,
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<Vec<u8>>,
    #[serde(skip)]
    pub signed_hash: Option<String>,
}

impl TxInfo for L2ChangePubKeyTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_CHANGE_PUB_KEY
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
        if self.pub_key.len() != PUBLIC_KEY_LENGTH {
            return Err(LighterError::PubKeyInvalid);
        }
        if self.nonce < MIN_NONCE {
            return Err(LighterError::NonceTooLow(self.nonce));
        }
        Ok(())
    }

    fn hash(&self, lighter_chain_id: u32) -> Result<Vec<u8>> {
        use poseidon_hash::{hash_to_quintic_extension, Goldilocks};

        let mut elements = Vec::new();

        // Add chain ID and transaction type
        elements.push(Goldilocks::from(lighter_chain_id as u64));
        elements.push(Goldilocks::from(TX_TYPE_L2_CHANGE_PUB_KEY as u64));

        // Add transaction fields
        elements.push(Goldilocks::from(self.account_index as u64));
        elements.push(Goldilocks::from(self.api_key_index as u64));

        // Add public key as field elements (40 bytes = 5 * u64)
        for chunk in self.pub_key.chunks(8) {
            let mut bytes = [0u8; 8];
            bytes[..chunk.len()].copy_from_slice(chunk);
            elements.push(Goldilocks::from(u64::from_le_bytes(bytes)));
        }

        // Add transaction metadata
        elements.push(Goldilocks::from(self.expired_at as u64));
        elements.push(Goldilocks::from(self.nonce as u64));

        // Hash using Poseidon2
        let hash_result = hash_to_quintic_extension(&elements);
        Ok(hash_result.to_bytes_le().to_vec())
    }
}

/// L2 Update Leverage Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2UpdateLeverageTxInfo {
    pub account_index: i64,
    pub api_key_index: u8,
    pub market_index: u8,
    pub initial_margin_fraction: u16,
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<Vec<u8>>,
    #[serde(skip)]
    pub signed_hash: Option<String>,
}

impl TxInfo for L2UpdateLeverageTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_UPDATE_LEVERAGE
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
        if self.initial_margin_fraction as i64 > MARGIN_FRACTION_TICK {
            return Err(LighterError::InitialMarginFractionTooHigh(
                self.initial_margin_fraction,
            ));
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

/// L2 Update Margin Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2UpdateMarginTxInfo {
    pub account_index: i64,
    pub api_key_index: u8,
    pub market_index: u8,
    pub usdc_amount: i64,
    pub direction: u8,
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<Vec<u8>>,
    #[serde(skip)]
    pub signed_hash: Option<String>,
}

impl TxInfo for L2UpdateMarginTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_UPDATE_MARGIN
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
        if self.direction != MARGIN_REMOVE_FROM_ISOLATED && self.direction != MARGIN_ADD_TO_ISOLATED
        {
            return Err(LighterError::InvalidUpdateMarginDirection);
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

/// L2 Create Sub Account Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2CreateSubAccountTxInfo {
    pub account_index: i64,
    pub api_key_index: u8,
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<Vec<u8>>,
    #[serde(skip)]
    pub signed_hash: Option<String>,
}

impl TxInfo for L2CreateSubAccountTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_CREATE_SUB_ACCOUNT
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_validation_success() {
        let tx_info = L2TransferTxInfo {
            from_account_index: 12345,
            api_key_index: 0,
            to_account_index: 54321,
            usdc_amount: 1000000,
            fee: 1000,
            memo: [0u8; 32],
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_TRANSFER);
    }

    #[test]
    fn test_transfer_amount_too_low() {
        let tx_info = L2TransferTxInfo {
            from_account_index: 12345,
            api_key_index: 0,
            to_account_index: 54321,
            usdc_amount: 0,
            fee: 1000,
            memo: [0u8; 32],
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_transfer_negative_fee() {
        let tx_info = L2TransferTxInfo {
            from_account_index: 12345,
            api_key_index: 0,
            to_account_index: 54321,
            usdc_amount: 1000000,
            fee: -1,
            memo: [0u8; 32],
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LighterError::TransferFeeNegative
        ));
    }

    #[test]
    fn test_withdraw_validation_success() {
        let tx_info = L2WithdrawTxInfo {
            from_account_index: 12345,
            api_key_index: 0,
            usdc_amount: 1000000,
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_WITHDRAW);
    }

    #[test]
    fn test_change_pub_key_validation_success() {
        let tx_info = L2ChangePubKeyTxInfo {
            account_index: 12345,
            api_key_index: 0,
            pub_key: vec![0u8; 40],
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_CHANGE_PUB_KEY);
    }

    #[test]
    fn test_change_pub_key_invalid_length() {
        let tx_info = L2ChangePubKeyTxInfo {
            account_index: 12345,
            api_key_index: 0,
            pub_key: vec![0u8; 20],
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LighterError::PubKeyInvalid));
    }

    #[test]
    fn test_update_leverage_validation_success() {
        let tx_info = L2UpdateLeverageTxInfo {
            account_index: 12345,
            api_key_index: 0,
            market_index: 0,
            initial_margin_fraction: 5000,
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_UPDATE_LEVERAGE);
    }

    #[test]
    fn test_update_margin_validation_success() {
        let tx_info = L2UpdateMarginTxInfo {
            account_index: 12345,
            api_key_index: 0,
            market_index: 0,
            usdc_amount: 1000000,
            direction: MARGIN_ADD_TO_ISOLATED,
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_UPDATE_MARGIN);
    }

    #[test]
    fn test_update_margin_invalid_direction() {
        let tx_info = L2UpdateMarginTxInfo {
            account_index: 12345,
            api_key_index: 0,
            market_index: 0,
            usdc_amount: 1000000,
            direction: 2,
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LighterError::InvalidUpdateMarginDirection
        ));
    }

    #[test]
    fn test_create_sub_account_validation_success() {
        let tx_info = L2CreateSubAccountTxInfo {
            account_index: 12345,
            api_key_index: 0,
            expired_at: 1000000,
            nonce: 1,
            sig: None,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_CREATE_SUB_ACCOUNT);
    }
}
