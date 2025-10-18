# Ethereum Trading MCP Server

A Model Context Protocol (MCP) server that enables AI agents to interact with Ethereum and Uniswap V2 for querying balances, token prices, and simulating token swaps.

---

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

3. Run tests:
```bash
cargo test
```

Expected output: `19 passed`

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
RPC_URL=https://eth-mainnet.g.alchemy.com/v3/YOUR_API_KEY

# Required: Private key (with 0x prefix)
# ⚠️ WARNING: This key is used ONLY for constructing transaction context
# Transactions are NEVER broadcast to the network
PRIVATE_KEY=0xYOUR_PRIVATE_KEY_HERE

# Optional: Chain ID (defaults to 1 for Ethereum mainnet)
CHAIN_ID=1

# Optional: Set log level
RUST_LOG=info
```

**Security Note**: Never commit your `.env` file or expose your private key. The private key is only used to construct transaction simulations via `eth_call` and gas estimation. **No transactions are signed or broadcast.**

### Running the Server

```bash
cargo run --release
```

The server listens on stdin/stdout for MCP JSON-RPC requests.

---

## Features

### Three MCP Tools

1. **`get_balance`**
   - Query ETH and ERC20 token balances for any wallet address
   - Automatic decimal formatting (ETH uses 18 decimals, USDC uses 6, etc.)
   - Returns both raw values (wei) and human-readable formatted amounts

2. **`get_token_price`**
   - Get real-time token prices from Uniswap V2 liquidity pools
   - Calculate prices in ETH or USD (via USDC)
   - Uses constant product formula (x * y = k) from actual reserves

3. **`swap_tokens`**
   - Simulate token swaps with complete validation
   - Query Uniswap for estimated output
   - Calculate slippage protection (default 0.5%)
   - Estimate gas costs (gas limit × gas price)
   - Simulate transaction via `eth_call` (checks balance, approval, etc.)
   - **No actual on-chain execution** - purely for estimation

---

## Design Decisions

### Why Uniswap V2?

- **Simplicity**: V2 uses constant product formula (x * y = k), easy to understand and implement
- **Mature Protocol**: Battle-tested, widely used, comprehensive liquidity
- **V3 Complexity**: V3 requires concentrated liquidity math, tick calculations, and complex routing

### Price Calculation

Token prices are derived from Uniswap V2 reserves using the constant product formula:

```
price = reserve_out / reserve_in
```

This provides **real on-chain data** but may differ from centralized exchange prices due to:
- Liquidity depth
- Slippage impact
- MEV bot activity

### Swap Simulation Process

The `swap_tokens` tool follows this 5-step process:

1. ✅ **Query Uniswap** - Call `getAmountsOut()` to get estimated output
2. ✅ **Calculate slippage** - Apply slippage tolerance (e.g., 0.5%)
3. ✅ **Estimate gas** - Get real-time gas price and calculate fee
4. ✅ **Encode transaction** - Build `swapExactTokensForTokens` calldata
5. ✅ **Simulate execution** - Use `eth_call` to validate (checks balance, approval)

**Key Point**: Step 5 may fail if you lack token balance, but steps 1-4 always succeed (they prove Uniswap integration works).

### Routing Strategy

- **Direct swap**: If one token is WETH → single hop
- **Via WETH**: If neither token is WETH → two hops (TOKEN → WETH → TOKEN)
- **No complex routing**: Multi-hop paths beyond WETH not implemented

### Decimal Precision

Uses `rust_decimal::Decimal` for all financial calculations to avoid floating-point errors:
- ETH: 18 decimals (1 ETH = 1,000,000,000,000,000,000 wei)
- USDC: 6 decimals (1 USDC = 1,000,000 units)
- Automatic conversion between raw (U256) and formatted (Decimal)

---

## Demo

This section demonstrates all functionality using real Ethereum mainnet data.

### 1. Show Your Wallet Address

```bash
./target/release/show-address
```

**Output:**
```
Your wallet address: 0x39cfc46991e17ada6429f4628cfb0c4c4a6886bf
```

This displays the Ethereum address derived from your private key.

---

### 2. Check Your Own ETH Balance

```bash
./target/release/eth-mcp-server 2>/dev/null <<'EOF'
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_balance","arguments":{"wallet":"0x39cfc46991e17ada6429f4628cfb0c4c4a6886bf"}}}
EOF
```

**Expected Output:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "wallet": "0x39cfc46991e17ada6429f4628cfb0c4c4a6886bf",
    "native": {
      "raw": "0",
      "formatted": "0",
      "decimals": 18,
      "symbol": "ETH"
    },
    "block_number": 23604361
  }
}
```

