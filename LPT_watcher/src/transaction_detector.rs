use web3::futures::Future;
use web3::types::{Log, TransactionId, U256};

use crate::initialization::Web3Itf;

pub struct TransactionDetector {
    // Discord credentials
}

impl TransactionDetector {
    pub fn new() -> Self {
        Self {}
    }

    pub fn has_valid_transaction_been_made(&self, web3: &Web3Itf, log: Log) -> bool {
        // If there is no transaction hash we do as if the transaction was not done
        let transaction_hash = match log.transaction_hash {
            Some(hash) => hash,
            None => return false,
        };

        // Get transaction info and prints it
        let transaction = match web3
            .eth()
            .transaction(TransactionId::Hash(transaction_hash))
            .wait()
        {
            Ok(transaction) => match transaction {
                Some(transaction) => transaction,
                None => return false,
            },
            Err(_) => return false,
        };

        println!("Transaction done");
        println!("Value {} Gas {}", transaction.value, transaction.gas);

        transaction.value != U256::zero()
    }
}
