//! Cryptographic signing and key management for Lighter Protocol

use crate::constants::{PRIVATE_KEY_LENGTH, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};
use crate::errors::{LighterError, Result};
use crate::utils::hex_to_bytes;
use goldilocks_crypto::{sign_with_nonce, Point, ScalarField};

/// Trait for signing messages
pub trait Signer {
    fn sign(&self, hashed_message: &[u8]) -> Result<Vec<u8>>;
}

/// Trait for key management operations
pub trait KeyManager: Signer {
    fn pub_key(&self) -> &[u8];
    fn pub_key_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH];
    fn prv_key_bytes(&self) -> Vec<u8>;
}

/// Implementation of key manager using Poseidon cryptography
pub struct PoseidonKeyManager {
    private_key: Vec<u8>,
    public_key: Vec<u8>,
}

impl PoseidonKeyManager {
    pub fn new(private_key_bytes: &[u8]) -> Result<Self> {
        // Accept both 32-byte (256-bit) and 40-byte keys
        // 32 bytes is standard for many cryptographic keys
        // 40 bytes is the Lighter protocol specification
        if private_key_bytes.len() != 32 && private_key_bytes.len() != PRIVATE_KEY_LENGTH {
            return Err(LighterError::InvalidPrivateKeyLength {
                expected: PRIVATE_KEY_LENGTH,
                actual: private_key_bytes.len(),
            });
        }

        let public_key = Self::derive_public_key(private_key_bytes)?;

        Ok(Self {
            private_key: private_key_bytes.to_vec(),
            public_key,
        })
    }

    pub fn from_hex(hex_private_key: &str) -> Result<Self> {
        let bytes = hex_to_bytes(hex_private_key)?;
        Self::new(&bytes)
    }

    fn derive_public_key(private_key: &[u8]) -> Result<Vec<u8>> {
        // Convert private key bytes to ScalarField
        let scalar = ScalarField::from_bytes_le(private_key)
            .map_err(|e| LighterError::CryptoError(format!("Invalid private key: {e:?}")))?;

        // Derive public key: G * private_key
        let public_key_point = Point::generator().mul(&scalar);

        // Encode the public key as Fp5Element
        let pub_key_encoded = public_key_point.encode();

        // Convert to bytes
        Ok(pub_key_encoded.to_bytes_le().to_vec())
    }

    /// Generate a deterministic nonce from private key and message
    /// This follows the RFC 6979 approach for deterministic signatures
    fn generate_nonce(private_key: &[u8], message: &[u8]) -> Result<ScalarField> {
        use sha2::{Digest, Sha256};

        // Combine private key and message
        let mut hasher = Sha256::new();
        hasher.update(private_key);
        hasher.update(message);
        let hash_result = hasher.finalize();

        // Convert hash to nonce (take first 40 bytes, pad if needed)
        let mut nonce_bytes = [0u8; 40];
        let copy_len = hash_result.len().min(32);
        nonce_bytes[..copy_len].copy_from_slice(&hash_result[..copy_len]);

        // Create ScalarField from the nonce bytes
        ScalarField::from_bytes_le(&nonce_bytes)
            .map_err(|e| LighterError::CryptoError(format!("Nonce generation failed: {e:?}")))
    }
}

impl Signer for PoseidonKeyManager {
    fn sign(&self, hashed_message: &[u8]) -> Result<Vec<u8>> {
        // The hashed message should be 40 bytes (5 * 8 bytes for Fp5Element)
        if hashed_message.len() != 40 {
            return Err(LighterError::CryptoError(format!(
                "Invalid hashed message length: expected 40, got {}",
                hashed_message.len()
            )));
        }

        // Generate a deterministic nonce from the message and private key
        // This ensures the same message always produces the same signature with the same key
        let nonce = Self::generate_nonce(&self.private_key, hashed_message)?;

        // Sign the message using Schnorr signature scheme
        let signature = sign_with_nonce(&self.private_key, hashed_message, &nonce.to_bytes_le())
            .map_err(|e| LighterError::CryptoError(format!("Signing failed: {e:?}")))?;

        if signature.len() != SIGNATURE_LENGTH {
            return Err(LighterError::CryptoError(format!(
                "Invalid signature length: expected {}, got {}",
                SIGNATURE_LENGTH,
                signature.len()
            )));
        }

        Ok(signature)
    }
}

impl KeyManager for PoseidonKeyManager {
    fn pub_key(&self) -> &[u8] {
        &self.public_key
    }

    fn pub_key_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        let mut result = [0u8; PUBLIC_KEY_LENGTH];
        result.copy_from_slice(&self.public_key[..PUBLIC_KEY_LENGTH]);
        result
    }

    fn prv_key_bytes(&self) -> Vec<u8> {
        self.private_key.clone()
    }
}

pub fn new_key_manager(hex_key: &str) -> Result<Box<dyn KeyManager>> {
    Ok(Box::new(PoseidonKeyManager::from_hex(hex_key)?))
}
