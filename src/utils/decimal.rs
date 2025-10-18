use anyhow::{anyhow, Result};
use ethers::types::U256;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Convert U256 wei/token units to Decimal with proper decimals
pub fn u256_to_decimal(value: U256, decimals: u8) -> Result<Decimal> {
    let value_str = value.to_string();
    let value_decimal = Decimal::from_str(&value_str)
        .map_err(|e| anyhow!("Failed to parse U256 to Decimal: {}", e))?;

    let divisor = Decimal::from(10u64.pow(decimals as u32));
    Ok(value_decimal / divisor)
}

/// Convert Decimal amount to U256 wei/token units
pub fn decimal_to_u256(value: Decimal, decimals: u8) -> Result<U256> {
    let multiplier = Decimal::from(10u64.pow(decimals as u32));
    let wei_value = value * multiplier;

    // Round to avoid fractional wei
    let wei_str = wei_value.round().to_string();

    // Remove decimal point if present
    let wei_str = wei_str.split('.').next().unwrap_or(&wei_str);

    U256::from_dec_str(wei_str)
        .map_err(|e| anyhow!("Failed to convert Decimal to U256: {}", e))
}

/// Parse a string amount to U256 with decimals
pub fn parse_amount(amount_str: &str, decimals: u8) -> Result<U256> {
    let amount = Decimal::from_str(amount_str)
        .map_err(|e| anyhow!("Invalid amount format: {}", e))?;
    decimal_to_u256(amount, decimals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u256_to_decimal_eth() {
        let one_eth = U256::from_dec_str("1000000000000000000").unwrap();
        let decimal = u256_to_decimal(one_eth, 18).unwrap();
        assert_eq!(decimal, Decimal::from(1));
    }

    #[test]
    fn test_u256_to_decimal_usdc() {
        let one_usdc = U256::from_dec_str("1000000").unwrap();
        let decimal = u256_to_decimal(one_usdc, 6).unwrap();
        assert_eq!(decimal, Decimal::from(1));
    }

    #[test]
    fn test_decimal_to_u256_eth() {
        let one = Decimal::from(1);
        let u256 = decimal_to_u256(one, 18).unwrap();
        assert_eq!(u256, U256::from_dec_str("1000000000000000000").unwrap());
    }

    #[test]
    fn test_decimal_to_u256_usdc() {
        let one = Decimal::from(1);
        let u256 = decimal_to_u256(one, 6).unwrap();
        assert_eq!(u256, U256::from_dec_str("1000000").unwrap());
    }

    #[test]
    fn test_parse_amount() {
        let amount = parse_amount("1.5", 18).unwrap();
        assert_eq!(amount, U256::from_dec_str("1500000000000000000").unwrap());
    }
}
