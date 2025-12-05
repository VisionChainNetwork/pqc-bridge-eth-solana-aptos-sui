use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub node_key_seed: Option<String>,
    pub libp2p_listen: String,
    pub rpc_listen: SocketAddr,
    pub rocksdb_path: String,
    pub chain_id: u64,
    pub target_tps: u64,
    pub block_time_ms: u64,
    pub validators: Vec<ValidatorConfig>,
    pub bridges: BridgeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    pub id: String,
    pub stake: u64,
    /// ML-DSA public key (bytes, hex encoded in config)
    pub pq_pubkey_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    pub solana_rpc_url: String,
    pub sui_rpc_url: String,
    pub aptos_rpc_url: String,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            node_key_seed: None,
            libp2p_listen: "/ip4/0.0.0.0/tcp/7000".to_string(),
            rpc_listen: "0.0.0.0:8545".parse().unwrap(),
            rocksdb_path: "data/chain.db".to_string(),
            chain_id: 1337,
            target_tps: 10_000,
            block_time_ms: 100, // 100ms * ~1000 tx/block ≈ 10k TPS target
            validators: vec![],
            bridges: BridgeConfig {
                solana_rpc_url: "https://api.devnet.solana.com".to_string(),
                sui_rpc_url: "https://fullnode.testnet.sui.io:443".to_string(),
                aptos_rpc_url: "https://fullnode.testnet.aptoslabs.com/v1".to_string(),
            },
        }
    }
}

