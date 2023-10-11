/// Logic for signing and sending Execute transactions to CosmWasm contract
use std::{collections::HashMap, str::FromStr, sync::mpsc};

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
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::{Config, QuotePrice};

/// Handles signing and sending Execute::SetPrice transactions to CosmWasm contract
pub struct Oracle {
    rx: mpsc::Receiver<QuotePrice>,
    contract_map: HashMap<Address, String>,
    signer: Mutex<AccountInfo>,
    grpc_url: String,
}

impl Oracle {
    pub fn new(config: &Config, mnemonic: String, rx: mpsc::Receiver<QuotePrice>) -> Result<Self> {
        let signer = AccountInfo::from_mnemonic(&mnemonic, "")?;
        info!("oracle wallet is {}", signer.address("osmo")?);
        
        let signer = Mutex::new(signer);

        Ok(Self {
            rx,
            contract_map: config.contract_map.clone(),
            signer,
            grpc_url: config.osmosis_grpc_url.clone(),
        })
    }

    pub async fn run(&mut self) {
        while let Ok(quote) = self.rx.recv() {
            if let Err(err) = self.submit_quote(quote).await {
                error!("failed to submit quote: {err}");
                continue;
            }
        }

        error!("channel closed unexpectely");
    }

    pub async fn submit_quote(&mut self, quote: QuotePrice) -> Result<()> {
        let contract = self
            .contract_map
            .get(&quote.asset.ethereum_contract)
            .unwrap();

        let inner_msg = simple_oracle::msg::ExecuteMsg::SetPrice {
            value: quote.value,
            timestamp: Some(Timestamp::from_seconds(quote.timestamp)),
        };

        let msg = MsgExecuteContract {
            sender: self.signer.lock().await.address("osmo")?,
            contract: contract.to_owned(),
            msg: serde_json::to_vec(&inner_msg)?,
            // ????
            funds: vec![],
        };

        self.sign_and_send(msg).await
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

        let signer = &self.signer.lock().await;

        let signed = unsigned
            .sign(signer, fee_info, &chain_context, &mut qclient)
            .await?;

        let mut mclient = MsgClient::new(&self.grpc_url)?;

        signed.broadcast_commit(&mut mclient).await?;

        Ok(())
    }
}
