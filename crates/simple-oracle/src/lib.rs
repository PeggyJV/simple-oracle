use std::{collections::HashMap, sync::mpsc};

use ethers::types::{Address, U256};
use eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

mod querier;
mod oracle;
mod utils;

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

pub async fn start(config: &Config) -> Result<()> {
    info!("starting application");

    let (tx, rx) = mpsc::sync_channel(config.assets.len());

    let mut querier = querier::Querier::new(config.to_owned(), tx)?;
    tokio::spawn(async move { querier.run().await });

    let mut oracle = oracle::Oracle::new(config, rx)?;
    oracle.run().await;

    info!("application stopping");

    Ok(())
}
