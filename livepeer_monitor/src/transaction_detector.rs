use web3::futures::Future;
use web3::types::{Log, Transaction, TransactionId};

use crate::initialization::Web3Itf;

pub struct TransactionDetector {}

impl TransactionDetector {
    pub fn new() -> Self {
        Self {}
    }

    pub fn exctract_transaction_from_log(&self, web3: &Web3Itf, log: Log) -> Option<Transaction> {
        eprintln!("Received element from stream");
        // If there is no transaction hash we do as if the transaction was not done
        log.transaction_hash.and_then(|tx_hash| {
            web3.eth()
                .transaction(TransactionId::Hash(tx_hash))
                .wait()
                .ok()
                .unwrap()
        })
    }
}
