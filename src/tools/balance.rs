use anyhow::{Context, Result};
use ethers::types::Address;
use std::str::FromStr;
use tracing::{debug, info};

use crate::{
    eth::{erc20, provider},
    types::{BalanceInfo, GetBalanceRequest, GetBalanceResponse},
    utils::decimal::u256_to_decimal,
};

pub async fn get_balance(
    provider: &provider::EthProvider,
    request: GetBalanceRequest,
) -> Result<GetBalanceResponse> {
    info!("get_balance called for wallet: {}", request.wallet);

    let wallet_addr = Address::from_str(&request.wallet)
        .context("Invalid wallet address")?;

    let block_number = provider::get_block_number(provider).await?;

    let mut response = GetBalanceResponse {
        wallet: request.wallet.clone(),
        native: None,
        erc20: None,
        block_number,
    };

    // Get native ETH balance if no token specified or token is empty
    if request.token.is_none() || request.token.as_ref().map(|s| s.is_empty()).unwrap_or(false) {
        debug!("Fetching ETH balance");
        let eth_balance = provider::get_eth_balance(provider, wallet_addr).await?;
        let eth_formatted = u256_to_decimal(eth_balance, 18)?;

        response.native = Some(BalanceInfo {
            raw: eth_balance.to_string(),
            formatted: eth_formatted.to_string(),
            decimals: Some(18),
            symbol: Some("ETH".to_string()),
        });

        info!("ETH balance: {} ETH", eth_formatted);
    }

    // Get ERC20 token balance if token is specified
    if let Some(token_str) = &request.token {
        if !token_str.is_empty() {
            debug!("Fetching ERC20 token balance for: {}", token_str);
            let token_addr = Address::from_str(token_str)
                .context("Invalid token address")?;

            let (symbol, decimals, balance) = erc20::get_token_info(provider, token_addr, wallet_addr).await?;
            let formatted = u256_to_decimal(balance, decimals)?;

            response.erc20 = Some(BalanceInfo {
                raw: balance.to_string(),
                formatted: formatted.to_string(),
                decimals: Some(decimals),
                symbol: Some(symbol.clone()),
            });

            info!("Token {} balance: {} {}", token_str, formatted, symbol);
        }
    }

    Ok(response)
}
