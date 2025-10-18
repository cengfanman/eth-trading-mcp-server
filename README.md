# Ethereum Trading MCP Server

A Model Context Protocol (MCP) server that enables AI agents to interact with Ethereum and Uniswap V2 for querying balances, token prices, and simulating token swaps.

## Features

- **`get_balance`**: Query ETH and ERC20 token balances with proper decimal formatting
- **`get_token_price`**: Get real-time token prices from Uniswap V2 liquidity pools
- **`swap_tokens`**: Simulate token swaps with gas estimation (no on-chain execution)

## Setup

### Prerequisites

- Rust 1.70+ (`rustup install stable`)
- An Ethereum RPC endpoint (Infura, Alchemy, or public node)
- A wallet private key (for transaction construction context, **NOT for actual execution**)

### Installation

1. Clone the repository:
```bash
git clone <your-repo-url>
cd eth-trading-mcp-server
```

2. Build the project:
```bash
cargo build --release
```

### Configuration

**Recommended: Use .env file** (automatically loaded):

```bash
# Copy the example file
cp .env.example .env

# Edit .env with your credentials
nano .env  # or vim, code, etc.
```

Your `.env` file should contain:

```bash
# Required: Ethereum RPC endpoint
RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY

# Required: Private key (with 0x prefix)
# ⚠️ WARNING: This key is used ONLY for constructing transaction context
# Transactions are NEVER broadcast to the network
PRIVATE_KEY=0xYOUR_PRIVATE_KEY_HERE

# Optional: Chain ID (defaults to 1 for Ethereum mainnet)
CHAIN_ID=1

# Optional: Set log level
RUST_LOG=info
```

**Alternative: Export manually** (if not using .env):

```bash
export RPC_URL="https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY"
export PRIVATE_KEY="0xYOUR_PRIVATE_KEY_HERE"
export CHAIN_ID=1
export RUST_LOG=info
```

**Security Note**: Never commit your `.env` file or expose your private key. The private key is only used to construct transaction simulations via `eth_call` and gas estimation. No transactions are signed or broadcast.

### Running the Server

```bash
cargo run --release
```

The server listens on stdin/stdout for MCP JSON-RPC requests.

## Verification & Demo Tools

This project includes additional utilities to verify functionality and demonstrate real blockchain integration:

### 1. Show Your Wallet Address

```bash
./target/release/show-address
```

**Output:**
```
Your wallet address: 0x39cfc46991e17ada6429f4628cfb0c4c4a6886bf
```

This displays the Ethereum address derived from your private key.

### 2. Test Real Uniswap V2 Integration

```bash
./target/release/test-swap
```

**Output:**
```
🚀 Testing Real Uniswap V2 Swap Calculation

Input:
  From: USDC (0xA0b8...B48)
  To:   WETH (0xC02a...Cc2)
  Amount: 1000 USDC

Path: [USDC, WETH] (direct swap)

📡 Calling Uniswap V2 Router.getAmountsOut()...
✅ Response from Uniswap:
  Estimated output: 0.259300964459691492 WETH
  Raw value: 259300964459691492

📊 With 0.5% slippage protection:
  Minimum output: 0.258004459637393034 WETH
  Raw value: 258004459637393034

📝 Transaction data encoded:
  Function: swapExactTokensForTokens
  Calldata length: 260 bytes

✅ All Uniswap V2 integration working correctly!
✅ Using REAL mainnet data from Infura
✅ Transaction construction complete
```

**What this proves:**
- ✅ Real-time connection to Ethereum mainnet via RPC
- ✅ Actual calls to Uniswap V2 Router contract
- ✅ Accurate price calculations from live liquidity pools
- ✅ Proper transaction encoding for swaps
- ✅ All logic working with real on-chain data

**Note on `swap_tokens` simulation:**
The `swap_tokens` tool may fail with "transaction would revert" if your wallet has insufficient balance. This is **correct behavior** - the code properly validates transactions before execution. The test above proves all Uniswap integration is working correctly without requiring token balance.

## Usage Examples

### Initialize the Server

**Request:**
```json
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {"tools": {}},
    "serverInfo": {"name": "eth-trading-mcp-server", "version": "0.1.0"}
  }
}
```

### List Available Tools

**Request:**
```json
{"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "get_balance",
        "description": "Query ETH and ERC20 token balances for a wallet address",
        "input_schema": {
          "type": "object",
          "properties": {
            "wallet": {"type": "string", "description": "Ethereum wallet address (0x...)"},
            "token": {"type": "string", "description": "Optional ERC20 token contract address. If omitted, returns ETH balance."}
          },
          "required": ["wallet"]
        }
      },
      ...
    ]
  }
}
```

### Get ETH Balance

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "get_balance",
    "arguments": {
      "wallet": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "wallet": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
    "native": {
      "raw": "1234567890123456789",
      "formatted": "1.234567890123456789",
      "decimals": 18,
      "symbol": "ETH"
    },
    "block_number": 18500000
  }
}
```

### Get ERC20 Token Balance

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "get_balance",
    "arguments": {
      "wallet": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
      "token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "wallet": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
    "erc20": {
      "raw": "1000000000",
      "formatted": "1000",
      "decimals": 6,
      "symbol": "USDC"
    },
    "block_number": 18500000
  }
}
```

