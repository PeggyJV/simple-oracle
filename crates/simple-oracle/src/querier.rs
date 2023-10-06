use std::sync::{Arc, mpsc};

use ethers::{
    providers::{Http, Provider},
    types::U256
};
use eyre::Result;

use crate::{Asset, QuotePrice};

#[derive(Clone, Debug)]
pub struct Querier {
    sender: mpsc::Sender<QuotePrice>,
    client: Arc<Provider<Http>>,
}

impl Querier {
    pub(crate) fn new(sender: mpsc::Sender<QuotePrice>, client: Provider<Http>) -> Self {
        Self { sender, client: Arc::new(client) }
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
