/// Logic for signing and sending Execute transactions to CosmWasm contract
use std::{borrow::BorrowMut, collections::HashMap, str::FromStr, sync::mpsc};

use cosmos_sdk_proto::{cosmwasm::wasm::v1::MsgExecuteContract, traits::Message};
use cosmwasm_std::Timestamp;
use ethers::types::Address;
use eyre::Result;
use ocular::{
    account::AccountInfo,
    chain::ChainContext,
    cosmrs::{Any, Coin, Denom},
    tx::{FeeInfo, UnsignedTx},
    MsgClient, QueryClient,
};

use crate::{format_ethereum_address, u256_to_decimal, Config, QuotePrice};

pub struct Oracle {
    rx: mpsc::Receiver<QuotePrice>,
    contract_map: HashMap<Address, String>,
    signer: AccountInfo,
    grpc_url: String,
}

impl Oracle {
    pub async fn new(config: &Config, rx: mpsc::Receiver<QuotePrice>) -> Result<Self> {
        let signer = AccountInfo::from_pem(&config.signing_key_path)?;
        Ok(Self {
            rx,
            contract_map: config.contract_map.clone(),
            signer,
            grpc_url: config.osmosis_grpc_url.clone(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            match self.rx.recv() {
                Ok(quote) => {
                    if let Err(err) = self.submit_quote(quote).await {
                        //log::error!("Error sending quote: {}", err);
                    }
                }
                Err(err) => {
                    //log::error!("Error receiving quote: {}", err);
                }
            }
        }
    }

    pub async fn submit_quote(&mut self, quote: QuotePrice) -> Result<()> {
        let contract = self
            .contract_map
            .get(&quote.asset.ethereum_contract)
            .unwrap();

        let inner_msg = simple_oracle::msg::ExecuteMsg::SetPrice {
            value: u256_to_decimal(quote.value)?,
            timestamp: Some(Timestamp::from_seconds(quote.timestamp)),
        };

        let msg = MsgExecuteContract {
            sender: self.signer.address("osmo")?,
            contract: contract.to_owned(),
            msg: serde_json::to_vec(&inner_msg)?,
            // ????
            funds: vec![],
        };

        Ok(self.sign_and_send(msg).await?)
    }

    pub async fn sign_and_send(&mut self, msg: MsgExecuteContract) -> Result<()> {
        let mut qclient = QueryClient::new(&self.grpc_url)?;

        let mut fee_info = FeeInfo::new(Coin {
            denom: Denom::from_str("uosmo")?,
            amount: 20_000,
        });
        fee_info.gas_limit(1_000_000);

        let chain_context = ChainContext {
            id: String::from("osmosis-1"),
            prefix: "osmo".to_string(),
        };

        let msg = Any {
            type_url: "/cosmwasm.wasm.v1.MsgExecuteContract".to_string(),
            value: msg.encode_to_vec(),
        };

        let mut unsigned = UnsignedTx::new();
        unsigned.add_msg(msg);

        let signed = unsigned
            .sign(&self.signer, fee_info, &chain_context, &mut qclient)
            .await?;

        let mut mclient = MsgClient::new(&self.grpc_url)?;

        signed.broadcast_commit(&mut mclient).await?;

        Ok(())
    }
}
