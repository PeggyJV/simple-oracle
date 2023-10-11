use std::{
    collections::{HashMap, HashSet},
    ops::Div,
    str::FromStr,
    sync::{mpsc, Arc},
    time::Duration,
};

use cosmwasm_std::Decimal256;
use ethers::{
    providers::{Http, Provider},
    types::{Address, U256},
};
use eyre::Result;
use tokio::time::{Instant, MissedTickBehavior};
use tracing::{error, info, trace};

use crate::{utils::*, Asset, Config, QuotePrice};

const DEFAULT_PRICE_VARIANCE_THRESHOLD: f64 = 0.0025;
const DEFAULT_SUBMISSION_PERIOD: u64 = 300;
const DEFAULT_VARIANCE_CHECK_PERIOD: u64 = 15;
const DEFAULT_MIN_TIME_BETWEEN_QUOTES: u64 = 6;

/// Handles querying for the current redemption rate of an asset and submitting it to the oracle
/// under certain conditions.
#[derive(Clone, Debug)]
pub struct Querier {
    sender: mpsc::SyncSender<QuotePrice>,
    client: Arc<Provider<Http>>,
    assets: HashSet<Asset>,
    last_submitted: HashMap<Address, QuotePrice>,
    price_variance_threshold: f64,
    check_variance_period: u64,
    submission_period: u64,
    min_time_between_quotes: u64,
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
            last_submitted: HashMap::new(),
            price_variance_threshold: if config.price_variance_threshold == 0f64 {
                DEFAULT_PRICE_VARIANCE_THRESHOLD
            } else {
                config.price_variance_threshold
            },
            check_variance_period: if config.check_variance_period == 0 {
                DEFAULT_VARIANCE_CHECK_PERIOD
            } else {
                config.check_variance_period
            },
            submission_period: if config.submission_period == 0 {
                DEFAULT_SUBMISSION_PERIOD
            } else {
                config.submission_period
            },
            min_time_between_quotes: if config.min_time_between_quotes == 0 {
                DEFAULT_MIN_TIME_BETWEEN_QUOTES
            } else {
                config.min_time_between_quotes
            },
        })
    }

    /// Queries for redemption rates every interval and only submits them to the oracle
    /// thread if either the configured refresh period has elapsed or a redemption rate has
    /// a significant delta compared to the previous value.
    pub async fn run(&mut self) -> Result<()> {
        let variance_check_period = Duration::from_secs(self.check_variance_period);
        let variance_check_start = Instant::now().checked_add(variance_check_period).unwrap();
        let mut variance_check_interval =
            tokio::time::interval_at(variance_check_start, variance_check_period);
        let mut periodic_submission_interval =
            tokio::time::interval(Duration::from_secs(self.submission_period));

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

    /// Queries and evaluates quotes for submission to the oracle thread
    pub async fn handle_quote(&mut self, check_variance: bool) -> Result<()> {
        info!("getting latest redemption rate quotes");

        for asset in self.assets.iter() {
            let Ok(quote) = self.get_quote(asset).await else {
                error!("failed to get quote for {}/{}", asset.quote, asset.base);
                continue;
            };

            let Some(prev) = self.last_submitted.get(&asset.ethereum_contract) else {
                self.send(quote.clone())?;

                self.last_submitted.insert(asset.ethereum_contract, quote);

                continue;
            };

            // if this is a variance check quote, we only submit if the change in value is
            // greater than the configured variance threshold.
            if check_variance && !self.significant_change(quote.value, prev.value) {
                continue;
            }

            // avoid sending two quotes one after the other due to different intervals
            if quote.timestamp - prev.timestamp < self.min_time_between_quotes {
                continue;
            }

            trace!("sending quote to oracle thread: {:?}", quote);

            if let Err(err) = self.send(quote.clone()) {
                error!("failed to send quote to oracle thread: {}", err);
            }

            self.last_submitted.insert(asset.ethereum_contract, quote);
        }

        Ok(())
    }

    /// Wraps the raw previewRedeem output in a [QuotePrice]
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
    pub(crate) async fn get_redemption_rate(&self, asset: &Asset) -> Result<Decimal256> {
        let contract = erc4626::ERC4626::new(asset.ethereum_contract, self.client.clone());
        let unit = U256::from(10u64.pow(asset.decimals));
        let rr = contract.preview_redeem(unit).call().await?;

        convert_u256(rr, asset.decimals)
    }

    /// Checks if the delta is greater than a 0.25% change
    pub fn significant_change(&self, current: Decimal256, previous: Decimal256) -> bool {
        let delta = current.abs_diff(previous).div(previous);
        let threshold = Decimal256::from_str(&self.price_variance_threshold.to_string()).unwrap();

        delta > threshold
    }

    /// Pushes the [QuotePrice] to the channel
    pub(crate) fn send(&self, quote: QuotePrice) -> Result<()> {
        self.sender.send(quote)?;

        Ok(())
    }
}
