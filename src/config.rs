use anyhow::{Context, Result};
use ethers::types::Address;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_url: String,
    pub chain_id: u64,
    pub private_key: String,
    pub uniswap_v2_router: Address,
    pub weth: Address,
    pub usdc: Address,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let rpc_url = std::env::var("RPC_URL")
            .context("RPC_URL environment variable not set")?;

        let chain_id = std::env::var("CHAIN_ID")
            .unwrap_or_else(|_| "1".to_string())
            .parse::<u64>()
            .context("Invalid CHAIN_ID")?;

        let private_key = std::env::var("PRIVATE_KEY")
            .context("PRIVATE_KEY environment variable not set")?;

        // Ethereum mainnet constants
        let uniswap_v2_router = Address::from_str("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D")
            .context("Invalid Uniswap V2 router address")?;

        let weth = Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")
            .context("Invalid WETH address")?;

        let usdc = Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
            .context("Invalid USDC address")?;

        Ok(Config {
            rpc_url,
            chain_id,
            private_key,
            uniswap_v2_router,
            weth,
            usdc,
        })
    }
}
