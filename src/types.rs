use revm::primitives::{Address, B256, Bytes, U256};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Simplified transaction with optional PQ metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridTx {
    pub hash: B256,
    pub from: Address,
    pub to: Option<Address>,
    pub nonce: U256,
    pub gas_limit: u64,
    pub max_fee_per_gas: U256,
    pub max_priority_fee_per_gas: U256,
    pub value: U256,
    pub data: Bytes,
    pub chain_id: u64,

    /// Standard Ethereum ECDSA sig (r,s,v) in 65-byte form.
    pub sig: Option<Bytes>,

    /// Optional PQ signature (ML-DSA) + public key.
    pub pq_sig: Option<Vec<u8>>,
    pub pq_pubkey: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub number: u64,
    pub hash: B256,
    pub parent_hash: B256,
    pub state_root: B256,
    pub tx_root: B256,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub txs: Vec<HybridTx>,
}

/// Narwhal “batch” node in the DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarwhalBatch {
    pub id: Uuid,
    pub round: u64,
    pub author: String, // validator id
    pub parents: Vec<Uuid>,
    pub txs: Vec<HybridTx>,
}

/// Consensus events sent from P2P to consensus engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusInput {
    NewTx(HybridTx),
    NarwhalBatch(NarwhalBatch),
}

/// Outputs of consensus into the executor / block pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusOutput {
    CommittedBlock(Block),
}

