use anyhow::{Context, Result};
use ethers::{
    abi::{encode, Function, Param, ParamType, StateMutability, Token},
    prelude::*,
    types::{Address, Bytes, U256},
};
use std::sync::Arc;

use crate::eth::provider::{EthProvider, EthSigner};

const UNISWAP_V2_FACTORY: &str = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f";

/// Get Uniswap V2 pair address
pub async fn get_pair(
    provider: &EthProvider,
    token_a: Address,
    token_b: Address,
) -> Result<Address> {
    let factory = Address::from_slice(&hex::decode(&UNISWAP_V2_FACTORY[2..]).unwrap());

    let get_pair_fn = Function {
        name: "getPair".to_string(),
        inputs: vec![
            Param {
                name: "tokenA".to_string(),
                kind: ParamType::Address,
                internal_type: Some("address".to_string()),
            },
            Param {
                name: "tokenB".to_string(),
                kind: ParamType::Address,
                internal_type: Some("address".to_string()),
            },
        ],
        outputs: vec![Param {
            name: "pair".to_string(),
            kind: ParamType::Address,
            internal_type: Some("address".to_string()),
        }],
        constant: None,
        state_mutability: StateMutability::View,
    };

    let data = get_pair_fn
        .encode_input(&[Token::Address(token_a), Token::Address(token_b)])
        .context("Failed to encode getPair call")?;

    let tx = TransactionRequest::new()
        .to(factory)
        .data(data)
        .into();

    let call_result = provider
        .call(&tx, None)
        .await
        .context("Failed to call getPair")?;

    let decoded = get_pair_fn
        .decode_output(&call_result)
        .context("Failed to decode getPair output")?;

    match decoded.first() {
        Some(Token::Address(pair)) => {
            if pair == &Address::zero() {
                anyhow::bail!("Pair does not exist for tokens");
            }
            Ok(*pair)
        }
        _ => anyhow::bail!("Unexpected getPair return type"),
    }
}

/// Get reserves from a Uniswap V2 pair
pub async fn get_reserves(
    provider: &EthProvider,
    pair: Address,
) -> Result<(U256, U256, u32)> {
    let get_reserves_fn = Function {
        name: "getReserves".to_string(),
        inputs: vec![],
        outputs: vec![
            Param {
                name: "reserve0".to_string(),
                kind: ParamType::Uint(112),
                internal_type: Some("uint112".to_string()),
            },
            Param {
                name: "reserve1".to_string(),
                kind: ParamType::Uint(112),
                internal_type: Some("uint112".to_string()),
            },
            Param {
                name: "blockTimestampLast".to_string(),
                kind: ParamType::Uint(32),
                internal_type: Some("uint32".to_string()),
            },
        ],
        constant: None,
        state_mutability: StateMutability::View,
    };

    let data = get_reserves_fn
        .encode_input(&[])
        .context("Failed to encode getReserves call")?;

    let tx = TransactionRequest::new()
        .to(pair)
        .data(data)
        .into();

    let call_result = provider
        .call(&tx, None)
        .await
        .context("Failed to call getReserves")?;

    let decoded = get_reserves_fn
        .decode_output(&call_result)
        .context("Failed to decode getReserves output")?;

    match decoded.as_slice() {
        [Token::Uint(reserve0), Token::Uint(reserve1), Token::Uint(timestamp)] => {
            Ok((*reserve0, *reserve1, timestamp.as_u32()))
        }
        _ => anyhow::bail!("Unexpected getReserves return type"),
    }
}

/// Get token0 from pair
pub async fn get_token0(provider: &EthProvider, pair: Address) -> Result<Address> {
    let token0_fn = Function {
        name: "token0".to_string(),
        inputs: vec![],
        outputs: vec![Param {
            name: "token0".to_string(),
            kind: ParamType::Address,
            internal_type: Some("address".to_string()),
        }],
        constant: None,
        state_mutability: StateMutability::View,
    };

    let data = token0_fn
        .encode_input(&[])
        .context("Failed to encode token0 call")?;

    let tx = TransactionRequest::new()
        .to(pair)
        .data(data)
        .into();

    let call_result = provider
        .call(&tx, None)
        .await
        .context("Failed to call token0")?;

    let decoded = token0_fn
        .decode_output(&call_result)
        .context("Failed to decode token0 output")?;

    match decoded.first() {
        Some(Token::Address(token)) => Ok(*token),
        _ => anyhow::bail!("Unexpected token0 return type"),
    }
}

/// Calculate price from reserves
pub fn calculate_price(
    reserve_in: U256,
    reserve_out: U256,
    decimals_in: u8,
    decimals_out: u8,
) -> Result<rust_decimal::Decimal> {
    use crate::utils::decimal::u256_to_decimal;

    let reserve_in_decimal = u256_to_decimal(reserve_in, decimals_in)?;
    let reserve_out_decimal = u256_to_decimal(reserve_out, decimals_out)?;

    if reserve_in_decimal.is_zero() {
        anyhow::bail!("Reserve in is zero");
    }

    Ok(reserve_out_decimal / reserve_in_decimal)
}

