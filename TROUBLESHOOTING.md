# Troubleshooting Guide for Lighter RS SDK

## Common API Errors and Solutions

### Error 1: `{"code":21701,"message":"invalid base amount"}`

**What it means:**
- The `base_amount` parameter doesn't meet the market's requirements
- This could be due to: incorrect decimals, below minimum order size, or signature validation failure

**Common Causes:**

1. **Incorrect Decimal Encoding**
   - `base_amount` represents the order size in the market's base currency
   - For most markets, use 6 decimals
   - Example: $10 = `10_000_000` (10 * 10^6)

2. **Below Minimum Order Size**
   - Each market has a minimum order size
   - Check market specifications before trading
   - Typical minimums: $10-$100 depending on the asset

3. **Invalid Signature** (misleading error message)
   - The API may return "invalid base amount" when the signature is invalid
   - This was the case before we implemented Poseidon signing
   - ✅ Now fixed with proper Schnorr signatures

**Solutions:**

```rust
// ✅ CORRECT: Using proper decimals
let base_amount = 10_000_000i64; // $10 with 6 decimals

// ❌ WRONG: Not using decimals
let base_amount = 10i64; // Only $0.00001

// Check your order:
let order = tx_client.create_limit_order(
    market_index,
    client_order_index,
    10_000_000,     // $10 (base_amount with 6 decimals)
    3_000_000_000,  // $3000 (price with 6 decimals)
    0,              // BUY
    false,
    None,
).await?;
```

**If error persists:**
- Verify your API key is valid and registered
- Check you have sufficient account balance
- Try a different market with lower minimums
- Contact Lighter support for market-specific requirements

---

### Error 2: `{"code":21109,"message":"api key not found"}`

**What it means:**
- Your API key is not recognized by the Lighter system
- The `account_index` or `api_key_index` doesn't match your registration

**Common Causes:**

1. **API Key Not Registered**
   - The API key hasn't been created at https://app.lighter.xyz
   - Using a test/example key instead of a real one

2. **Incorrect Account/API Key Indices**
   - `account_index` doesn't match your account
   - `api_key_index` is wrong (usually 0, 1, or 2)

3. **Wrong API Key Format**
   - API key should be 40-byte hex (80 characters)
   - Should NOT have '0x' prefix
   - Should NOT have spaces or other characters

**Solutions:**

1. **Verify Your API Key:**
   ```bash
   # Check .env file
   cat .env | grep LIGHTER_API_KEY

   # Should be 80 characters, no 0x prefix
   # Example: 05fca652fa2030f7be4f12ae3af7c8b8e373af4567c440eb07d3ec5b838787cbb8cfc76658b32c40
   ```

2. **Get Your Correct Credentials:**
   - Go to https://app.lighter.xyz
   - Navigate to Account Settings > API Keys
   - Create a new API key if needed
   - Copy the exact `account_index` and `api_key_index`

3. **Update Your `.env` File:**
   ```bash
   LIGHTER_API_KEY=<your-40-byte-hex-key>  # No 0x prefix!
   LIGHTER_ACCOUNT_INDEX=<your-account-index>
   LIGHTER_API_KEY_INDEX=<your-api-key-index>  # Usually 0, 1, or 2
   LIGHTER_CHAIN_ID=304  # 304 for mainnet, 300 for testnet
   LIGHTER_API_URL=https://mainnet.zklighter.elliot.ai
   ```

4. **Test Your Configuration:**
   ```bash
   cargo run --example diagnose_api_errors
   ```

---

### Error 3: `{"code":20001,"message":"invalid param: field \"tx_type\" is not set"}`

**Status:** ✅ **RESOLVED** (Fixed in commit 5297851)

**What it was:**
- The SDK was sending JSON instead of form data
- The API couldn't parse the `tx_type` field

**Solution:**
- Update to the latest version of the SDK
- The fix changes request format from JSON to form-encoded data

---

### Error 4: Signature is All Zeros

**Status:** ✅ **RESOLVED** (Fixed in commit 9696201)

