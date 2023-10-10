use std::{
    collections::{HashMap, HashSet},
    sync::{mpsc, Arc},
    time::Duration,
};

use ethers::{
    providers::{Http, Provider},
    types::{Address, U256},
};
use eyre::Result;
use tokio::time::MissedTickBehavior;
use tracing::{error, info, trace};

use crate::{utils::*, Asset, Config, QuotePrice};

// 5 minutes
const DEFAULT_SUBMISSION_PERIOD: u64 = 300;
const MIN_TIME_BETWEEN_QUOTES: u64 = 3;

#[derive(Clone, Debug)]
pub struct Querier {
    sender: mpsc::SyncSender<QuotePrice>,
    client: Arc<Provider<Http>>,
    assets: HashSet<Asset>,
    previous_quotes: HashMap<Address, QuotePrice>,
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
            previous_quotes: HashMap::new(),
        })
    }

    /// Queries for redemption rates every interval and only submits them to the oracle
    /// thread if either the configured refresh period has elapsed or a redemption rate has
    /// a significant delta compared to the previous value.
    pub async fn run(&mut self) -> Result<()> {
        let mut variance_check_interval = tokio::time::interval(Duration::from_secs(30));
        let mut periodic_submission_interval =
            tokio::time::interval(Duration::from_secs(DEFAULT_SUBMISSION_PERIOD));

        variance_check_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        periodic_submission_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            if let Err(err) = tokio::select! {
                _ = variance_check_interval.tick() => {
                    self.handle_quote(true).await
                },
                _ = periodic_submission_interval.tick() => {
                    self.handle_quote(false).await
                },
            } {
                error!("failed to retreive quotes: {err}");
            }
        }
    }

    pub async fn handle_quote(&mut self, check_variance: bool) -> Result<()> {
        info!("getting latest redemption rate quotes");

        for asset in self.assets.iter() {
            let Ok(quote) = self.get_quote(asset).await else {
                error!("failed to get quote for {}/{}", asset.quote, asset.base);
                continue;
            };

            let Some(prev) = self.previous_quotes.get(&asset.ethereum_contract) else {
                self.previous_quotes.insert(asset.ethereum_contract, quote.clone());

                self.send(quote)?;

                continue;
            };

            // if this is a variance check quote, we only submit if the change in value is
            // greater than 0.25%.
            if check_variance {
                let diff: U256;
                if quote.value > prev.value {
                    diff = quote.value - prev.value;
                } else {
                    diff = prev.value - quote.value;
                }

                // TODO: this is not a good check, u64 is too small. need to figure out how to do
                // do the math with the U256 type
                let change = diff.as_u64() as f64 / quote.value.as_u64() as f64;

                // if the change is less than 0.25% then we continue to wait for the regular
                // update interval to update this asset.
                if change < 0.0025 {
                    continue;
                }
            }

            // avoid sending two quotes one after the other due to different intervals
            if quote.timestamp - prev.timestamp < MIN_TIME_BETWEEN_QUOTES {
                continue;
            }

            trace!("sending quote to oracle thread: {:?}", quote);

            if let Err(err) = self.send(quote) {
                error!("failed to send quote to oracle thread: {}", err);
            }
        }

        Ok(())
    }

    pub async fn get_quote(&self, asset: &Asset) -> Result<QuotePrice> {
        trace!("getting redemption rate for {}/{}", asset.quote, asset.base);
        trace!(
            "calling previewRedeem on contract {}",
            asset.ethereum_contract
        );

        let rate = self.get_redemption_rate(asset).await?;

        Ok(QuotePrice {
            asset: asset.clone(),
            value: rate,
            timestamp: unix_now(),
        })
    }

    /// Calls previewRedeem(uint265) on the Cellar contract and returns the redemption rate
    pub(crate) async fn get_redemption_rate(&self, asset: &Asset) -> Result<U256> {
        let contract = erc4626::ERC4626::new(asset.ethereum_contract, self.client.clone());
        let unit = U256::from(10u64.pow(asset.decimals));

        Ok(contract.preview_redeem(unit).call().await?)
    }

    /// Pushes the [QuotePrice] to the channel
    pub(crate) fn send(&self, quote: QuotePrice) -> Result<()> {
        self.sender.send(quote)?;

        Ok(())
    }
}
