#[cfg(test)]
mod tests {
    use eth_trading_mcp_server::uniswap::v2::*;
    use ethers::types::{Address, U256};
    use std::str::FromStr;

    const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    const DAI: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";

    #[test]
    fn test_build_swap_path_direct() {
        let weth = Address::from_str(WETH).unwrap();
        let usdc = Address::from_str(USDC).unwrap();

        let path = build_swap_path(weth, usdc, weth);
        assert_eq!(path.len(), 2);
        assert_eq!(path[0], weth);
        assert_eq!(path[1], usdc);
    }

    #[test]
    fn test_build_swap_path_via_weth() {
        let weth = Address::from_str(WETH).unwrap();
        let usdc = Address::from_str(USDC).unwrap();
        let dai = Address::from_str(DAI).unwrap();

        let path = build_swap_path(usdc, dai, weth);
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], usdc);
        assert_eq!(path[1], weth);
        assert_eq!(path[2], dai);
    }

    #[test]
    fn test_calculate_price() {
        // Simulate reserves: 1000 ETH and 2,000,000 USDC
        let reserve_eth = U256::from_dec_str("1000000000000000000000").unwrap(); // 1000 ETH
        let reserve_usdc = U256::from_dec_str("2000000000000").unwrap(); // 2,000,000 USDC

        let price = calculate_price(reserve_eth, reserve_usdc, 18, 6).unwrap();

        // Price should be 2000 USDC per ETH
        assert_eq!(price.to_string(), "2000");
    }

    #[test]
    fn test_encode_swap_data() {
        let amount_in = U256::from_dec_str("1000000000000000000").unwrap();
        let amount_out_min = U256::from_dec_str("1900000000").unwrap();
        let weth = Address::from_str(WETH).unwrap();
        let usdc = Address::from_str(USDC).unwrap();
        let path = vec![weth, usdc];
        let to = Address::from_str("0x1234567890123456789012345678901234567890").unwrap();
        let deadline = U256::from(1700000000u64);

        let result = encode_swap_exact_tokens_for_tokens(
            amount_in,
            amount_out_min,
            path,
            to,
            deadline,
        );

        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty());

        // Verify function selector (first 4 bytes)
        // swapExactTokensForTokens selector: 0x38ed1739
        let selector = &data[0..4];
        assert_eq!(hex::encode(selector), "38ed1739");
    }
}
