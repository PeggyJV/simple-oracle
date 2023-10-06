use std::{
    collections::HashSet,
    sync::{Arc, mpsc}
};

use ethers::{
    providers::{Http, Provider},
    types::U256
};
use eyre::Result;

use crate::{Asset, Config, QuotePrice};

#[derive(Clone, Debug)]
pub struct Querier {
    sender: mpsc::Sender<QuotePrice>,
    client: Arc<Provider<Http>>,
    assets: HashSet<Asset>,
}

impl Querier {
    pub(crate) fn new(config: &Config, sender: mpsc::Sender<QuotePrice>, client: Provider<Http>) -> Result<Self> {
        let provider = Provider::<Http>::try_from(config.ethereum_rpc_url.as_str())?;

        Ok(Self { sender, client: Arc::new(provider), assets: config.assets.clone() })
    }

    pub fn run(&self) -> Result<()> {
        loop {
            let asset = Asset::ETH;
            let redemption_rate = self.get_redemption_rate(asset)?;
            let quote = QuotePrice::new(asset, redemption_rate);

            self.send(quote)?;
        }
    }

    /// Calls previewRedeem(uint265) on the Cellar contract and returns a [QuotePrice]
    pub(crate) async fn get_redemption_rate(&self, asset: Asset) -> Result<U256> {
        let contract = erc4626::ERC4626::new(asset.ethereum_contract, self.client.clone());
        let unit = U256::from(1 * 10u64.pow(asset.decimals));

        Ok(contract.preview_redeem(unit).call().await?) 
    }

    /// Pushes the [QuotePrice] to the channel
    pub(crate) fn send(&self, quote: QuotePrice) -> Result<()> {
        self.sender.send(quote)?;

        Ok(())
    }
}
