use web3::futures::Future;
use web3::types::{Address, BlockHeader, BlockId, Transaction};

use crate::initialization::Web3Itf;

pub struct TransactionDetector {
    address: Address,
}

impl TransactionDetector {
    pub fn new(address: Address) -> Self {
        Self { address }
    }

    pub fn contains_reward_transaction(
        &self,
        web3: &Web3Itf,
        block_header: BlockHeader,
    ) -> Option<Transaction> {
        // Get transactions list in block

        let block_id = BlockId::Hash(block_header.hash.expect("No block hash"));
        self.tries_to_get_transaction(web3, block_id)
            .and_then(|transactions| {
                // Check each transaction for requested address
                transactions
                    .iter()
                    .find(|tx| {
                        tx.from == self.address || tx.to.map_or(false, |to| to == self.address)
                    })
                    .cloned()
            })
    }

    fn tries_to_get_transaction(
        &self,
        web3: &Web3Itf,
        block_id: BlockId,
    ) -> Option<Vec<Transaction>> {
        // Tries 5 times to get block content
        for _ in 0..5 {
            if let Some(content) = web3
                .eth()
                .block_with_txs(block_id.clone())
                .wait()
                .ok()
                .unwrap()
            {
                return Some(content.transactions);
            }
        }
        None
    }
}