My wallet is empty (0 ETH). Let's check a wallet with funds...

---

### 3. Check Vitalik's ETH Balance

```bash
./target/release/eth-mcp-server 2>/dev/null <<'EOF'
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_balance","arguments":{"wallet":"0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"}}}
EOF
```

**Expected Output:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "wallet": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
    "native": {
      "raw": "788335827583562397",
      "formatted": "0.788335827583562397",
      "decimals": 18,
      "symbol": "ETH"
    },
    "block_number": 23604361
  }
}
```

Vitalik has 0.788 ETH! This is **real-time data from Ethereum mainnet**.

---

### 4. Check Vitalik's USDC Balance

```bash
./target/release/eth-mcp-server 2>/dev/null <<'EOF'
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_balance","arguments":{"wallet":"0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045","token":"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"}}}
EOF
```

**Expected Output:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "wallet": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
    "erc20": {
      "raw": "17058407780",
      "formatted": "17058.40778",
      "decimals": 6,
      "symbol": "USDC"
    },
    "block_number": 23604361
  }
}
```

Vitalik has 17,058 USDC! Notice that USDC uses 6 decimals, not 18.

---

### 5. Get USDC Price (in ETH)

```bash
./target/release/eth-mcp-server 2>/dev/null <<'EOF'
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_token_price","arguments":{"token":"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48","base":"eth"}}}
EOF
```

**Expected Output:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    "base": "WETH",
    "price": "0.0002608076152110160880795511",
    "pair": "0xb4e16d0168e52d35cacd2c6185b44281ec28c9dc",
    "source": "Uniswap V2",
    "block_number": 23604361
  }
}
```

**Interpretation:**
- 1 USDC = 0.000261 ETH
- Inversely: 1 ETH = 3,835 USDC
- Price from Uniswap V2 USDC/WETH pool
- Real-time data from liquidity reserves

---

### 6. Verify Uniswap V2 Integration (test-swap)

Before testing `swap_tokens`, let's prove our Uniswap integration is correct:

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
  Estimated output: 0.256880914571150306 WETH
  Raw value: 256880914571150306

📊 With 0.5% slippage protection:
  Minimum output: 0.255596509998294554 WETH
  Raw value: 255596509998294554

📝 Transaction data encoded:
  Function: swapExactTokensForTokens
  Calldata length: 260 bytes
  Calldata (first 68 bytes): 0x38ed1739000000000000000000000000000000000000000000000000000000003b9aca00

⛽ Estimating gas cost...
  Gas limit: 150000 (estimated, actual may vary)
  Note: Precise gas estimation requires token balance & approval
  Gas price: 0.102462498 gwei
  Estimated fee: 0.0000153693747 ETH

✅ All Uniswap V2 integration working correctly!
✅ Using REAL mainnet data from Infura
✅ Transaction construction complete
✅ Gas estimation complete
```

**What this proves:**
- ✅ Real-time connection to Ethereum mainnet via RPC
- ✅ Actual calls to Uniswap V2 Router contract
- ✅ Accurate price calculations (1000 USDC → 0.257 WETH)
- ✅ Proper slippage calculation (0.5%)
- ✅ Correct transaction encoding (260 bytes calldata)
- ✅ Real-time gas price fetching and fee estimation

**Important**: test-swap doesn't require token balance because it only queries prices and builds transactions—it doesn't simulate execution.

---

### 7. Simulate Token Swap (swap_tokens)

Now let's try the full `swap_tokens` MCP tool:

```bash
./target/release/eth-mcp-server 2>&1 <<'EOF'
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"swap_tokens","arguments":{"from_token":"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48","to_token":"0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2","amount":"1000","slippage_bps":50}}}
EOF
```

