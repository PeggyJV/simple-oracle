use std::{str::FromStr, time};

use cosmwasm_std::Decimal256;
use ethers::types::{Address, U256};
use eyre::Result;

/// Returns the current unix time in milliseconds
pub fn unix_now() -> u64 {
    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

/// Converts an H160 to human readable format
pub fn format_ethereum_address(address: Address) -> String {
    format!(
        "0x{}",
        address
            .as_bytes()
            .iter()
            .map(|b| format!("{:0>2x?}", b))
            .fold(String::new(), |acc, x| acc + &x)
    )
}

/// Converts a U256 to a Decimal256
pub fn u256_to_decimal(value: U256) -> Result<Decimal256> {
    let mut value = value.to_string();

    value.push_str(".0");

    Ok(Decimal256::from_str(&value)?)
}
