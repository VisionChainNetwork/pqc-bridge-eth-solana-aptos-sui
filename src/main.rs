mod bridge;
mod config;
mod consensus;
mod crypto;
mod db;
mod evm;
mod node;
mod p2p;
mod rpc;
mod types;

use crate::{
    bridge::BridgeManager,
    config::NodeConfig,
    consensus::NarwhalBullsharkEngine,
    db::ChainStore,
    evm::EvmExecutor,
    node::NodeRuntime,
    rpc::{spawn_rpc, EthApiImpl},
};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // In real life load from a config file / CLI
    let cfg = NodeConfig::default();

    // Chain store
    let store = Arc::new(ChainStore::open(&cfg.rocksdb_path)?);

    // EVM executor
    let executor = Arc::new(EvmExecutor::new(cfg.chain_id));

    // Bridges
    let bridge = Arc::new(BridgeManager::new(cfg.bridges.clone()));

    // Channels:
    // 1. P2P/RPC → Consensus
    let (consensus_tx, consensus_rx) = mpsc::channel(1024);
    // 2. Consensus → NodeRuntime (executors + bridges)
    let (cons_out_tx, cons_out_rx) = mpsc::channel(1024);

    // Spawn P2P
    p2p::spawn_p2p(&cfg.libp2p_listen, consensus_tx.clone()).await?;

    // Spawn consensus
    let engine = NarwhalBullsharkEngine::new(
        store.clone(),
        consensus_rx,
        cons_out_tx,
        "validator-0".to_string(),
        cfg.target_tps,
        cfg.block_time_ms,
    );
    tokio::spawn(async move {
        if let Err(e) = engine.run().await {
            eprintln!("Consensus engine failed: {e:?}");
        }
    });

    // Spawn JSON-RPC
    let api_impl = EthApiImpl::new(store.clone(), consensus_tx);
    let _rpc_handle = spawn_rpc(cfg.rpc_listen, api_impl).await?;

    // Spawn node runtime (execute committed blocks + bridge)
    let runtime = NodeRuntime::new(executor.clone(), cons_out_rx, bridge.clone());
    tokio::spawn(async move {
        if let Err(e) = runtime.run().await {
            eprintln!("Node runtime failed: {e:?}");
        }
    });

    // Keep running
    futures::future::pending::<()>().await;
    Ok(())
}

