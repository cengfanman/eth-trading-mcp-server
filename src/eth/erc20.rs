use anyhow::{Context, Result};
use ethers::{
    abi::{Function, Param, ParamType, StateMutability, Token},
    contract::Contract,
    prelude::*,
    types::{Address, U256},
};
use std::sync::Arc;

use super::provider::EthProvider;

/// Get ERC20 token balance
pub async fn get_balance(
    provider: &EthProvider,
    token: Address,
    owner: Address,
) -> Result<U256> {
    let balance_of = Function {
        name: "balanceOf".to_string(),
        inputs: vec![Param {
            name: "account".to_string(),
            kind: ParamType::Address,
            internal_type: Some("address".to_string()),
        }],
        outputs: vec![Param {
            name: "balance".to_string(),
            kind: ParamType::Uint(256),
            internal_type: Some("uint256".to_string()),
        }],
        constant: None,
        state_mutability: StateMutability::View,
    };

    let data = balance_of.encode_input(&[Token::Address(owner)])
        .context("Failed to encode balanceOf call")?;

    let tx = TransactionRequest::new()
        .to(token)
        .data(data)
        .into();

    let call_result = provider
        .call(&tx, None)
        .await
        .context("Failed to call balanceOf")?;

    let decoded = balance_of.decode_output(&call_result)
        .context("Failed to decode balanceOf output")?;

    match decoded.first() {
        Some(Token::Uint(balance)) => Ok(*balance),
        _ => anyhow::bail!("Unexpected balanceOf return type"),
    }
}

/// Get ERC20 token decimals
pub async fn get_decimals(provider: &EthProvider, token: Address) -> Result<u8> {
    let decimals_fn = Function {
        name: "decimals".to_string(),
        inputs: vec![],
        outputs: vec![Param {
            name: "decimals".to_string(),
            kind: ParamType::Uint(8),
            internal_type: Some("uint8".to_string()),
        }],
        constant: None,
        state_mutability: StateMutability::View,
    };

    let data = decimals_fn.encode_input(&[])
        .context("Failed to encode decimals call")?;

    let tx = TransactionRequest::new()
        .to(token)
        .data(data)
        .into();

    let call_result = provider
        .call(&tx, None)
        .await
        .context("Failed to call decimals")?;

    let decoded = decimals_fn.decode_output(&call_result)
        .context("Failed to decode decimals output")?;

    match decoded.first() {
        Some(Token::Uint(decimals)) => Ok(decimals.as_u32() as u8),
        _ => anyhow::bail!("Unexpected decimals return type"),
    }
}

/// Get ERC20 token symbol
pub async fn get_symbol(provider: &EthProvider, token: Address) -> Result<String> {
    let symbol_fn = Function {
        name: "symbol".to_string(),
        inputs: vec![],
        outputs: vec![Param {
            name: "symbol".to_string(),
            kind: ParamType::String,
            internal_type: Some("string".to_string()),
        }],
        constant: None,
        state_mutability: StateMutability::View,
    };

    let data = symbol_fn.encode_input(&[])
        .context("Failed to encode symbol call")?;

    let tx = TransactionRequest::new()
        .to(token)
        .data(data)
        .into();

    let call_result = provider
        .call(&tx, None)
        .await
        .context("Failed to call symbol")?;

    let decoded = symbol_fn.decode_output(&call_result)
        .context("Failed to decode symbol output")?;

    match decoded.first() {
        Some(Token::String(symbol)) => Ok(symbol.clone()),
        _ => anyhow::bail!("Unexpected symbol return type"),
    }
}

/// Get token info (symbol, decimals, balance)
pub async fn get_token_info(
    provider: &EthProvider,
    token: Address,
    owner: Address,
) -> Result<(String, u8, U256)> {
    let symbol = get_symbol(provider, token).await?;
    let decimals = get_decimals(provider, token).await?;
    let balance = get_balance(provider, token, owner).await?;

    Ok((symbol, decimals, balance))
}
