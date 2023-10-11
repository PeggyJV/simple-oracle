use std::{collections::HashMap, sync::mpsc};

use cosmwasm_std::Decimal256;
use ethers::types::Address;
use eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

mod oracle;
mod querier;
mod utils;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub ethereum_rpc_url: String,
    pub osmosis_rpc_url: String,
    pub osmosis_grpc_url: String,
    pub price_variance_threshold: f64,
    pub check_variance_period: u64,
    pub submission_period: u64,
    pub min_time_between_quotes: u64,
    pub assets: Vec<Asset>,
    pub contract_map: HashMap<Address, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ethereum_rpc_url: "http://localhost:8545".to_string(),
            osmosis_rpc_url: "https://osmosis-rpc.polkachu.com:443".to_string(),
            osmosis_grpc_url: "grpc://osmosis-grpc.polkachu.com:12590".to_string(),
            price_variance_threshold: 0.0025,
            check_variance_period: 15,
            submission_period: 300,
            min_time_between_quotes: 6,
            assets: vec![],
            contract_map: HashMap::new(),
        }
    }
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
    pub value: Decimal256,
    pub timestamp: u64,
}

/// Entrypoint should call this to start the application
pub async fn start(config: &Config, mnemonic: String) -> Result<()> {
    info!("starting application");

    let (tx, rx) = mpsc::sync_channel(config.assets.len());

    let mut querier = querier::Querier::new(config.to_owned(), tx)?;
    tokio::spawn(async move { querier.run().await });

    let mut oracle = oracle::Oracle::new(config, mnemonic, rx)?;
    oracle.run().await;

    info!("application stopping");

    Ok(())
}