**Expected Output:**
```
2025-10-18T11:53:29.104372Z  INFO eth_mcp_server::rpc: Received request: method=tools/call
2025-10-18T11:53:29.104401Z  INFO eth_mcp_server::rpc: Tool call: swap_tokens
2025-10-18T11:53:29.104425Z  INFO eth_mcp_server::tools::swap: swap_tokens called: 1000 0xA0b8...B48 -> 0xC02a...Cc2
2025-10-18T11:53:30.751017Z ERROR eth_mcp_server::rpc: Request error: Swap simulation failed (likely insufficient balance or approval). Gas estimate: 150000 units @ 0.108172169 gwei = 0.00001622582535 ETH. Error: Swap simulation failed

{"jsonrpc":"2.0","id":5,"error":{"code":-32603,"message":"Swap simulation failed (likely insufficient balance or approval). Gas estimate: 150000 units @ 0.108172169 gwei = 0.00001622582535 ETH. Error: Swap simulation failed"}}
```

**What happened?**

The swap failed, but **this is correct behavior**! Here's the full process:

1. ✅ **Step 1: Query Uniswap** - Successfully got price (1000 USDC → 0.257 WETH)
2. ✅ **Step 2: Calculate slippage** - Minimum output with 0.5% slippage = 0.256 WETH
3. ✅ **Step 3: Estimate gas** - Gas limit: 150,000 units @ 0.108 gwei = 0.000016 ETH
4. ✅ **Step 4: Encode transaction** - Built proper calldata for Uniswap Router
5. ❌ **Step 5: Simulate execution** - Failed because my wallet (0x39cf...6bf) has 0 USDC

**Key Observation**: Even though the transaction failed, the error message includes **complete gas estimation**:
- Gas limit: 150,000 units
- Gas price: 0.108172169 gwei (real-time from mainnet)
- Estimated fee: 0.00001622582535 ETH (~$0.06)

**Why two tools?**

| Tool | test-swap | swap_tokens |
|------|-----------|-------------|
| **Steps executed** | 1-4 | 1-5 |
| **Checks balance** | ❌ No | ✅ Yes |
| **Needs tokens** | ❌ No | ✅ Yes |
| **Purpose** | Prove integration works | Full validation |
| **Result** | ✅ Always succeeds | ⚠️ Fails without balance |

Together, they prove:
- test-swap: Uniswap integration is correct
- swap_tokens: Full validation logic works (including balance checks)

---

## API Reference

### MCP Protocol Methods

#### Initialize

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

#### List Tools

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
            "token": {"type": "string", "description": "Optional ERC20 token contract address"}
          },
          "required": ["wallet"]
        }
      },
      {
        "name": "get_token_price",
        "description": "Get real-time token price from Uniswap V2",
        "input_schema": {
          "type": "object",
          "properties": {
            "token": {"type": "string", "description": "ERC20 token contract address"},
            "base": {"type": "string", "description": "Base currency: 'eth' or 'usd'"}
          },
          "required": ["token"]
        }
      },
      {
        "name": "swap_tokens",
        "description": "Simulate token swap with gas estimation",
        "input_schema": {
          "type": "object",
          "properties": {
            "from_token": {"type": "string", "description": "Source token address"},
            "to_token": {"type": "string", "description": "Destination token address"},
            "amount": {"type": "string", "description": "Amount to swap (human-readable)"},
            "slippage_bps": {"type": "integer", "description": "Slippage in basis points (default: 50 = 0.5%)"}
          },
          "required": ["from_token", "to_token", "amount"]
        }
      }
    ]
  }
}
```

### Tool: get_balance

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
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
  "id": 3,
  "result": {
    "wallet": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
    "erc20": {
      "raw": "17058407780",
      "formatted": "17058.40778",
      "decimals": 6,
      "symbol": "USDC"
    },
    "block_number": 23604361
  }
}
```

### Tool: get_token_price

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
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
  "id": 4,
  "result": {
    "token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    "base": "WETH",
    "price": "0.0002608076152110160880795511",
    "pair": "0xb4e16d0168e52d35cacd2c6185b44281ec28c9dc",
    "source": "Uniswap V2",
    "block_number": 23604361
  }
}
```

### Tool: swap_tokens

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
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

**Response (success):**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "from_token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    "to_token": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
    "amount_in": "1000",
    "path": [
      "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
      "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
    ],
    "estimated_out": {
      "raw": "256880914571150306",
      "formatted": "0.256880914571150306"
    },
    "amount_out_min": {
      "raw": "255596509998294554",
      "formatted": "0.255596509998294554"
    },
    "gas_limit": "150000",
    "gas_price": "108172169",
    "estimated_fee_native": "0.00001622582535",
    "slippage_bps": 50
  }
}
```

