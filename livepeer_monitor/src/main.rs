mod initialization;
mod round_detector;
mod transaction_detector;

use std::sync::atomic::Ordering;

use web3::futures::{Future, Stream};
use web3::types::U256;

use chrono::prelude::Utc;

use initialization::*;
use round_detector::RoundDetector;
use transaction_detector::TransactionDetector;

#[tokio::main]
async fn main() {
    // Loading config file and instanciating websocket transport
    let init = Init::load_config();

    // Subscribing to reward() transactions
    let mut reward_subscription = init.reward_call_subscription();
    eprintln!("Subscribed to reward() call");

    // Subscribing to new block header
    let mut new_block_subscription = init.new_block_subscription();
    eprintln!("Subscribed to new block headers");

    // Round manager contract interface
    let contract_interface = init.contract_interface();
    eprintln!("Round manager contract interface instanciated");

    // Shared value tracking if the reward call has been done
    let current_round_transaction_done = init.transaction_state();

    // Block delay after round start before triggering an alert
    let safety_window = init.safety_window();

    // Initializing transaction detector
    let transaction_detector = TransactionDetector::new();
    eprintln!("Transaction detector initialized");

    // Initializing round detector
    let mut round_detector = RoundDetector::from_contract(contract_interface, safety_window);
    eprintln!("Round detector initialized");

    eprintln!("Running...");

    // Test stdout
    println!("Testing standard output");

    // Watching Livepeer rounds
    let round_detector_stream = (&mut new_block_subscription).for_each(|block_header| {
        let block_number: U256 = block_header.number.unwrap().as_usize().into();
        if round_detector.has_new_round_started(block_number) {
            println!(
                "Started new round at {}",
                Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
            );
            // If the transaction was not done we missed the call
            if !current_round_transaction_done.load(Ordering::Acquire) {
                println!("Missed reward call :exclamation: :exclamation:");
            }
            // set the transaction as not done for this new round
            current_round_transaction_done.store(false, Ordering::Release);
        } else if round_detector.reached_security_window(block_number) {
            // Checks that the transaction has be done, otherwise triggers an alert
            if !current_round_transaction_done.load(Ordering::Acquire) {
                // Triggers an alert on standard output
                println!("Transaction has to be done :exclamation: :exclamation:");
            }
        }
        Ok(())
    });

    let reward_stream = (&mut reward_subscription).for_each(|log| {
        transaction_detector
            .exctract_transaction_from_log(init.web3(), log)
            .map(|transaction| {
                // prints the transaction on standard output
                let url = format!("https://etherscan.io/tx/{}", transaction.hash);
                let value = transaction.value.to_string();
                println!(
                    "Received new transaction:\nTime: {} UTC\nToken transfered: {}\nUrl: {}\n",
                    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    value,
                    url
                );

                // If the transaction is a success
                if transaction.value != U256::zero() {
                    eprintln!("Transaction success");
                    // sets the transaction as done
                    current_round_transaction_done.store(true, Ordering::Release);
                }
            });

        Ok(())
    });

    let main_loop = round_detector_stream.select(reward_stream);

    // We wait indefinitely
    main_loop.wait();
}
