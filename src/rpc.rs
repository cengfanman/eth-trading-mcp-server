use anyhow::Result;
use serde_json::json;
use tracing::{error, info, warn};

use crate::{
    config::Config,
    eth::provider::{EthProvider, EthSigner},
    tools,
    types::*,
};

pub struct MCPServer {
    provider: EthProvider,
    signer: EthSigner,
    config: Config,
}

impl MCPServer {
    pub fn new(provider: EthProvider, signer: EthSigner, config: Config) -> Self {
        Self {
            provider,
            signer,
            config,
        }
    }

    pub async fn handle_request(&self, request: MCPRequest) -> MCPResponse {
        info!("Received request: method={}", request.method);

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(),
            "tools/list" => self.handle_list_tools(),
            "tools/call" => self.handle_tool_call(request.params).await,
            _ => {
                warn!("Unknown method: {}", request.method);
                return MCPResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(MCPError {
                        code: -32601,
                        message: format!("Method not found: {}", request.method),
                        data: None,
                    }),
                };
            }
        };

        match result {
            Ok(value) => MCPResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            },
            Err(e) => {
                error!("Request error: {}", e);
                MCPResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(MCPError {
                        code: -32603,
                        message: e.to_string(),
                        data: None,
                    }),
                }
            }
        }
    }

    fn handle_initialize(&self) -> Result<serde_json::Value> {
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "eth-trading-mcp-server",
                "version": "0.1.0"
            }
        }))
    }

    fn handle_list_tools(&self) -> Result<serde_json::Value> {
        let tools = vec![
            ToolDefinition {
                name: "get_balance".to_string(),
                description: "Query ETH and ERC20 token balances for a wallet address".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "wallet": {
                            "type": "string",
                            "description": "Ethereum wallet address (0x...)"
                        },
                        "token": {
                            "type": "string",
                            "description": "Optional ERC20 token contract address. If omitted, returns ETH balance."
                        }
                    },
                    "required": ["wallet"]
                }),
            },
            ToolDefinition {
                name: "get_token_price".to_string(),
                description: "Get current token price from Uniswap V2 liquidity pools".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "token": {
                            "type": "string",
                            "description": "Token contract address (0x...)"
                        },
                        "base": {
                            "type": "string",
                            "description": "Base currency: 'usd' (via USDC) or 'eth' (via WETH). Default: 'usd'",
                            "enum": ["usd", "eth"]
                        }
                    },
                    "required": ["token"]
                }),
            },
            ToolDefinition {
                name: "swap_tokens".to_string(),
                description: "Simulate a token swap on Uniswap V2. Returns estimated output and gas costs WITHOUT executing the transaction.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "from_token": {
                            "type": "string",
                            "description": "Source token address (0x...)"
                        },
                        "to_token": {
                            "type": "string",
                            "description": "Destination token address (0x...)"
                        },
                        "amount": {
                            "type": "string",
                            "description": "Amount to swap (in human-readable format, e.g., '1.5')"
                        },
                        "slippage_bps": {
                            "type": "integer",
                            "description": "Slippage tolerance in basis points (100 = 1%). Default: 50 (0.5%)",
                            "default": 50
                        }
                    },
                    "required": ["from_token", "to_token", "amount"]
                }),
            },
        ];

        Ok(serde_json::to_value(ListToolsResponse { tools })?)
    }

    async fn handle_tool_call(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let tool_call: ToolCallRequest = serde_json::from_value(params)?;

        info!("Tool call: {}", tool_call.name);

        match tool_call.name.as_str() {
            "get_balance" => {
                let request: GetBalanceRequest = serde_json::from_value(tool_call.arguments)?;
                let response = tools::balance::get_balance(&self.provider, request).await?;
                Ok(serde_json::to_value(response)?)
            }
            "get_token_price" => {
                let request: GetTokenPriceRequest = serde_json::from_value(tool_call.arguments)?;
                let response = tools::price::get_token_price(&self.provider, &self.config, request).await?;
                Ok(serde_json::to_value(response)?)
            }
            "swap_tokens" => {
                let request: SwapTokensRequest = serde_json::from_value(tool_call.arguments)?;
                let response = tools::swap::swap_tokens(&self.provider, &self.signer, &self.config, request).await?;
                Ok(serde_json::to_value(response)?)
            }
            _ => anyhow::bail!("Unknown tool: {}", tool_call.name),
        }
    }
}