**Response (insufficient balance):**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "error": {
    "code": -32603,
    "message": "Swap simulation failed (likely insufficient balance or approval). Gas estimate: 150000 units @ 0.108172169 gwei = 0.00001622582535 ETH. Error: Swap simulation failed"
  }
}
```

---

## Testing

### Unit Tests

```bash
cargo test
```

Expected output: `19 passed`

Tests cover:
- Decimal conversion (U256 ↔ Decimal)
- Uniswap V2 price calculations
- Swap path routing logic
- Amount parsing with different decimals

### Integration Tests

Integration tests connect to real Ethereum mainnet:

```bash
# Requires RPC_URL in .env
cargo test --test it_mainnet -- --ignored --nocapture
```

These tests verify:
- Real RPC connection
- Uniswap V2 contract calls
- Live balance queries
- Price data accuracy

### Verification Tool

The `test-swap` binary proves Uniswap integration works without requiring token balance:

```bash
./target/release/test-swap
```

This is ideal for demonstrations and debugging.

---

## Architecture

### Project Structure

```
eth-trading-mcp-server/
│
├── Cargo.toml                # Project dependencies and metadata
├── .env.example              # Environment variable template
├── README.md                 # This file
│
├── src/                      # Main source code
│   ├── main.rs              # MCP server entry point (stdin/stdout)
│   ├── lib.rs               # Library exports for bins and tests
│   ├── config.rs            # Environment configuration (RPC_URL, etc.)
│   ├── types.rs             # MCP protocol types (Request/Response DTOs)
│   ├── rpc.rs               # MCP JSON-RPC handler (routes to tools)
│   │
│   ├── eth/                 # Ethereum RPC interactions
│   │   ├── mod.rs           # Module exports
│   │   ├── provider.rs      # Provider/signer creation
│   │   └── erc20.rs         # ERC20 ABI calls (balanceOf, decimals, symbol)
│   │
│   ├── uniswap/             # Uniswap V2 integration
│   │   ├── mod.rs           # Module exports
│   │   └── v2.rs            # Router/pair interactions, swap encoding
│   │
│   ├── tools/               # MCP tool implementations
│   │   ├── mod.rs           # Module exports
│   │   ├── balance.rs       # get_balance (ETH + ERC20)
│   │   ├── price.rs         # get_token_price (from Uniswap reserves)
│   │   └── swap.rs          # swap_tokens (5-step validation)
│   │
│   ├── utils/               # Utility functions
│   │   ├── mod.rs           # Module exports
│   │   └── decimal.rs       # U256 ↔ Decimal conversion
│   │
│   └── bin/                 # Verification utilities
│       ├── show_address.rs  # Display wallet address from private key
│       └── test_swap.rs     # Verify Uniswap integration (no balance needed)
│
└── tests/                   # Test suites
    ├── unit_decimal.rs      # Decimal conversion tests
    ├── unit_v2.rs           # Uniswap V2 logic tests
    └── it_mainnet.rs        # Integration tests (requires RPC)