/// Get amounts out from Uniswap V2 Router
pub async fn get_amounts_out(
    provider: &EthProvider,
    router: Address,
    amount_in: U256,
    path: Vec<Address>,
) -> Result<Vec<U256>> {
    let get_amounts_out_fn = Function {
        name: "getAmountsOut".to_string(),
        inputs: vec![
            Param {
                name: "amountIn".to_string(),
                kind: ParamType::Uint(256),
                internal_type: Some("uint256".to_string()),
            },
            Param {
                name: "path".to_string(),
                kind: ParamType::Array(Box::new(ParamType::Address)),
                internal_type: Some("address[]".to_string()),
            },
        ],
        outputs: vec![Param {
            name: "amounts".to_string(),
            kind: ParamType::Array(Box::new(ParamType::Uint(256))),
            internal_type: Some("uint256[]".to_string()),
        }],
        constant: None,
        state_mutability: StateMutability::View,
    };

    let path_tokens: Vec<Token> = path.iter().map(|addr| Token::Address(*addr)).collect();

    let data = get_amounts_out_fn
        .encode_input(&[Token::Uint(amount_in), Token::Array(path_tokens)])
        .context("Failed to encode getAmountsOut call")?;

    let tx = TransactionRequest::new()
        .to(router)
        .data(data)
        .into();

    let call_result = provider
        .call(&tx, None)
        .await
        .context("Failed to call getAmountsOut")?;

    let decoded = get_amounts_out_fn
        .decode_output(&call_result)
        .context("Failed to decode getAmountsOut output")?;

    match decoded.first() {
        Some(Token::Array(amounts)) => {
            let mut result = Vec::new();
            for amount in amounts {
                match amount {
                    Token::Uint(val) => result.push(*val),
                    _ => anyhow::bail!("Unexpected amount type in getAmountsOut"),
                }
            }
            Ok(result)
        }
        _ => anyhow::bail!("Unexpected getAmountsOut return type"),
    }
}

/// Build swap path (direct or via WETH)
pub fn build_swap_path(from_token: Address, to_token: Address, weth: Address) -> Vec<Address> {
    // Direct swap if one is WETH
    if from_token == weth || to_token == weth {
        vec![from_token, to_token]
    } else {
        // Via WETH
        vec![from_token, weth, to_token]
    }
}

/// Encode swapExactTokensForTokens call
pub fn encode_swap_exact_tokens_for_tokens(
    amount_in: U256,
    amount_out_min: U256,
    path: Vec<Address>,
    to: Address,
    deadline: U256,
) -> Result<Bytes> {
    let swap_fn = Function {
        name: "swapExactTokensForTokens".to_string(),
        inputs: vec![
            Param {
                name: "amountIn".to_string(),
                kind: ParamType::Uint(256),
                internal_type: Some("uint256".to_string()),
            },
            Param {
                name: "amountOutMin".to_string(),
                kind: ParamType::Uint(256),
                internal_type: Some("uint256".to_string()),
            },
            Param {
                name: "path".to_string(),
                kind: ParamType::Array(Box::new(ParamType::Address)),
                internal_type: Some("address[]".to_string()),
            },
            Param {
                name: "to".to_string(),
                kind: ParamType::Address,
                internal_type: Some("address".to_string()),
            },
            Param {
                name: "deadline".to_string(),
                kind: ParamType::Uint(256),
                internal_type: Some("uint256".to_string()),
            },
        ],
        outputs: vec![Param {
            name: "amounts".to_string(),
            kind: ParamType::Array(Box::new(ParamType::Uint(256))),
            internal_type: Some("uint256[]".to_string()),
        }],
        constant: None,
        state_mutability: StateMutability::NonPayable,
    };

    let path_tokens: Vec<Token> = path.iter().map(|addr| Token::Address(*addr)).collect();

    let data = swap_fn
        .encode_input(&[
            Token::Uint(amount_in),
            Token::Uint(amount_out_min),
            Token::Array(path_tokens),
            Token::Address(to),
            Token::Uint(deadline),
        ])
        .context("Failed to encode swap call")?;

    Ok(data.into())
}

/// Simulate swap transaction (eth_call)
pub async fn simulate_swap(
    provider: &EthProvider,
    router: Address,
    from: Address,
    swap_data: Bytes,
) -> Result<()> {
    let tx = TransactionRequest::new()
        .from(from)
        .to(router)
        .data(swap_data)
        .into();

    provider
        .call(&tx, None)
        .await
        .context("Swap simulation failed")?;

    Ok(())
}

/// Estimate gas for swap
pub async fn estimate_swap_gas(
    provider: &EthProvider,
    router: Address,
    from: Address,
    swap_data: Bytes,
) -> Result<U256> {
    let tx = TransactionRequest::new()
        .from(from)
        .to(router)
        .data(swap_data)
        .into();

    provider
        .estimate_gas(&tx, None)
        .await
        .context("Failed to estimate gas")
}
