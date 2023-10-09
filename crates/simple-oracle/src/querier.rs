use std::{
    collections::HashSet,
    sync::{mpsc, Arc},
};

use ethers::{
    providers::{Http, Provider},
    types::U256,
};
use eyre::Result;

use crate::{unix_now, Asset, Config, QuotePrice};

#[derive(Clone, Debug)]
pub struct Querier {
    sender: mpsc::SyncSender<QuotePrice>,
    client: Arc<Provider<Http>>,
    assets: HashSet<Asset>,
}

impl Querier {
    pub(crate) fn new(config: Config, sender: mpsc::SyncSender<QuotePrice>) -> Result<Self> {
        let provider = Provider::<Http>::try_from(config.ethereum_rpc_url.as_str())?;
        let mut assets = HashSet::new();

        for a in config.assets.iter() {
            assets.insert(a.clone());
        }

        Ok(Self {
            sender,
            client: Arc::new(provider),
            assets,
        })
    }

    pub async fn run(&self) -> Result<()> {
        loop {
            for asset in self.assets.iter() {
                let Ok(redemption_rate) = self.get_redemption_rate(asset).await else {
                    //error!("failed to get redemption rate for {:?}: {}", asset, e);
                    continue;
                };

                self.send(QuotePrice {
                    asset: asset.clone(),
                    value: redemption_rate,
                    timestamp: unix_now(),
                })?;
            }

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }

    /// Calls previewRedeem(uint265) on the Cellar contract and returns a [QuotePrice]
    pub(crate) async fn get_redemption_rate(&self, asset: &Asset) -> Result<U256> {
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
