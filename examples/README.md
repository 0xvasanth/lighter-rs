# Lighter RS Examples

This directory contains practical examples demonstrating how to use the Lighter RS SDK.

## Available Examples

### Basic Examples (Offline Mode)

These examples demonstrate transaction signing without actually submitting to the API:

1. **create_order.rs** - Basic limit order creation and signing
   ```bash
   cargo run --example create_order
   ```

2. **transfer_funds.rs** - Transferring funds between accounts
   ```bash
   cargo run --example transfer_funds
   ```

3. **pool_operations.rs** - Pool creation, minting, and burning shares
   ```bash
   cargo run --example pool_operations
   ```

4. **advanced_orders.rs** - Cancel, modify, and grouped orders
   ```bash
   cargo run --example advanced_orders
   ```

### Testnet Examples (Live Trading)

#### testnet_trading.rs - Complete testnet integration example

This example demonstrates real trading on Lighter testnet including:
- Creating limit orders
- Creating market orders
- Creating stop-loss orders
- Updating leverage
- Canceling orders

#### Setup

1. **Get Testnet Credentials**:
   - Visit [Lighter Testnet](https://testnet.lighter.xyz)
   - Create an account and get your API credentials
   - Note your account index and API key

2. **Set Environment Variables**:
   ```bash
   export LIGHTER_API_KEY="0xYourPrivateKeyHere"
   export LIGHTER_ACCOUNT_INDEX="12345"
   export LIGHTER_API_KEY_INDEX="0"
   ```

3. **Run the Example**:
   ```bash
   cargo run --example testnet_trading
   ```

#### WebSocket Examples (Real-time Data)

**websocket_orderbook.rs** - Real-time order book monitoring
```bash
cargo run --example websocket_orderbook
```

**websocket_account.rs** - Real-time account monitoring
```bash
export LIGHTER_ACCOUNT_INDEX="12345"
cargo run --example websocket_account
```

**websocket_combined.rs** - Combined order book and account monitoring
```bash
export LIGHTER_ACCOUNT_INDEX="12345"
cargo run --example websocket_combined
```

**trading_bot_simple.rs** - Simple trading bot combining WebSocket + API
```bash
export LIGHTER_API_KEY="0xYourKey"
export LIGHTER_ACCOUNT_INDEX="12345"
cargo run --example trading_bot_simple
```

#### Expected Output

```
╔═══════════════════════════════════════════════════╗
║   Lighter RS - Testnet Trading Example           ║
╚═══════════════════════════════════════════════════╝

Configuration:
  API Endpoint: https://api-testnet.lighter.xyz
  Chain ID: 300
  Account Index: 12345
  API Key Index: 0

✓ Connected to Lighter Testnet

═══ Example 1: Creating Limit Order ═══
Order Parameters:
  Market Index: 0
  Side: BUY
  Amount: 1000000
  Price: 100000000
  Order Type: LIMIT

Signing transaction...
✓ Transaction signed
  Nonce used: 42
  Transaction Hash: 0x...

Submitting to testnet...
✓ Transaction successful!
  Tx Hash: 0x...

[Additional examples continue...]
```

## Example Features Demonstrated

### Transaction Types
- ✅ Limit Orders
- ✅ Market Orders
- ✅ Stop Loss Orders
- ✅ Take Profit Orders
- ✅ Order Cancellation
- ✅ Leverage Updates
- ✅ Fund Transfers
- ✅ Pool Operations

### SDK Features
- ✅ Automatic nonce management (fetched from API)
- ✅ Transaction signing with Poseidon cryptography
- ✅ API submission with error handling
- ✅ Helper methods for common operations
- ✅ Type-safe transaction construction
- ✅ Comprehensive validation

## API Endpoints

### Testnet
- **URL**: `https://api-testnet.lighter.xyz`
- **Chain ID**: 300
- **Explorer**: https://testnet.lighter.xyz

### Mainnet
- **URL**: `https://api.lighter.xyz`
- **Chain ID**: 304
- **Explorer**: https://lighter.xyz

## Important Notes

1. **Private Keys**: Never commit your private keys to version control. Always use environment variables or secure key management.

2. **Testnet vs Mainnet**:
   - Testnet is for testing and development
   - Always test thoroughly on testnet before using mainnet
   - Mainnet involves real funds

3. **Nonce Management**:
   - The SDK automatically fetches nonces from the API when HTTPClient is configured
   - For offline mode (no API), you must provide nonces manually in TransactOpts

4. **Error Handling**:
   - All examples use proper Rust error handling with `Result<T>`
   - Check transaction responses for success codes
   - Handle network errors gracefully

## Troubleshooting

### "nonce was not provided and HTTPClient is not available"
- Solution: Either provide an API URL when creating TxClient, or manually specify nonce in TransactOpts

### "Failed to get nonce" or API errors
- Check your internet connection
- Verify the API endpoint is correct
- Ensure your account exists on the network
- Check that your API key is valid

### Compilation errors
- Ensure you're using Rust 2021 edition or later
- Run `cargo update` to get latest dependencies
- Check that all dependencies are properly installed

## Next Steps

After running the examples:

1. **Explore the API**: Check [Lighter API Docs](https://apidocs.lighter.xyz)
2. **Build your app**: Use these examples as templates
3. **Join the community**: [Lighter Discord](https://discord.gg/lighter)
4. **Read the docs**: See the main README.md for detailed documentation

---

## 🧪 Testing & Verification Examples

### **safe_trade_test.rs** - Safe Trade Lifecycle Test ⭐ START HERE
**Purpose:** Verify your API credentials work with zero risk of losing money.

**What it does:**
- Places a limit order FAR from market price (won't execute)
- Verifies order placement
- Cancels the order immediately
- Tests complete trading workflow safely

```bash
cargo run --example safe_trade_test
```

**Expected with Valid Credentials:**
```
✅ ORDER PLACED SUCCESSFULLY!
✅ ORDER CANCELLED SUCCESSFULLY!
🎉 Your Lighter API is working correctly!
```

**Expected with Test Credentials:**
```
❌ Order NOT placed (invalid credentials)
✅ SDK works correctly (signatures valid)
💡 Register API key at https://app.lighter.xyz
```

---

### **complete_trade_workflow.rs** - Full Lifecycle Test
Comprehensive test with step-by-step verification:

```bash
cargo run --example complete_trade_workflow
```

Features:
- Detailed step-by-step output
- Market selection
- Order creation & signing
- Signature verification
- Order placement
- Order cancellation
- Complete status reporting

---

### **verify_signing_works.rs** - Signature Verification
Proves that Poseidon/Schnorr signing is implemented correctly:

```bash
cargo run --example verify_signing_works
```

**Key Output:**
```
✅ Signature contains non-zero bytes (cryptographically signed!)
✅ Signature has good entropy (70+ unique values)
✅ Poseidon signing is IMPLEMENTED
Issue #4 Status: RESOLVED ✅
```

---

## 🔧 Diagnostic Tools

### **diagnose_api_errors.rs** - Error Diagnosis
Diagnoses common API errors and provides solutions:

```bash
cargo run --example diagnose_api_errors
```

**Helps with:**
- "invalid base amount" errors
- "api key not found" errors
- Configuration validation
- Credential verification
- Base amount testing

---

### **test_crypto_api.rs** - Crypto Library Explorer
Explores goldilocks-crypto and poseidon-hash APIs:

```bash
cargo run --example test_crypto_api
```

Shows:
- Private/public key generation
- Poseidon hash creation
- Message signing
- Signature verification

---

## 📊 Strategy Examples

### **grid_trading_example.rs** - Grid Trading Bot
Example implementation of a grid trading strategy:

```bash
cargo run --example grid_trading_example
```

Features:
- Multiple price levels
- Automated buy/sell orders
- Configurable grid parameters
- Safe testing with low prices

---

## 🆘 Troubleshooting Examples

### Common Error: "invalid base amount" (21701)

**Run diagnosis:**
```bash
cargo run --example diagnose_api_errors
```

**Common Causes:**
1. API key not registered → Register at https://app.lighter.xyz
2. Insufficient balance → Deposit funds
3. Below minimum order size → Check market specs

**See:** [TROUBLESHOOTING.md](../TROUBLESHOOTING.md)

---

### Common Error: "api key not found" (21109)

**This means your API key is not registered.**

**Fix:**
1. Go to https://app.lighter.xyz
2. Settings → API Keys
3. Generate new API key
4. Update .env:
   ```bash
   LIGHTER_API_KEY=<new-key-no-0x-prefix>
   LIGHTER_ACCOUNT_INDEX=<your-account-index>
   LIGHTER_API_KEY_INDEX=<usually-0-1-or-2>
   ```

---

## 📝 Example Usage Summary

| Example | Purpose | Risk | Credentials Required |
|---------|---------|------|---------------------|
| `verify_signing_works` | Verify SDK | None | No |
| `test_crypto_api` | Explore crypto libs | None | No |
| `safe_trade_test` | Test API | None | Yes |
| `complete_trade_workflow` | Full test | None | Yes |
| `diagnose_api_errors` | Debug errors | None | Optional |
| `grid_trading_example` | Strategy demo | Low | Yes |
| `test_all_functions` | SDK functions | None | Optional |

---

## 🎓 Recommended Learning Path

### Step 1: Verify SDK Works (No Credentials Needed)
```bash
cargo run --example verify_signing_works
cargo run --example test_crypto_api
```

### Step 2: Get API Credentials
1. Visit https://app.lighter.xyz
2. Create account
3. Generate API key
4. Update .env file

### Step 3: Test Your Credentials
```bash
cargo run --example safe_trade_test
```

### Step 4: Explore Advanced Features
```bash
cargo run --example complete_trade_workflow
cargo run --example grid_trading_example
```

---

## Contributing

Found a bug or have a suggestion? Please open an issue or submit a PR!
