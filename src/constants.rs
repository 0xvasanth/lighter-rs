//! Constants and limits for the Lighter Protocol
//!
//! This module contains all protocol constants including transaction types,
//! order types, time-in-force values, and various limits.

// Transaction Types - L2 Transactions
pub const TX_TYPE_L2_CHANGE_PUB_KEY: u8 = 8;
pub const TX_TYPE_L2_CREATE_SUB_ACCOUNT: u8 = 9;
pub const TX_TYPE_L2_CREATE_PUBLIC_POOL: u8 = 10;
pub const TX_TYPE_L2_UPDATE_PUBLIC_POOL: u8 = 11;
pub const TX_TYPE_L2_TRANSFER: u8 = 12;
pub const TX_TYPE_L2_WITHDRAW: u8 = 13;
pub const TX_TYPE_L2_CREATE_ORDER: u8 = 14;
pub const TX_TYPE_L2_CANCEL_ORDER: u8 = 15;
pub const TX_TYPE_L2_CANCEL_ALL_ORDERS: u8 = 16;
pub const TX_TYPE_L2_MODIFY_ORDER: u8 = 17;
pub const TX_TYPE_L2_MINT_SHARES: u8 = 18;
pub const TX_TYPE_L2_BURN_SHARES: u8 = 19;
pub const TX_TYPE_L2_UPDATE_LEVERAGE: u8 = 20;
pub const TX_TYPE_L2_CREATE_GROUPED_ORDERS: u8 = 28;
pub const TX_TYPE_L2_UPDATE_MARGIN: u8 = 29;

// Transaction Types - Internal
pub const TX_TYPE_INTERNAL_CLAIM_ORDER: u8 = 21;
pub const TX_TYPE_INTERNAL_CANCEL_ORDER: u8 = 22;
pub const TX_TYPE_INTERNAL_DELEVERAGE: u8 = 23;
pub const TX_TYPE_INTERNAL_EXIT_POSITION: u8 = 24;
pub const TX_TYPE_INTERNAL_CANCEL_ALL_ORDERS: u8 = 25;
pub const TX_TYPE_INTERNAL_LIQUIDATE_POSITION: u8 = 26;
pub const TX_TYPE_INTERNAL_CREATE_ORDER: u8 = 27;

// Order Types
pub const ORDER_TYPE_LIMIT: u8 = 0;
pub const ORDER_TYPE_MARKET: u8 = 1;
pub const ORDER_TYPE_STOP_LOSS: u8 = 2;
pub const ORDER_TYPE_STOP_LOSS_LIMIT: u8 = 3;
pub const ORDER_TYPE_TAKE_PROFIT: u8 = 4;
pub const ORDER_TYPE_TAKE_PROFIT_LIMIT: u8 = 5;
pub const ORDER_TYPE_TWAP: u8 = 6;
pub const ORDER_TYPE_TWAP_SUB: u8 = 7;
pub const ORDER_TYPE_LIQUIDATION: u8 = 8;
pub const API_MAX_ORDER_TYPE: u8 = ORDER_TYPE_TWAP;

// Order Time-In-Force
pub const TIME_IN_FORCE_IMMEDIATE_OR_CANCEL: u8 = 0;
pub const TIME_IN_FORCE_GOOD_TILL_TIME: u8 = 1;
pub const TIME_IN_FORCE_POST_ONLY: u8 = 2;

// Grouping Types
pub const GROUPING_TYPE_DEFAULT: u8 = 0;
pub const GROUPING_TYPE_ONE_TRIGGERS_THE_OTHER: u8 = 1;
pub const GROUPING_TYPE_ONE_CANCELS_THE_OTHER: u8 = 2;
pub const GROUPING_TYPE_ONE_TRIGGERS_A_ONE_CANCELS_THE_OTHER: u8 = 3;

// Cancel All Orders Time-In-Force
pub const CANCEL_ALL_IMMEDIATE: u8 = 0;
pub const CANCEL_ALL_SCHEDULED: u8 = 1;
pub const CANCEL_ALL_ABORT_SCHEDULED: u8 = 2;

// Margin Modes
pub const MARGIN_MODE_CROSS: u8 = 0;
pub const MARGIN_MODE_ISOLATED: u8 = 1;

// Margin Direction
pub const MARGIN_REMOVE_FROM_ISOLATED: u8 = 0;
pub const MARGIN_ADD_TO_ISOLATED: u8 = 1;

// Hash and Crypto Constants
pub const HASH_LENGTH: usize = 32;
pub const PRIVATE_KEY_LENGTH: usize = 40;
pub const PUBLIC_KEY_LENGTH: usize = 40;
pub const SIGNATURE_LENGTH: usize = 80;

// USDC and Precision
pub const ONE_USDC: i64 = 1_000_000;
pub const FEE_TICK: i64 = 1_000_000;
pub const MARGIN_FRACTION_TICK: i64 = 10_000;
pub const SHARE_TICK: i64 = 10_000;

// Account Index Limits
pub const MIN_ACCOUNT_INDEX: i64 = 0;
pub const MAX_ACCOUNT_INDEX: i64 = 281_474_976_710_654; // (1 << 48) - 2
pub const MAX_MASTER_ACCOUNT_INDEX: i64 = 140_737_488_355_327; // (1 << 47) - 1