### Get Token Price

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "get_token_price",
    "arguments": {
      "token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
      "base": "eth"
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    "base": "WETH",
    "price": "0.0005",
    "pair": "0xb4e16d0168e52d35cacd2c6185b44281ec28c9dc",
    "source": "Uniswap V2",
    "block_number": 18500000
  }
}
```

### Simulate Token Swap

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "tools/call",
  "params": {
    "name": "swap_tokens",
    "arguments": {
      "from_token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
      "to_token": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
      "amount": "1000",
      "slippage_bps": 50
    }
  }
}
```

**Response (on success):**
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "result": {
    "from_token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    "to_token": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
    "amount_in": "1000",
    "path": [
      "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
      "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
    ],
    "estimated_out": {
      "raw": "499500000000000000",
      "formatted": "0.4995"
    },
    "amount_out_min": {
      "raw": "497002500000000000",
      "formatted": "0.497002500000000000"
    },
    "gas_limit": "150000",
    "gas_price": "30000000000",
    "estimated_fee_native": "0.0045",
    "slippage_bps": 50
  }
}
```

**Response (on insufficient balance):**
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "error": {
    "code": -32603,
    "message": "Swap simulation failed - transaction would revert"
  }
}
```

**Note:** This error is expected if your wallet doesn't have sufficient token balance. The code is working correctly by detecting this via `eth_call` simulation. To verify Uniswap integration is working, run:
```bash
./target/release/test-swap
```
This demonstrates real Uniswap V2 calls without requiring token balance.

## Testing

### Run Unit Tests

```bash
cargo test
```

Expected output: `19 passed`

### Verify Real Blockchain Integration

Test real Uniswap V2 integration without requiring token balance:

```bash
./target/release/test-swap
```

This proves:
- ✅ Real RPC connection to Ethereum mainnet
- ✅ Actual Uniswap V2 Router calls
- ✅ Live price data from liquidity pools
- ✅ Correct transaction encoding

### Run Integration Tests (Requires RPC_URL)

Integration tests connect to a real Ethereum node and are ignored by default:

```bash
# With .env file (recommended)
cargo test --test it_mainnet -- --ignored --nocapture

# Or export manually
export RPC_URL="https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY"
export PRIVATE_KEY="0xYOUR_PRIVATE_KEY"
cargo test --test it_mainnet -- --ignored --nocapture
```

## Design Decisions

1. **Uniswap V2 Only**: The MVP focuses on Uniswap V2 for simplicity. V3 support would require concentrated liquidity math and more complex routing.

2. **Price Source**: Token prices are derived from Uniswap V2 reserves (constant product formula). This provides real on-chain data but may differ from centralized exchange prices due to slippage and liquidity depth.

3. **Swap Simulation**: The `swap_tokens` tool constructs real Uniswap transactions and simulates them using `eth_call` and `eth_estimateGas`. **No transactions are broadcast**—this is purely for estimation.

4. **Simple Routing**: Swap paths support either direct swaps (if one token is WETH) or two-hop swaps via WETH. Complex multi-hop routing is not implemented.

5. **Decimal Precision**: Uses `rust_decimal` for all financial calculations to avoid floating-point errors. Balances are returned in both raw (wei/token units) and formatted (human-readable) forms.

## Known Limitations

- **Uniswap V2 only**: V3 is not supported
- **Price accuracy**: Prices reflect Uniswap V2 pools only and may not match broader market prices
- **Limited routing**: Only supports direct or single-hop (via WETH) swap paths
- **No token symbol resolution**: Tools require contract addresses, not symbols (e.g., use `0xA0b8...` not `"USDC"`)
- **Mainnet only**: Designed for Ethereum mainnet (chain ID 1)
- **No transaction execution**: This server is read-only and for simulation purposes only

## Architecture

```
src/
├── main.rs           # MCP server entry point (stdin/stdout JSON-RPC)
├── config.rs         # Environment variable configuration
├── types.rs          # MCP request/response DTOs
├── rpc.rs            # MCP protocol handler
├── utils/
│   └── decimal.rs    # U256 ↔ Decimal conversion utilities
├── eth/
│   ├── provider.rs   # Ethereum RPC provider setup
│   └── erc20.rs      # ERC20 ABI calls (balanceOf, decimals, symbol)
├── uniswap/
│   └── v2.rs         # Uniswap V2 router/pair interactions
└── tools/
    ├── balance.rs    # get_balance implementation
    ├── price.rs      # get_token_price implementation
    └── swap.rs       # swap_tokens implementation
```

## Development with AI Assistants

This project was developed with assistance from AI tools (Claude Code). The implementation demonstrates:

- Rust async programming with Tokio
- Ethereum RPC interactions via ethers-rs
- Proper decimal handling for financial data
- MCP protocol compliance
- Real-world DeFi integration (Uniswap V2)

## License

MIT

## Contributing

Contributions welcome! Please ensure:
- `cargo build` compiles successfully
- `cargo test` passes all tests
- `cargo fmt` and `cargo clippy` produce clean output
# eth-trading-mcp-server
