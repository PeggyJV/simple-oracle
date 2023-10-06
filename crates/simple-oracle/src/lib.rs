use std::{
    collections::HashMap,
    sync::mpsc,
};

use eyre::Result;
use ethers::{
    abi::Address,
    providers::{Http, Provider},
};
use serde::{Deserialize, Serialize};

mod querier;
mod tx;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub ethereum_rpc_url: String,
    pub osmosis_grpc_url: String,
    pub contract_map: HashMap<String, String>,
    pub signing_key_path: String,
    pub assets: Vec<Asset>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Asset {
    pub ethereum_contract: Address,
    pub decimals: u32,
}

#[derive(Clone, Debug)]
pub struct QuotePrice {
    pub asset: Asset,
    pub value: u64,
    pub base: String,
    pub quote: String,
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

