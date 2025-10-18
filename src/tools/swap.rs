use anyhow::{Context, Result};
use ethers::types::{Address, U256};
use std::str::FromStr;
use tracing::{debug, info};

use crate::{
    config::Config,
    eth::{erc20, provider},
    types::{AmountInfo, SwapTokensRequest, SwapTokensResponse},
    uniswap::v2,
    utils::decimal::{parse_amount, u256_to_decimal},
};

pub async fn swap_tokens(
    provider: &provider::EthProvider,
    signer: &provider::EthSigner,
    config: &Config,
    request: SwapTokensRequest,
) -> Result<SwapTokensResponse> {
    info!(
        "swap_tokens called: {} {} -> {}",
        request.amount, request.from_token, request.to_token
    );

    // Parse addresses
    let from_token_addr = Address::from_str(&request.from_token)
        .context("Invalid from_token address")?;
    let to_token_addr = Address::from_str(&request.to_token)
        .context("Invalid to_token address")?;

    // Get token decimals
    let from_decimals = erc20::get_decimals(provider, from_token_addr).await?;
    let to_decimals = erc20::get_decimals(provider, to_token_addr).await?;

    // Parse amount
    let amount_in = parse_amount(&request.amount, from_decimals)
        .context("Failed to parse amount")?;

    debug!("Amount in (raw): {}", amount_in);

    // Build swap path
    let path = v2::build_swap_path(from_token_addr, to_token_addr, config.weth);
    debug!("Swap path: {:?}", path);

    // Get amounts out from router
    let amounts_out = v2::get_amounts_out(provider, config.uniswap_v2_router, amount_in, path.clone())
        .await
        .context("Failed to get amounts out")?;

    let estimated_out = amounts_out
        .last()
        .copied()
        .context("No output amount returned")?;

    debug!("Estimated out (raw): {}", estimated_out);

    // Calculate minimum output with slippage
    let slippage_multiplier = 10000u64 - request.slippage_bps;
    let amount_out_min = estimated_out
        .saturating_mul(U256::from(slippage_multiplier))
        / U256::from(10000u64);

    debug!("Amount out min (raw): {}", amount_out_min);

    // Get signer address
    let signer_addr = signer.address();

    // Calculate deadline (current time + 20 minutes)
    let deadline = U256::from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 1200,
    );

    // Encode swap transaction
    let swap_data = v2::encode_swap_exact_tokens_for_tokens(
        amount_in,
        amount_out_min,
        path.clone(),
        signer_addr,
        deadline,
    )?;

    // Get current gas price (always works)
    let gas_price = provider::get_gas_price(provider).await?;

    // Estimate gas (may fail without balance, use fallback)
    let gas_limit = match v2::estimate_swap_gas(
        provider,
        config.uniswap_v2_router,
        signer_addr,
        swap_data.clone(),
    )
    .await
    {
        Ok(gas) => gas,
        Err(_) => {
            // Use typical value for Uniswap V2 swap
            U256::from(150_000u64)
        }
    };

    // Calculate estimated fee
    let estimated_fee = gas_limit * gas_price;
    let estimated_fee_eth = u256_to_decimal(estimated_fee, 18)?;

    // Simulate the swap (eth_call) - this validates the transaction
    if let Err(e) = v2::simulate_swap(provider, config.uniswap_v2_router, signer_addr, swap_data.clone()).await {
        // Simulation failed, but return gas info in error
        let gas_price_gwei = u256_to_decimal(gas_price, 9)?;
        return Err(anyhow::anyhow!(
            "Swap simulation failed (likely insufficient balance or approval). Gas estimate: {} units @ {} gwei = {} ETH. Error: {}",
            gas_limit,
            gas_price_gwei,
            estimated_fee_eth,
            e
        ));
    }

    debug!("Swap simulation succeeded");

    info!(
        "Swap simulation: {} -> {} (estimated), gas: {} @ {} gwei",
        request.amount,
        u256_to_decimal(estimated_out, to_decimals)?,
        gas_limit,
        u256_to_decimal(gas_price, 9)?
    );

    // Format amounts
    let estimated_out_formatted = u256_to_decimal(estimated_out, to_decimals)?;
    let amount_out_min_formatted = u256_to_decimal(amount_out_min, to_decimals)?;

    Ok(SwapTokensResponse {
        from_token: request.from_token.clone(),
        to_token: request.to_token.clone(),
        amount_in: request.amount.clone(),
        path: path.iter().map(|addr| format!("{:?}", addr)).collect(),
        estimated_out: AmountInfo::new(estimated_out, estimated_out_formatted),
        amount_out_min: AmountInfo::new(amount_out_min, amount_out_min_formatted),
        gas_limit: gas_limit.to_string(),
        gas_price: gas_price.to_string(),
        estimated_fee_native: estimated_fee_eth.to_string(),
        slippage_bps: request.slippage_bps,
    })
}