**What it was:**
- The Poseidon signing was not implemented (stub)
- All signatures were `[0,0,0,...,0]`
- Orders were rejected with various cryptic errors

**Solution:**
- Update to the latest version with Poseidon/Schnorr signing
- Verify signatures are non-zero:
  ```bash
  cargo run --example verify_signing_works
  ```

---

## General Debugging Steps

### 1. Check SDK Version
```bash
git log --oneline | head -5
```

Should include:
- `9696201` - Implement Poseidon/Schnorr signing
- `5297851` - Fix sendTx API: Use form data instead of JSON

### 2. Verify Configuration
```bash
cargo run --example diagnose_api_errors
```

This will check:
- ✅ API key format
- ✅ Client initialization
- ✅ Various base_amount values
- ✅ Configuration warnings

### 3. Test Signing
```bash
cargo run --example verify_signing_works
```

Should show:
- ✅ Signatures with non-zero bytes
- ✅ High entropy (70+ unique values)
- ✅ Proper signature length (80 bytes)

### 4. Check Your Balance
- Go to https://app.lighter.xyz
- Verify you have sufficient funds for trading
- Check collateral and margin requirements

---

## Working Example

Here's a complete working example:

```rust
use dotenv::dotenv;
use lighter_rs::client::TxClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Load from .env
    let client = TxClient::new(
        &std::env::var("LIGHTER_API_URL")?,
        &std::env::var("LIGHTER_API_KEY")?,
        std::env::var("LIGHTER_ACCOUNT_INDEX")?.parse()?,
        std::env::var("LIGHTER_API_KEY_INDEX")?.parse()?,
        304, // Mainnet
    )?;

    // Create order with proper decimals
    let order = client.create_limit_order(
        0,                      // Market 0 (ETH)
        chrono::Utc::now().timestamp_millis(),
        10_000_000,             // $10 base amount (6 decimals)
        3_000_000_000,          // $3000 price (6 decimals)
        0,                      // BUY
        false,                  // Not reduce-only
        None,
    ).await?;

    // Submit
    match client.send_transaction(&order).await {
        Ok(response) if response.code == 200 => {
            println!("✅ Order submitted!");
            println!("Tx Hash: {:?}", response.tx_hash);
        }
        Ok(response) => {
            println!("⚠️ API returned: {:?}", response.message);
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }

    Ok(())
}
```

---

## Still Having Issues?

1. **Check the Examples:**
   ```bash
   ls examples/
   cargo run --example test_all_functions
   ```

2. **Review Recent Commits:**
   ```bash
   git log --oneline
   ```

3. **Get Help:**
   - GitHub Issues: https://github.com/0xvasanth/lighter-rs/issues
   - Lighter Discord: https://discord.gg/lighter
   - API Docs: https://apidocs.lighter.xyz

4. **Verify Your Setup:**
   - Is your API key registered at https://app.lighter.xyz?
   - Do you have sufficient balance?
   - Are you using the correct market index?
   - Are your decimals correct (usually 6)?

---

## Quick Reference

### Decimal Encoding

| Value | With 6 Decimals | Code |
|-------|-----------------|------|
| $0.10 | 100,000 | `100_000` |
| $1.00 | 1,000,000 | `1_000_000` |
| $10.00 | 10,000,000 | `10_000_000` |
| $100.00 | 100,000,000 | `100_000_000` |
| $1000.00 | 1,000,000,000 | `1_000_000_000` |

### Environment Variables

```bash
# Required
LIGHTER_API_KEY=<40-byte-hex-without-0x-prefix>
LIGHTER_ACCOUNT_INDEX=<your-account-number>
LIGHTER_API_KEY_INDEX=<usually-0-1-or-2>
LIGHTER_API_URL=https://mainnet.zklighter.elliot.ai
LIGHTER_CHAIN_ID=304

# Optional
RUST_LOG=info
ENABLE_LIVE_TRADING=true
```

### Common Market Indices

- `0` = ETH
- `1` = BTC
- `98` = USDJPY
- (Check Lighter docs for complete list)
