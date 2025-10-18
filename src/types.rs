use ethers::types::U256;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// MCP Protocol types
#[derive(Debug, Serialize, Deserialize)]
pub struct MCPRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<MCPError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

// Tool call types
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCallRequest {
    pub name: String,
    pub arguments: serde_json::Value,
}

// get_balance types
#[derive(Debug, Serialize, Deserialize)]
pub struct GetBalanceRequest {
    pub wallet: String,
    pub token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub raw: String,
    pub formatted: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decimals: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetBalanceResponse {
    pub wallet: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native: Option<BalanceInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub erc20: Option<BalanceInfo>,
    pub block_number: u64,
}

// get_token_price types
#[derive(Debug, Serialize, Deserialize)]
pub struct GetTokenPriceRequest {
    pub token: String,
    #[serde(default)]
    pub base: Option<String>, // "usd" or "eth"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTokenPriceResponse {
    pub token: String,
    pub base: String,
    pub price: String,
    pub pair: String,
    pub source: String,
    pub block_number: u64,
}

// swap_tokens types
#[derive(Debug, Serialize, Deserialize)]
pub struct SwapTokensRequest {
    pub from_token: String,
    pub to_token: String,
    pub amount: String,
    #[serde(default = "default_slippage")]
    pub slippage_bps: u64, // basis points (100 = 1%)
}

fn default_slippage() -> u64 {
    50 // 0.5%
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapTokensResponse {
    pub from_token: String,
    pub to_token: String,
    pub amount_in: String,
    pub path: Vec<String>,
    pub estimated_out: AmountInfo,
    pub amount_out_min: AmountInfo,
    pub gas_limit: String,
    pub gas_price: String,
    pub estimated_fee_native: String,
    pub slippage_bps: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AmountInfo {
    pub raw: String,
    pub formatted: String,
}

impl AmountInfo {
    pub fn new(raw: U256, formatted: Decimal) -> Self {
        Self {
            raw: raw.to_string(),
            formatted: formatted.to_string(),
        }
    }
}

// List tools response
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListToolsResponse {
    pub tools: Vec<ToolDefinition>,
}
