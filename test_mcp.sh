#!/bin/bash

# Simple test script to interact with the MCP server
# Usage: ./test_mcp.sh

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Testing Ethereum Trading MCP Server ===${NC}\n"

# Check if server binary exists
if [ ! -f "target/release/eth-mcp-server" ]; then
    echo "Building server..."
    cargo build --release
fi

# Test 1: Initialize
echo -e "${GREEN}Test 1: Initialize${NC}"
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | \
    cargo run --release 2>/dev/null | jq .

echo -e "\n${GREEN}Test 2: List Tools${NC}"
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | \
    cargo run --release 2>/dev/null | jq '.result.tools[] | {name, description}'

echo -e "\n${GREEN}Test 3: Get ETH Balance (Vitalik's address)${NC}"
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_balance","arguments":{"wallet":"0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"}}}' | \
    cargo run --release 2>/dev/null | jq .

echo -e "\n${GREEN}Test 4: Get USDC Balance${NC}"
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_balance","arguments":{"wallet":"0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045","token":"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"}}}' | \
    cargo run --release 2>/dev/null | jq .

echo -e "\n${BLUE}=== Tests Complete ===${NC}"
