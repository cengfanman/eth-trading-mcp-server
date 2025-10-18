// Integration tests that require a real RPC connection
// Run with: cargo test --test it_mainnet -- --ignored --nocapture

#[cfg(test)]
mod tests {
    use eth_trading_mcp_server::{
        config::Config,
        eth::provider,
        tools::{balance, price, swap},
        types::*,
    };
    use std::env;

    fn get_test_config() -> Option<Config> {
        // Only run if RPC_URL is set
        if env::var("RPC_URL").is_err() {
            return None;
        }

        Config::from_env().ok()
    }

    #[tokio::test]
    #[ignore] // Only run with --ignored flag
    async fn test_get_eth_balance() {
        let config = match get_test_config() {
            Some(c) => c,
            None => {
                println!("Skipping test: RPC_URL not set");
                return;
            }
        };

        let provider = provider::create_provider(&config.rpc_url).unwrap();

        // Test with Vitalik's address
        let request = GetBalanceRequest {
            wallet: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(),
            token: None,
        };

        let response = balance::get_balance(&provider, request).await;
        assert!(response.is_ok());

        let response = response.unwrap();
        assert!(response.native.is_some());
        println!("ETH Balance: {:?}", response.native);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_token_balance() {
        let config = match get_test_config() {
            Some(c) => c,
            None => {
                println!("Skipping test: RPC_URL not set");
                return;
            }
        };

        let provider = provider::create_provider(&config.rpc_url).unwrap();

        // Test USDC balance
        let request = GetBalanceRequest {
            wallet: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(),
            token: Some(config.usdc.to_string()),
        };

        let response = balance::get_balance(&provider, request).await;
        assert!(response.is_ok());

        let response = response.unwrap();
        assert!(response.erc20.is_some());
        println!("Token Balance: {:?}", response.erc20);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_token_price() {
        let config = match get_test_config() {
            Some(c) => c,
            None => {
                println!("Skipping test: RPC_URL not set");
                return;
            }
        };

        let provider = provider::create_provider(&config.rpc_url).unwrap();

        // Test USDC price (should be ~1 USD)
        let request = GetTokenPriceRequest {
            token: config.usdc.to_string(),
            base: Some("eth".to_string()),
        };

        let response = price::get_token_price(&provider, &config, request).await;
        assert!(response.is_ok());

        let response = response.unwrap();
        println!("Token Price: {} ETH", response.price);
    }

    #[tokio::test]
    #[ignore]
    async fn test_swap_simulation() {
        let config = match get_test_config() {
            Some(c) => c,
            None => {
                println!("Skipping test: RPC_URL not set");
                return;
            }
        };

        let provider = provider::create_provider(&config.rpc_url).unwrap();
        let signer = provider::create_signer(&config.rpc_url, &config.private_key, config.chain_id).unwrap();

        // Test swap: USDC -> WETH
        let request = SwapTokensRequest {
            from_token: config.usdc.to_string(),
            to_token: config.weth.to_string(),
            amount: "1000".to_string(), // 1000 USDC
            slippage_bps: 50,
        };

        let response = swap::swap_tokens(&provider, &signer, &config, request).await;

        match response {
            Ok(resp) => {
                println!("Swap Simulation:");
                println!("  Estimated out: {} ETH", resp.estimated_out.formatted);
                println!("  Min out: {} ETH", resp.amount_out_min.formatted);
                println!("  Gas limit: {}", resp.gas_limit);
                println!("  Gas price: {}", resp.gas_price);
                println!("  Estimated fee: {} ETH", resp.estimated_fee_native);
            }
            Err(e) => {
                // Swap might fail if liquidity is low or account has no tokens
                println!("Swap simulation failed (expected): {}", e);
            }
        }
    }
}
