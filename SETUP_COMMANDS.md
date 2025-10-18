# Setup Commands

## Prerequisites
Ensure you have:
- Rust 1.70+ installed (`rustup install stable`)
- An Ethereum RPC endpoint (Infura, Alchemy, or public node)
- A wallet private key (for simulation context only)

## Step-by-Step Setup

### 1. Navigate to Project Directory
```bash
cd /Users/yingxuegu/Documents/project/eth-trading-mcp-server
```

### 2. Set Environment Variables

**Option A: Export directly (temporary)**
```bash
export RPC_URL="https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY"
export PRIVATE_KEY="0xYOUR_PRIVATE_KEY_HERE"
export CHAIN_ID=1
export RUST_LOG=info
```

**Option B: Create .env file (recommended)**
```bash
# Copy example
cp .env.example .env

# Edit .env with your favorite editor
nano .env
# OR
vim .env
# OR
code .env
```

Then source it:
```bash
# Note: You may need a tool like `direnv` or manually export
# For manual export:
export $(cat .env | grep -v '^#' | xargs)
```

### 3. Build the Project
```bash
cargo build --release
```

Expected output:
```
   Compiling eth-trading-mcp-server v0.1.0
   Finished `release` profile [optimized] target(s) in X.XXs
```

### 4. Run Tests
```bash
# Run all unit tests (no RPC needed)
cargo test

# Run integration tests (requires RPC_URL)
cargo test --test it_mainnet -- --ignored --nocapture
```

Expected output:
```
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### 5. Run the MCP Server
```bash
cargo run --release
```

The server will start and listen on stdin/stdout for JSON-RPC requests.

### 6. Test the Server (in another terminal)

**Initialize:**
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | \
  cargo run --release 2>/dev/null | jq .
```

**List Tools:**
```bash
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | \
  cargo run --release 2>/dev/null | jq .
```

**Get ETH Balance (Vitalik's address):**
```bash
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_balance","arguments":{"wallet":"0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"}}}' | \
  cargo run --release 2>/dev/null | jq .
```

**Get Token Price (USDC in ETH):**
```bash
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_token_price","arguments":{"token":"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48","base":"eth"}}}' | \
  cargo run --release 2>/dev/null | jq .
```

**Simulate Swap (1000 USDC -> WETH):**
```bash
echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"swap_tokens","arguments":{"from_token":"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48","to_token":"0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2","amount":"1000","slippage_bps":50}}}' | \
  cargo run --release 2>/dev/null | jq .
```

### 7. Use the Convenience Test Script
```bash
./test_mcp.sh
```

## Common Token Addresses (Ethereum Mainnet)

```bash
# Wrapped ETH
WETH=0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2

# Stablecoins
USDC=0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
DAI=0x6B175474E89094C44Da98b954EedeAC495271d0F
USDT=0xdAC17F958D2ee523a2206206994597C13D831ec7

# DeFi Tokens
UNI=0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984
AAVE=0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9
```

## Troubleshooting

### Issue: "RPC_URL not set"
```bash
# Check if environment variable is set
echo $RPC_URL

# If empty, export it
export RPC_URL="https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY"
```

### Issue: "Failed to parse private key"
```bash
# Ensure private key starts with 0x and is 66 characters total
echo $PRIVATE_KEY | wc -c  # Should output 67 (66 + newline)

# Example of valid format
export PRIVATE_KEY="0x1234567890123456789012345678901234567890123456789012345678901234"
```

### Issue: "Pair does not exist"
- The token may not have a Uniswap V2 pool
- Try using well-known tokens (WETH, USDC, DAI)
- Check token address on Etherscan

### Issue: Compilation errors
```bash
# Update Rust
rustup update stable

# Clean and rebuild
cargo clean
cargo build --release
```

## Optional: Git Setup

```bash
# Initialize git repository
git init

# Add all files
git add .

# Create initial commit
git commit -m "Initial commit: Ethereum Trading MCP Server"

# Add remote (replace with your GitHub repo URL)
git remote add origin https://github.com/YOUR_USERNAME/eth-trading-mcp-server.git

# Push to GitHub
git push -u origin main
```

## Optional: Docker Setup

Create a `Dockerfile`:
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/eth-mcp-server /usr/local/bin/
ENV RUST_LOG=info
ENTRYPOINT ["eth-mcp-server"]
```

Build and run:
```bash
docker build -t eth-mcp-server .
docker run -e RPC_URL="..." -e PRIVATE_KEY="..." eth-mcp-server
```

## Getting API Keys

### Infura
1. Visit https://infura.io/
2. Sign up for free account
3. Create new project
4. Copy the mainnet HTTPS endpoint
5. Use as: `https://mainnet.infura.io/v3/YOUR_PROJECT_ID`

### Alchemy
1. Visit https://www.alchemy.com/
2. Sign up for free account
3. Create new app (select Ethereum Mainnet)
4. Copy the HTTPS endpoint
5. Use as: `https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY`

## Security Reminders

⚠️ **NEVER commit your .env file to git**
⚠️ **Use a test/development wallet, not your main wallet**
⚠️ **This server does NOT execute transactions - it's simulation only**
⚠️ **Keep your private key secure and never share it**

---

For more details, see:
- [README.md](README.md) - Full documentation
- [QUICKSTART.md](QUICKSTART.md) - Quick start guide
- [PROJECT_SUMMARY.md](PROJECT_SUMMARY.md) - Implementation summary
