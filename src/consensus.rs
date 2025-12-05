use crate::crypto::verify_pq_tx;
use crate::types::{
    Block, BlockHeader, ConsensusInput, ConsensusOutput, HybridTx, NarwhalBatch,
};
use crate::db::ChainStore;
use anyhow::Result;
use revm::primitives::{B256, U256};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct NarwhalBullsharkEngine {
    store: Arc<ChainStore>,
    input_rx: Receiver<ConsensusInput>,
    output_tx: Sender<ConsensusOutput>,
    validator_id: String,
    target_tps: u64,
    block_time_ms: u64,
    dag: HashMap<u64, Vec<NarwhalBatch>>, // round -> batches
    pending_txs: Vec<HybridTx>,
}

impl NarwhalBullsharkEngine {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        store: Arc<ChainStore>,
        input_rx: Receiver<ConsensusInput>,
        output_tx: Sender<ConsensusOutput>,
        validator_id: String,
        target_tps: u64,
        block_time_ms: u64,
    ) -> Self {
        Self {
            store,
            input_rx,
            output_tx,
            validator_id,
            target_tps,
            block_time_ms,
            dag: HashMap::new(),
            pending_txs: Vec::new(),
        }
    }

    pub async fn run(mut self) -> Result<()> {
        let mut current_round: u64 = 0;

        loop {
            tokio::select! {
                Some(msg) = self.input_rx.recv() => {
                    match msg {
                        ConsensusInput::NewTx(tx) => {
                            // basic PQ check (optional)
                            let _ = verify_pq_tx(&tx);
                            self.pending_txs.push(tx);
                        }
                        ConsensusInput::NarwhalBatch(batch) => {
                            self.dag.entry(batch.round).or_default().push(batch);
                        }
                    }
                }
                _ = tokio::time::sleep(
                    std::time::Duration::from_millis(self.block_time_ms)
                ) => {
                    // produce a new round/batch from local pending txs
                    current_round += 1;
                    if !self.pending_txs.is_empty() {
                        let batch = self.build_local_batch(current_round);
                        self.dag.entry(current_round).or_default().push(batch);
                    }

                    if let Some(block) = self.bullshark_commit(current_round).await? {
                        self.store.put_block(&block)?;
                        self.store.put_head(block.header.number, block.header.hash.0)?;
                        self.output_tx.send(ConsensusOutput::CommittedBlock(block)).await?;
                    }
                }
            }
        }
    }

    fn build_local_batch(&mut self, round: u64) -> NarwhalBatch {
        let txs = std::mem::take(&mut self.pending_txs);
        NarwhalBatch {
            id: uuid::Uuid::new_v4(),
            round,
            author: self.validator_id.clone(),
            parents: self
                .dag
                .get(&(round.saturating_sub(1)))
                .map(|batches| batches.iter().map(|b| b.id).collect())
                .unwrap_or_default(),
            txs,
        }
    }

    /// Extremely simplified Bullshark: if we have any batches for the last 3
    /// rounds, we “commit” them in topological order to form a block.
    async fn bullshark_commit(&self, current_round: u64) -> Result<Option<Block>> {
        if current_round < 3 {
            return Ok(None);
        }
        let commit_round = current_round - 2;

        let mut batches: Vec<&NarwhalBatch> =
            self.dag.get(&commit_round).map(|v| v.iter().collect()).unwrap_or_default();

        if batches.is_empty() {
            return Ok(None);
        }

        // Deterministic order: sort by UUID bytes.
        batches.sort_by_key(|b| b.id.as_u128());

        let mut all_txs = Vec::new();
        for b in batches {
            all_txs.extend(b.txs.clone());
        }

        // Construct block header
        let parent_header = self.store.get_head_header()?;
        let number = parent_header.as_ref().map(|h| h.number + 1).unwrap_or(0);
        let parent_hash = parent_header
            .as_ref()
            .map(|h| h.hash)
            .unwrap_or(B256::ZERO);

        let tx_root = self.compute_fake_root(&all_txs);
        let state_root = B256::ZERO; // would come from EVM executor after commit

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let header = BlockHeader {
            number,
            hash: self.compute_block_hash(number, parent_hash, &tx_root),
            parent_hash,
            state_root,
            tx_root,
            timestamp: ts,
        };

        Ok(Some(Block { header, txs: all_txs }))
    }

    fn compute_fake_root(&self, txs: &[HybridTx]) -> B256 {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        for tx in txs {
            hasher.update(&tx.hash.0);
        }
        let out = hasher.finalize();
        B256::from_slice(&out)
    }

    fn compute_block_hash(&self, number: u64, parent_hash: B256, tx_root: &B256) -> B256 {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(number.to_be_bytes());
        hasher.update(parent_hash.0);
        hasher.update(tx_root.0);
        let out = hasher.finalize();
        B256::from_slice(&out)
    }
}

