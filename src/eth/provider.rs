use anyhow::{Context, Result};
use ethers::{
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, U256},
};
use std::sync::Arc;

pub type EthProvider = Arc<Provider<Http>>;
pub type EthSigner = Arc<SignerMiddleware<Provider<Http>, LocalWallet>>;

/// Create an Ethereum provider
pub fn create_provider(rpc_url: &str) -> Result<EthProvider> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .context("Failed to create provider")?;
    Ok(Arc::new(provider))
}

/// Create a signer with provider
pub fn create_signer(
    rpc_url: &str,
    private_key: &str,
    chain_id: u64,
) -> Result<EthSigner> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .context("Failed to create provider")?;

    let wallet: LocalWallet = private_key
        .parse::<LocalWallet>()
        .context("Failed to parse private key")?
        .with_chain_id(chain_id);

    let client = SignerMiddleware::new(provider, wallet);
    Ok(Arc::new(client))
}

/// Get ETH balance for an address
pub async fn get_eth_balance(provider: &EthProvider, address: Address) -> Result<U256> {
    provider
        .get_balance(address, None)
        .await
        .context("Failed to get ETH balance")
}

/// Get current block number
pub async fn get_block_number(provider: &EthProvider) -> Result<u64> {
    provider
        .get_block_number()
        .await
        .context("Failed to get block number")
        .map(|bn| bn.as_u64())
}

/// Get current gas price
pub async fn get_gas_price(provider: &EthProvider) -> Result<U256> {
    provider
        .get_gas_price()
        .await
        .context("Failed to get gas price")
}
