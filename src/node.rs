use crate::{
    bridge::BridgeManager,
    consensus::{NarwhalBullsharkEngine},
    db::ChainStore,
    evm::EvmExecutor,
    types::ConsensusOutput,
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver};

pub struct NodeRuntime {
    executor: Arc<EvmExecutor>,
    consensus_output_rx: Receiver<ConsensusOutput>,
    bridge: Arc<BridgeManager>,
}

impl NodeRuntime {
    pub fn new(
        executor: Arc<EvmExecutor>,
        consensus_output_rx: Receiver<ConsensusOutput>,
        bridge: Arc<BridgeManager>,
    ) -> Self {
        Self {
            executor,
            consensus_output_rx,
            bridge,
        }
    }

    pub async fn run(mut self) -> Result<()> {
        while let Some(msg) = self.consensus_output_rx.recv().await {
            match msg {
                ConsensusOutput::CommittedBlock(block) => {
                    // Execute block on EVM (state updates).
                    let _results = self.executor.execute_block(&block)?;

                    // In a full node you’d now compute and persist state_root, receipts, etc.

                    // Notify bridges (fire-and-forget style).
                    let bridge = self.bridge.clone();
                    let block_clone = block.clone();
                    tokio::spawn(async move {
                        let _ = bridge.notify_solana(&block_clone).await;
                        let _ = bridge.notify_sui(&block_clone).await;
                        let _ = bridge.notify_aptos(&block_clone).await;
                    });
                }
            }
        }
        Ok(())
    }
}

