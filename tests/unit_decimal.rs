// Unit tests are already in src/utils/decimal.rs
// This file serves as a placeholder for additional decimal-related tests

#[cfg(test)]
mod tests {
    use eth_trading_mcp_server::utils::decimal::*;
    use ethers::types::U256;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_large_eth_amount() {
        let amount = U256::from_dec_str("123456789000000000000").unwrap(); // 123.456789 ETH
        let decimal = u256_to_decimal(amount, 18).unwrap();
        assert_eq!(decimal, Decimal::from_str("123.456789").unwrap());
    }

    #[test]
    fn test_small_usdc_amount() {
        let amount = U256::from_dec_str("123456").unwrap(); // 0.123456 USDC
        let decimal = u256_to_decimal(amount, 6).unwrap();
        assert_eq!(decimal, Decimal::from_str("0.123456").unwrap());
    }

    #[test]
    fn test_zero_amount() {
        let amount = U256::zero();
        let decimal = u256_to_decimal(amount, 18).unwrap();
        assert_eq!(decimal, Decimal::ZERO);
    }

    #[test]
    fn test_max_precision() {
        let decimal = Decimal::from_str("0.000000000000000001").unwrap(); // 1 wei
        let u256 = decimal_to_u256(decimal, 18).unwrap();
        assert_eq!(u256, U256::one());
    }

    #[test]
    fn test_round_trip() {
        let original = U256::from_dec_str("1500000000000000000").unwrap(); // 1.5 ETH
        let decimal = u256_to_decimal(original, 18).unwrap();
        let back = decimal_to_u256(decimal, 18).unwrap();
        assert_eq!(original, back);
    }
}
