extern crate web3;

mod round_detector;
mod transaction_detector;

use std::sync::atomic::{AtomicBool, Ordering};

use web3::contract::Contract;
use web3::futures::{Future, Stream};
use web3::types::FilterBuilder;

use round_detector::RoundDetector;
use transaction_detector::TransactionDetector;

#[tokio::main]
async fn main() {
    // Loading config file
    let mut settings = config::Config::new();
    settings
        .merge(config::File::with_name("Settings"))
        .expect("Cannot load config from Settings.toml");

    // Creating event loop and web3 interface
    let (_eloop, transport) = web3::transports::WebSocket::new(
        &settings
            .get_str("node_endpoint")
            .expect("Cannot load node endpoint from config."),
    )
    .unwrap();

    let web3 = web3::Web3::new(transport);

    // Subscribing to reward() transactions
    let mut reward_subscription = web3
        .eth_subscribe()
        .subscribe_logs(
            FilterBuilder::default()
                .address(vec![settings
                    .get_str("recipient_address")
                    .expect("Cannot load recipient address from config")
                    .parse()
                    .expect("Cannot parse recipient address")])
                .build(),
        )
        .wait()
        .expect("Cannot subscribe to reward() calls");

    println!("Subscribed to reward() call");

    // Shared value tracking if the reward call has been done
    let current_round_transaction_done = AtomicBool::new(
        settings
            .get_bool("current_round_transaction_done")
            .unwrap_or(true),
    );

    // Subscribing to new block header
    let mut new_block_subscription = web3
        .eth_subscribe()
        .subscribe_new_heads()
        .wait()
        .expect("Cannot subscribe to new block header");

    // Round manager contract interface
    let contract_interface = Contract::from_json(
        web3.eth(),
        settings
            .get_str("proxy_address")
            .expect("Cannot load proxy address from config")
            .parse()
            .expect("Cannot parse proxy address"),
        include_bytes!("../build/RoundsManager.abi"),
    )
    .expect("Cannot instanciate round manager interface");

    println!("Round manager contract instanciated");

    // Initializing transaction detector
    let transaction_detector = TransactionDetector::new();

    // Initializing round detector
    let mut round_detector = RoundDetector::from_contract(contract_interface);

    println!("Round detector initialized, runnning...");

    // Watching Livepeer rounds
    let round_detector_stream = (&mut new_block_subscription).for_each(|block_number| {
        if round_detector.has_new_round_started(block_number) {
            // Check that the current round transaction has been done

            // set the transaction as not done for this new round
            current_round_transaction_done.store(false, Ordering::Release);
        }
        Ok(())
    });

    let reward_stream = (&mut reward_subscription).for_each(|log| {
        // if the transaction is a success
        if transaction_detector.has_valid_transaction_been_made(&web3, log) {
            println!("Transaction success");
            // sets the transaction as done
            current_round_transaction_done.store(true, Ordering::Release);
        }

        Ok(())
    });

    let main_loop = round_detector_stream.select(reward_stream);

    // We wait indefinitely
    main_loop.wait();
}
