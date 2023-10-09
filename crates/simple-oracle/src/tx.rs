/// Logic for signing and sending Execute transactions to CosmWasm contract
use std::{collections::HashMap, sync::mpsc};

use cosmos_sdk_proto::cosmwasm::wasm::v1::msg_client::MsgClient;
use eyre::Result;
use ocular::account::AccountInfo;
use tonic::transport::Channel;

use crate::{Config, QuotePrice};

pub struct Oracle {
    msg_client: MsgClient<Channel>,
    rx: mpsc::Receiver<QuotePrice>,
    contract_map: HashMap<String, String>,
    signer: AccountInfo,
}

impl Oracle {
    pub async fn new(config: &Config, rx: mpsc::Receiver<QuotePrice>) -> Result<Self> {
        let msg_client = MsgClient::connect(config.osmosis_grpc_url.clone()).await?;
        let signer = AccountInfo::from_pem(&config.signing_key_path)?;

        Ok(Self {
            msg_client,
            rx,
            contract_map: config.contract_map.clone(),
            signer,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            match self.rx.recv() {
                Ok(quote) => {
                    if let Err(err) = self.sign_and_send(quote).await {
                        //log::error!("Error sending quote: {}", err);
                    }
                }
                Err(err) => {
                    //log::error!("Error receiving quote: {}", err);
                }
            }
        }
    }

    pub async fn sign_and_send(&mut self, quote: QuotePrice) -> Result<()> {
        todo!();
    }
}