// API Key Index Limits
pub const MIN_API_KEY_INDEX: u8 = 0;
pub const MAX_API_KEY_INDEX: u8 = 254; // (1 << 8) - 2
pub const NIL_API_KEY_INDEX: u8 = MAX_API_KEY_INDEX + 1;

// Market Index Limits
pub const MIN_MARKET_INDEX: u8 = 0;
pub const MAX_MARKET_INDEX: u8 = 254; // (1 << 8) - 2

// Pool Constants
pub const MAX_INVESTED_PUBLIC_POOL_COUNT: i64 = 16;
pub const INITIAL_POOL_SHARE_VALUE: i64 = 1_000; // 0.001 USDC
pub const MIN_INITIAL_TOTAL_SHARES: i64 = 1_000 * (ONE_USDC / INITIAL_POOL_SHARE_VALUE); // 1,000 USDC worth
pub const MAX_INITIAL_TOTAL_SHARES: i64 = 1_000_000_000 * (ONE_USDC / INITIAL_POOL_SHARE_VALUE); // 1B USDC worth
pub const MAX_POOL_SHARES: i64 = (1i64 << 60) - 1;
pub const MAX_BURNT_SHARE_USDC_VALUE: i64 = (1i64 << 60) - 1;
pub const MAX_POOL_ENTRY_USDC: i64 = (1i64 << 56) - 1;
pub const MIN_POOL_SHARES_TO_MINT_OR_BURN: i64 = 1;
pub const MAX_POOL_SHARES_TO_MINT_OR_BURN: i64 = (1i64 << 60) - 1;

// Nonce Limits
pub const MIN_NONCE: i64 = 0;
pub const MIN_ORDER_NONCE: i64 = 0;
pub const MAX_ORDER_NONCE: i64 = (1i64 << 48) - 1;

// Order Index Limits
pub const NIL_CLIENT_ORDER_INDEX: i64 = 0;
pub const NIL_ORDER_INDEX: i64 = 0;
pub const MIN_CLIENT_ORDER_INDEX: i64 = 1;
pub const MAX_CLIENT_ORDER_INDEX: i64 = (1i64 << 48) - 1;
pub const MIN_ORDER_INDEX: i64 = MAX_CLIENT_ORDER_INDEX + 1;
pub const MAX_ORDER_INDEX: i64 = (1i64 << 56) - 1;

// Order Amount Limits
pub const MIN_ORDER_BASE_AMOUNT: i64 = 1;
pub const MAX_ORDER_BASE_AMOUNT: i64 = (1i64 << 48) - 1;
pub const NIL_ORDER_BASE_AMOUNT: i64 = 0;

// Order Price Limits
pub const NIL_ORDER_PRICE: u32 = 0;
pub const MIN_ORDER_PRICE: u32 = 1;
pub const MAX_ORDER_PRICE: u32 = u32::MAX;

// Order Cancel All Period Limits (milliseconds)
pub const MIN_ORDER_CANCEL_ALL_PERIOD: i64 = 1000 * 60 * 5; // 5 minutes
pub const MAX_ORDER_CANCEL_ALL_PERIOD: i64 = 1000 * 60 * 60 * 24 * 15; // 15 days

// Order Expiry Limits
pub const NIL_ORDER_EXPIRY: i64 = 0;
pub const MIN_ORDER_EXPIRY: i64 = 1;
pub const MAX_ORDER_EXPIRY: i64 = i64::MAX;
pub const MIN_ORDER_EXPIRY_PERIOD: i64 = 1000 * 60 * 5; // 5 minutes
pub const MAX_ORDER_EXPIRY_PERIOD: i64 = 1000 * 60 * 60 * 24 * 30; // 30 days

// Order Trigger Price Limits
pub const NIL_ORDER_TRIGGER_PRICE: u32 = 0;
pub const MIN_ORDER_TRIGGER_PRICE: u32 = 1;
pub const MAX_ORDER_TRIGGER_PRICE: u32 = u32::MAX;

// Grouped Orders
pub const MAX_GROUPED_ORDER_COUNT: i64 = 3;

// Timestamp Limits
pub const MAX_TIMESTAMP: i64 = (1i64 << 48) - 1;

// Exchange Limits
pub const MAX_EXCHANGE_USDC: i64 = (1i64 << 60) - 1;

// Transfer Limits
pub const MIN_TRANSFER_AMOUNT: i64 = 1;
pub const MAX_TRANSFER_AMOUNT: i64 = MAX_EXCHANGE_USDC;

// Withdrawal Limits
pub const MIN_WITHDRAWAL_AMOUNT: u64 = 1;
pub const MAX_WITHDRAWAL_AMOUNT: u64 = MAX_EXCHANGE_USDC as u64;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_types() {
        assert_eq!(ORDER_TYPE_LIMIT, 0);
        assert_eq!(ORDER_TYPE_MARKET, 1);
        assert_eq!(API_MAX_ORDER_TYPE, ORDER_TYPE_TWAP);
    }
}
