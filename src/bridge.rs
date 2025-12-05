use crate::types::Block;
use crate::config::BridgeConfig;
use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use tracing::info;

pub struct BridgeManager {
    cfg: BridgeConfig,
    client: Client,
}

impl BridgeManager {
    pub fn new(cfg: BridgeConfig) -> Self {
        Self {
            cfg,
            client: Client::new(),
        }
    }

    /// Example: notify Solana about a committed block (dummy payload).
    pub async fn notify_solana(&self, block: &Block) -> Result<()> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "bridge_notifyEthBlock",
            "params": [{
                "number": block.header.number,
                "hash": format!("0x{}", hex::encode(block.header.hash.0)),
            }]
        });

        let resp = self.client
            .post(&self.cfg.solana_rpc_url)
            .json(&payload)
            .send()
            .await?;

        info!("Solana bridge response: {:?}", resp.status());
        Ok(())
    }

    pub async fn notify_sui(&self, block: &Block) -> Result<()> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "bridge_notifyEthBlock",
            "params": [{
                "number": block.header.number,
                "hash": format!("0x{}", hex::encode(block.header.hash.0)),
            }]
        });

        let resp = self.client
            .post(&self.cfg.sui_rpc_url)
            .json(&payload)
            .send()
            .await?;

        info!("Sui bridge response: {:?}", resp.status());
        Ok(())
    }

    pub async fn notify_aptos(&self, block: &Block) -> Result<()> {
        // Aptos tends to use REST+JSON; we fake a simple endpoint.
        let url = format!("{}/bridge/eth_block", self.cfg.aptos_rpc_url);
        let payload = json!({
            "number": block.header.number,
            "hash": format!("0x{}", hex::encode(block.header.hash.0)),
        });

        let resp = self.client.post(&url).json(&payload).send().await?;
        info!("Aptos bridge response: {:?}", resp.status());
        Ok(())
    }
}

