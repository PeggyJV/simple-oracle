use std::time;

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
}
