use std::{ops::Div, str::FromStr, time};

use cosmwasm_std::Decimal256;
use ethers::types::U256;
use eyre::Result;

/// Returns the current unix time in milliseconds
pub fn unix_now() -> u64 {
    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Converts a U256 to a Decimal256
pub fn convert_u256(value: U256, decimals: u32) -> Result<Decimal256> {
    let value = value.as_u128();

    Ok(Decimal256::from_atomics(value, decimals)?)
}

/// Checks if the delta is greater than a 0.25% change
pub fn significant_change(current: Decimal256, previous: Decimal256) -> bool {
    let delta = current.abs_diff(previous).div(previous);
    let threshold = Decimal256::from_str("0.0025").unwrap();

    delta > threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_u256() {
        let value = U256::from_dec_str("1000000000000000000").unwrap();
        let result = convert_u256(value, 18).unwrap();

        assert_eq!(Decimal256::from_str("1.0").unwrap(), result);

        let value: u128 = 1_000_000;
        let value = U256::from(value);
        let result = convert_u256(value, 6).unwrap();
        assert_eq!(Decimal256::from_str("1.0").unwrap(), result);
    }

    #[test]
    fn test_significant_change() {
        let current = Decimal256::from_str("1.0").unwrap();
        let previous = Decimal256::from_str("1.1").unwrap();

        assert!(significant_change(current, previous));

        let previous = Decimal256::from_str("1.00024").unwrap();
        assert!(!significant_change(current, previous));
    }
}
