use anyhow::{Context, Result};
use ethers::types::Address;
use std::str::FromStr;
use tracing::{debug, info};

use crate::{
    config::Config,
    eth::{erc20, provider},
    types::{GetTokenPriceRequest, GetTokenPriceResponse},
    uniswap::v2,
};

pub async fn get_token_price(
    provider: &provider::EthProvider,
    config: &Config,
    request: GetTokenPriceRequest,
) -> Result<GetTokenPriceResponse> {
    info!("get_token_price called for token: {}", request.token);

    let token_addr = Address::from_str(&request.token)
        .context("Invalid token address")?;

    let base = request.base.unwrap_or_else(|| "usd".to_string()).to_lowercase();

    // Determine the base token (USDC for USD, WETH for ETH)
    let (base_token, base_symbol) = match base.as_str() {
        "usd" => (config.usdc, "USDC"),
        "eth" => (config.weth, "WETH"),
        _ => anyhow::bail!("Unsupported base: {}. Use 'usd' or 'eth'", base),
    };

    debug!("Fetching price for token via {} pair", base_symbol);

    // Get the pair
    let pair = v2::get_pair(provider, token_addr, base_token)
        .await
        .context("Failed to get Uniswap V2 pair")?;

    debug!("Pair address: {:?}", pair);

    // Get reserves
    let (reserve0, reserve1, _) = v2::get_reserves(provider, pair)
        .await
        .context("Failed to get reserves")?;

    // Determine which reserve corresponds to which token
    let token0 = v2::get_token0(provider, pair).await?;

    let (token_reserve, base_reserve) = if token0 == token_addr {
        (reserve0, reserve1)
    } else {
        (reserve1, reserve0)
    };

    // Get decimals for price calculation
    let token_decimals = erc20::get_decimals(provider, token_addr).await?;
    let base_decimals = erc20::get_decimals(provider, base_token).await?;

    // Calculate price: base_reserve / token_reserve (how much base per 1 token)
    let price = v2::calculate_price(token_reserve, base_reserve, token_decimals, base_decimals)?;

    let block_number = provider::get_block_number(provider).await?;

    info!("Token price: {} {}", price, base_symbol);

    Ok(GetTokenPriceResponse {
        token: request.token.clone(),
        base: base_symbol.to_string(),
        price: price.to_string(),
        pair: format!("{:?}", pair),
        source: "Uniswap V2".to_string(),
        block_number,
    })
}
