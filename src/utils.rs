//! Utility functions for the Lighter SDK

use crate::errors::{LighterError, Result};
use hex;

/// Convert hex string to bytes, handling optional 0x prefix
pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>> {
    let hex_str = if hex_str.starts_with("0x") || hex_str.starts_with("0X") {
        &hex_str[2..]
    } else {
        hex_str
    };

    Ok(hex::decode(hex_str)?)
}

/// Convert bytes to hex string with 0x prefix
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

/// Convert bytes to hex string without 0x prefix
pub fn bytes_to_hex_no_prefix(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Validate that a value is within a specified range
pub fn validate_range<T: PartialOrd + std::fmt::Display>(
    value: T,
    min: T,
    max: T,
    field_name: &str,
) -> Result<()> {
    if value < min || value > max {
        return Err(LighterError::Other(format!(
            "{field_name} must be between {min} and {max}, got {value}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_bytes() {
        let hex_with_prefix = "0x48656c6c6f";
        let hex_without_prefix = "48656c6c6f";

        let bytes1 = hex_to_bytes(hex_with_prefix).unwrap();
        let bytes2 = hex_to_bytes(hex_without_prefix).unwrap();

        assert_eq!(bytes1, bytes2);
        assert_eq!(bytes1, b"Hello");
    }

    #[test]
    fn test_bytes_to_hex() {
        let bytes = b"Hello";
        let hex_with_prefix = bytes_to_hex(bytes);
        let hex_without_prefix = bytes_to_hex_no_prefix(bytes);

        assert_eq!(hex_with_prefix, "0x48656c6c6f");
        assert_eq!(hex_without_prefix, "48656c6c6f");
    }

    #[test]
    fn test_validate_range() {
        assert!(validate_range(5, 1, 10, "test").is_ok());
        assert!(validate_range(0, 1, 10, "test").is_err());
        assert!(validate_range(11, 1, 10, "test").is_err());
    }
}
