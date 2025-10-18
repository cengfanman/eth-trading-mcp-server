# Quick Start Guide

## 1. Setup Environment

```bash
# Copy the example environment file
cp .env.example .env

# Edit .env with your actual credentials
# Required:
#   - RPC_URL: Get from Infura (https://infura.io/) or Alchemy (https://www.alchemy.com/)
#   - PRIVATE_KEY: Any Ethereum private key (transactions are NOT executed)

# Example:
export RPC_URL="https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY"
export PRIVATE_KEY="0xYOUR_PRIVATE_KEY_HERE"
```

## 2. Build the Project

```bash
cargo build --release
```

## 3. Run Tests

```bash
# Run unit tests (no RPC needed)
cargo test

# Expected output: test result: ok. 19 passed
```

## 4. Verify Your Setup (Optional but Recommended)

After configuring your `.env` file:

```bash
# Check your wallet address
./target/release/show-address

# Test real Uniswap V2 integration (proves everything works!)
./target/release/test-swap
```

**Expected output from test-swap:**
```
🚀 Testing Real Uniswap V2 Swap Calculation
...
✅ Response from Uniswap:
  Estimated output: 0.259... WETH
✅ All Uniswap V2 integration working correctly!
✅ Using REAL mainnet data
```

This proves:
- ✅ Your RPC connection works
- ✅ Real blockchain data is being fetched
- ✅ Uniswap integration is functional
- ✅ All calculations are correct

## 5. Run the MCP Server

```bash
# Start the server (listens on stdin/stdout)
# Environment variables are automatically loaded from .env
./target/release/eth-mcp-server
```

The server will listen for MCP JSON-RPC requests on stdin and respond on stdout.

## 6. Example Commands

### Initialize the Server

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | cargo run --release
```

### List Available Tools

```bash
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | cargo run --release
```

### Get ETH Balance (Vitalik's address)

```bash
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_balance","arguments":{"wallet":"0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"}}}' | cargo run --release
```

### Get Token Price (USDC in ETH)

```bash
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_token_price","arguments":{"token":"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48","base":"eth"}}}' | cargo run --release
```

## 7. Common Token Addresses (Ethereum Mainnet)

- **WETH**: `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2`
- **USDC**: `0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48`
- **DAI**: `0x6B175474E89094C44Da98b954EedeAC495271d0F`
- **USDT**: `0xdAC17F958D2ee523a2206206994597C13D831ec7`

## 8. Testing with Test Script

A convenience script is provided for quick testing:

```bash
./test_mcp.sh
```

This script runs several example queries and displays the formatted results.

## 9. Troubleshooting

### "Failed to connect to provider"
- Check that RPC_URL is set correctly
- Verify your API key is valid
- Ensure you have internet connectivity

### "Failed to parse private key"
- Ensure PRIVATE_KEY starts with "0x"
- Verify it's a valid 64-character hex string (plus "0x" prefix)

### "Pair does not exist"
- The token pair may not have a Uniswap V2 pool
- Try swapping via WETH (the router does this automatically for most tokens)

### Tests failing
- Unit tests should always pass (no network needed)
- Integration tests require RPC_URL to be set
- Run integration tests with: `cargo test -- --ignored`

## Next Steps

1. Read the full [README.md](README.md) for detailed documentation
2. Review the [MCP protocol documentation](https://modelcontextprotocol.io/)
3. Integrate with your AI agent or MCP client
4. Explore the source code to understand the implementation

## Security Reminder

⚠️ **IMPORTANT**: This server is for **simulation only**. No transactions are broadcast to the blockchain. However:
- Never expose your private key
- Never commit `.env` to version control
- Use a test/development wallet, not your main wallet
- Always review code before running with real credentials
