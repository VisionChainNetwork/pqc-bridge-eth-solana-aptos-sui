use crate::db::ChainStore;
use crate::types::{HybridTx};
use anyhow::Result;
use jsonrpsee::{
    core::RpcResult,
    http_server::{HttpServerBuilder, HttpServerHandle},
    proc_macros::rpc,
};
use revm::primitives::B256;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::mpsc::Sender;
use crate::types::ConsensusInput;

#[rpc(server)]
pub trait EthApi {
    /// eth_blockNumber
    #[method(name = "eth_blockNumber")]
    async fn block_number(&self) -> RpcResult<String>;

    /// eth_getBlockByNumber (simplified, no tx details)
    #[method(name = "eth_getBlockByNumber")]
    async fn get_block_by_number(&self, number_hex: String, _full: bool) -> RpcResult<Option<serde_json::Value>>;

    /// eth_sendRawTransaction – here we cheat and accept a JSON HybridTx
    /// encoded as hex(bincode(HybridTx)).
    #[method(name = "eth_sendRawTransaction")]
    async fn send_raw_transaction(&self, tx_hex: String) -> RpcResult<String>;
}

pub struct EthApiImpl {
    store: Arc<ChainStore>,
    consensus_tx: Sender<ConsensusInput>,
}

impl EthApiImpl {
    pub fn new(store: Arc<ChainStore>, consensus_tx: Sender<ConsensusInput>) -> Self {
        Self { store, consensus_tx }
    }
}

#[jsonrpsee::core::async_trait]
impl EthApiServer for EthApiImpl {
    async fn block_number(&self) -> RpcResult<String> {
        let n = self.store.get_head_number().map_err(to_rpc_err)?;
        Ok(format!("0x{:x}", n))
    }

    async fn get_block_by_number(
        &self,
        number_hex: String,
        _full: bool,
    ) -> RpcResult<Option<serde_json::Value>> {
        let n = u64::from_str_radix(number_hex.trim_start_matches("0x"), 16)
            .map_err(to_rpc_err)?;
        let block = self.store.get_block(n).map_err(to_rpc_err)?;
        Ok(block.map(|b| serde_json::json!({
            "number": format!("0x{:x}", b.header.number),
            "hash": format!("0x{}", hex::encode(b.header.hash.0)),
            "parentHash": format!("0x{}", hex::encode(b.header.parent_hash.0)),
            "timestamp": format!("0x{:x}", b.header.timestamp),
            "transactions": b.txs.iter().map(|tx| format!("0x{}", hex::encode(tx.hash.0))).collect::<Vec<_>>()
        })))
    }

    async fn send_raw_transaction(&self, tx_hex: String) -> RpcResult<String> {
        let bytes = hex::decode(tx_hex.trim_start_matches("0x"))
            .map_err(to_rpc_err)?;
        let tx: HybridTx = bincode::deserialize(&bytes).map_err(to_rpc_err)?;
        let hash_str = format!("0x{}", hex::encode(tx.hash.0));

        self.consensus_tx
            .send(ConsensusInput::NewTx(tx))
            .await
            .map_err(to_rpc_err)?;

        Ok(hash_str)
    }
}

fn to_rpc_err<E: std::fmt::Display>(e: E) -> jsonrpsee::core::Error {
    jsonrpsee::core::Error::Custom(e.to_string())
}

pub async fn spawn_rpc(
    addr: SocketAddr,
    api_impl: EthApiImpl,
) -> Result<HttpServerHandle> {
    let server = HttpServerBuilder::default().build(addr).await?;
    let mut module = EthApiServer::into_rpc(api_impl);
    let handle = server.start(module)?;
    Ok(handle)
}

