use std::sync::mpsc;

use ethers::providers::{Http, Provider};
use eyre::Result;

use crate::Quote;

#[derive(Clone, Debug)]
pub struct Querier {
    sender: mpsc::Sender<Quote>,
    client: Provider<Http>,
}

impl Querier {
    pub(crate) fn new(sender: mpsc::Sender<Quote>, client: Provider<Http>) -> Self {
        Self { sender, client }
    }

    /// Calls previewRedeem(uint265) on the Cellar contract and returns a [Price]
    pub(crate) fn get_price() -> Result<Quote> {
        todo!();
    }

    /// Pushes the [Price] to the channel
    pub(crate) fn send(&self, quote: Quote) -> Result<()> {
        todo!();
    }
}
