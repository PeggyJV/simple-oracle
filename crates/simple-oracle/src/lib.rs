use std::{collections::HashMap, str::FromStr, sync::mpsc, time};

use cosmwasm_std::Decimal256;
use ethers::types::{Address, U256};
use eyre::Result;
use serde::{Deserialize, Serialize};

mod querier;
mod tx;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub ethereum_rpc_url: String,
    pub osmosis_grpc_url: String,
    pub contract_map: HashMap<Address, String>,
    pub signing_key_path: String,
    pub assets: Vec<Asset>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Asset {
    pub ethereum_contract: Address,
    pub decimals: u32,
    pub base: String,
    pub quote: String,
}

#[derive(Clone, Debug)]
pub struct QuotePrice {
    pub asset: Asset,
    pub value: U256,
    pub timestamp: u64,
}

/// Should crate a channel, passing the sender to a thread running a [Querier], and the receiver
/// to a thread running the tx submission logic. Tx thread will also need to construct the signing
/// key from the key path in [Config]
pub fn start(config: &Config) -> Result<()> {
    let (tx, rx) = mpsc::channel();

    start_querier_thread(config, tx);
    start_tx_thread(config, rx);

    Ok(())
}

pub fn start_querier_thread(config: &Config, tx: mpsc::Sender<QuotePrice>) -> Result<()> {
    let config = config.clone();

    /// start thread
    Ok(())
}

pub fn start_tx_thread(config: &Config, rx: mpsc::Receiver<QuotePrice>) -> Result<()> {
    let config = config.clone();

    /// start thread
    Ok(())
}

pub fn unix_now() -> u64 {
    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

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

pub fn u256_to_decimal(value: U256) -> Result<Decimal256> {
    let mut value = value.to_string();

    value.push_str(".0");

    Ok(Decimal256::from_str(&value)?)
}
