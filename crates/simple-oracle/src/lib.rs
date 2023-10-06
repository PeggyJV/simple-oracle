use std::collections::HashMap;

use eyre::Result;
use serde::{Deserialize, Serialize};

mod querier;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub ethereum_rpc_url: String,
    pub osmosis_grpc_url: String,
    pub contract_mappings: HashMap<String, String>,
    pub signing_key_path: String,
}

#[derive(Clone, Debug)]
pub(crate) struct Quote {
    // Collin: should maybe put the associated eth contract address in 
    // a wrapper struct?
    pub ethereum_contract: String,
    pub value: u64,
    pub base: String,
    pub quote: String,
    pub timestamp: u64,
}

/// Should crate a channel, passing the sender to a thread running a [Querier], and the receiver
/// to a thread running the tx submission logic. Tx thread will also need to construct the signing
/// key from the key path in [Config]
pub fn start(config: &Config) -> Result<()> {
    todo!();
}

