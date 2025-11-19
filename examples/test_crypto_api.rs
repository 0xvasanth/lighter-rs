/// Test to explore the goldilocks-crypto and poseidon-hash APIs
use goldilocks_crypto::*;
use poseidon_hash::*;

fn main() {
    println!("=== Testing Goldilocks Crypto API ===\n");

    // Test 1: Generate a private key
    println!("1. Generating private key...");
    let private_key = ScalarField::sample_crypto();
    println!("   Private key generated: {:?}", private_key.to_bytes_le());

    // Test 2: Derive public key
    println!("\n2. Deriving public key...");
    let public_key = Point::generator().mul(&private_key);
    println!("   Public key: {:?}", public_key.encode());

    // Test 3: Hash a message with Poseidon
    println!("\n3. Hashing message with Poseidon...");
    let message_elements = vec![
        Goldilocks::from(12345u64),
        Goldilocks::from(67890u64),
    ];
    let hash = hash_to_quintic_extension(&message_elements);
    println!("   Hash result: {:?}", hash);

    // Test 4: Sign a message
    println!("\n4. Signing message...");
    let message = b"Hello, Lighter Protocol!";
    let nonce = ScalarField::sample_crypto();

    match sign_with_nonce(&private_key.to_bytes_le(), message, &nonce.to_bytes_le()) {
        Ok(signature) => {
            println!("   Signature generated: {} bytes", signature.len());
            println!("   Signature (first 20 bytes): {:?}", &signature[..20.min(signature.len())]);

            // Test 5: Verify the signature
            println!("\n5. Verifying signature...");
            let pub_key_encoded = public_key.encode();
            let pub_key_bytes = pub_key_encoded.to_bytes_le();
            match verify_signature(&signature, message, &pub_key_bytes) {
                Ok(is_valid) => println!("   Signature valid: {}", is_valid),
                Err(e) => println!("   Verification error: {:?}", e),
            }
        }
        Err(e) => {
            println!("   Error signing: {:?}", e);
        }
    }

    println!("\n=== API Exploration Complete ===");
}