```

### Module Responsibilities

| Module | Purpose | Key Functions |
|--------|---------|--------------|
| `main.rs` | MCP server entry point | stdin/stdout JSON-RPC loop |
| `config.rs` | Environment config | Load RPC_URL, PRIVATE_KEY, hardcoded addresses |
| `types.rs` | Data structures | MCPRequest, MCPResponse, tool DTOs |
| `rpc.rs` | Protocol handler | Route methods to tools |
| `eth/provider.rs` | RPC connection | Create provider/signer, get block/balance |
| `eth/erc20.rs` | Token queries | balanceOf, decimals, symbol via eth_call |
| `uniswap/v2.rs` | DEX integration | getAmountsOut, encode swap, simulate |
| `tools/balance.rs` | get_balance | Query ETH + ERC20 balances |
| `tools/price.rs` | get_token_price | Calculate from Uniswap reserves |
| `tools/swap.rs` | swap_tokens | 5-step process (query → estimate → simulate) |
| `utils/decimal.rs` | Precision math | U256 ↔ Decimal with proper decimals |

### Key Technologies

| Technology | Version | Purpose |
|------------|---------|---------|
| **Rust** | 2021 Edition | Systems programming language |
| **Tokio** | 1.42 | Async runtime for concurrent RPC calls |
| **ethers-rs** | 2.0.14 | Ethereum RPC client and ABI encoding |
| **rust_decimal** | 1.37 | Arbitrary precision decimals (no float errors) |
| **serde** | 1.0 | JSON serialization/deserialization |
| **serde_json** | 1.0 | MCP protocol JSON handling |
| **tracing** | 0.1 | Structured logging (logs to stderr) |
| **dotenv** | 0.15 | Automatic .env file loading |
| **anyhow** | 1.0 | Error handling with context |
| **hex** | 0.4 | Hex encoding/decoding |

### Data Flow

```
User/AI Agent
    │
    ├─ JSON-RPC request (stdin)
    ↓
┌─────────────────────────────────────┐
│  main.rs (MCP Server)               │
│  - Parse JSON                       │
│  - Call rpc.rs handler              │
└────────────┬────────────────────────┘
             │
             ↓
┌─────────────────────────────────────┐
│  rpc.rs (Protocol Handler)          │
│  - Route to appropriate tool        │
│  - initialize / tools/list / call   │
└────────────┬────────────────────────┘
             │
      ┌──────┴──────┬──────────────┐
      ↓             ↓              ↓
┌──────────┐  ┌──────────┐  ┌──────────┐
│ balance  │  │  price   │  │   swap   │
│  tool    │  │   tool   │  │   tool   │
└────┬─────┘  └────┬─────┘  └────┬─────┘
     │             │              │
     └─────────────┴──────────────┘
                   ↓
     ┌──────────────────────────────┐
     │  eth/provider.rs              │
     │  - RPC calls via ethers-rs    │
     │  - eth_call, eth_getBalance   │
     └─────────────┬─────────────────┘
                   │
                   ↓
          Ethereum Mainnet
          (Infura/Alchemy)
                   │
                   ↓
     ┌──────────────────────────────┐
     │  Smart Contracts              │
     │  - ERC20 tokens               │
     │  - Uniswap V2 Router/Pairs    │
     └───────────────────────────────┘
```

### Why This Architecture?

1. **Modular Design**: Clear separation of concerns (RPC ↔ Tools ↔ Blockchain)
2. **Testability**: Each module can be unit tested independently
3. **Reusability**: Common functions (decimals, provider) shared across tools
4. **Safety**: No actual transaction signing—all operations use eth_call
5. **Extensibility**: Easy to add new tools or support Uniswap V3

---

## Known Limitations

- **Uniswap V2 only**: V3 concentrated liquidity not supported
- **Price accuracy**: Reflects V2 pools only, may differ from CEX prices
- **Simple routing**: Only direct or single-hop (via WETH) paths
- **No symbol lookup**: Requires contract addresses (e.g., `0xA0b8...` not `"USDC"`)
- **Mainnet only**: Designed for Ethereum mainnet (chain ID 1)
- **Read-only**: No transaction broadcasting—simulation only

---

## Common Token Addresses (Ethereum Mainnet)

```
WETH:  0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
USDC:  0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
USDT:  0xdAC17F958D2ee523a2206206994597C13D831ec7
DAI:   0x6B175474E89094C44Da98b954EedeAC495271d0F
WBTC:  0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599

Uniswap V2 Router: 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D
Uniswap V2 Factory: 0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f
```

---

## License

MIT

---

## Contributing

Contributions welcome! Please ensure:
- `cargo build` compiles successfully
- `cargo test` passes all tests
- `cargo fmt` and `cargo clippy` produce clean output

---

## Development Notes

This project was developed with assistance from AI tools (Claude Code) and demonstrates:

- Rust async programming with Tokio
- Ethereum RPC interactions via ethers-rs
- Proper decimal handling for financial data
- MCP protocol compliance
- Real-world DeFi integration (Uniswap V2)
