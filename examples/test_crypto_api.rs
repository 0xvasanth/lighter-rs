/// Test to explore the goldilocks-crypto and poseidon-hash APIs
use goldilocks_crypto::*;
use poseidon_hash::*;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("=== Testing Goldilocks Crypto API ===\n");

    // Test 1: Generate a private key
    tracing::info!("1. Generating private key...");
    let private_key = ScalarField::sample_crypto();
    tracing::info!("   Private key generated: {:?}", private_key.to_bytes_le());

    // Test 2: Derive public key
    tracing::info!("\n2. Deriving public key...");
    let public_key = Point::generator().mul(&private_key);
    tracing::info!("   Public key: {:?}", public_key.encode());

    // Test 3: Hash a message with Poseidon
    tracing::info!("\n3. Hashing message with Poseidon...");
    let message_elements = vec![Goldilocks::from(12345u64), Goldilocks::from(67890u64)];
    let hash = hash_to_quintic_extension(&message_elements);
    tracing::info!("   Hash result: {:?}", hash);

    // Test 4: Sign a message
    tracing::info!("\n4. Signing message...");
    let message = b"Hello, Lighter Protocol!";
    let nonce = ScalarField::sample_crypto();

    match sign_with_nonce(&private_key.to_bytes_le(), message, &nonce.to_bytes_le()) {
        Ok(signature) => {
            tracing::info!("   Signature generated: {} bytes", signature.len());
            tracing::info!(
                "   Signature (first 20 bytes): {:?}",
                &signature[..20.min(signature.len())]
            );

            // Test 5: Verify the signature
            tracing::info!("\n5. Verifying signature...");
            let pub_key_encoded = public_key.encode();
            let pub_key_bytes = pub_key_encoded.to_bytes_le();
            match verify_signature(&signature, message, &pub_key_bytes) {
                Ok(is_valid) => tracing::info!("   Signature valid: {}", is_valid),
                Err(e) => tracing::info!("   Verification error: {:?}", e),
            }
        }
        Err(e) => {
            tracing::info!("   Error signing: {:?}", e);
        }
    }

    tracing::info!("\n=== API Exploration Complete ===");
}
