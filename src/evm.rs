use crate::types::{Block, HybridTx};
use anyhow::Result;
use revm::{
    db::EmptyDB,
    primitives::{Address, TransactTo, TxEnv, U256},
    EVM,
};

use std::sync::Mutex;

pub struct EvmExecutor {
    inner: Mutex<Evm<'static, (), EmptyDB>>,
    chain_id: u64,
}

impl EvmExecutor {
    pub fn new(chain_id: u64) -> Self {

        let db = EmptyDB::default();
        let mut evm = EVM::new();
        evm.database(db);


        Self {
            inner: Mutex::new(evm),
            chain_id,
        }
    }

    pub fn execute_block(&self, block: &Block) -> Result<Vec<ExecutionResult>> {
        let mut evm = self.inner.lock().unwrap();

        let mut results = Vec::with_capacity(block.txs.len());

        for tx in &block.txs {
            let mut tx_env = TxEnv::default();
            tx_env.caller = tx.from;
            tx_env.transact_to = match tx.to {
                Some(to) => TransactTo::Call(to),
                None => TransactTo::Create,
            };
            tx_env.data = tx.data.clone();
            tx_env.value = tx.value;
            tx_env.nonce = tx.nonce;
            tx_env.gas_limit = tx.gas_limit;
            tx_env.gas_price = tx.max_fee_per_gas; // simplification
            tx_env.chain_id = Some(U256::from(self.chain_id));

            evm.context.evm.env.tx = tx_env;

            let out = evm.transact_commit()?; // commit state into DB
            results.push(out);
        }

        Ok(results)
    }
}

