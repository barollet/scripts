mod initialization;
mod round_detector;
mod transaction_detector;

use std::sync::atomic::Ordering;

use web3::futures::{Future, Stream};
use web3::types::U256;

use web3::contract;

use initialization::*;
use round_detector::RoundDetector;
use transaction_detector::TransactionDetector;

// Converts a given amount to a token amount without decimals (as a owned string)
// panic! if decimal is smaller than the string lenght (amount below 1)
// TODO return 0 if decimal > length
fn convert_amount(amount: U256, decimal: usize) -> String {
    // converts the amount to string
    let mut string_amount = amount.to_string();
    let lenght = string_amount.len();
    let amount_end = lenght - decimal;

    // We drain the first characters up to the last 18 ones
    string_amount.drain(0..amount_end).collect()
}

#[tokio::main]
async fn main() {
    // Loading config file and instanciating websocket transport
    let init = Init::load_config();

    let transcoder_address = init.load_transcoder_address();

    // Subscribing to reward() transactions
    let mut reward_subscription = init.reward_call_subscription();
    eprintln!(
        "Subscribed to reward() call to address {}",
        init.load_transcoder_address()
    );

    // Subscribing to new block header
    let mut new_block_subscription = init.new_block_subscription();
    eprintln!("Subscribed to new block headers");

    // Round manager contract interface
    let round_manager_contract_interface = init.round_manager_contract_interface();
    eprintln!(
        "Round manager contract interface instanciated at address {}",
        round_manager_contract_interface.address()
    );

    // Bonding manager contract interface
    let bonding_manager_contract_interface = init.bonding_manager_contract_interface();
    eprintln!(
        "Bonding manager contract interface instanciated at address {}",
        bonding_manager_contract_interface.address()
    );

    // Shared value tracking if the reward call has been done
    let current_round_transaction_done = init.transaction_state();
    let current_round = init.current_round();

    // Block delay after round start before triggering an alert
    let safety_window = init.safety_window();

    // Initializing transaction detector
    let transaction_detector = TransactionDetector::new();
    eprintln!("Transaction detector initialized");

    // Initializing round detector
    let mut round_detector =
        RoundDetector::from_contract(round_manager_contract_interface, safety_window);
    eprintln!("Round detector initialized");
    eprintln!("Running...");

    // Test stdout
    println!("Testing standard output");

    // Watching Livepeer rounds
    let round_detector_stream = (&mut new_block_subscription).for_each(|block_header| {
        let block_number: U256 = block_header.number.unwrap().as_usize().into();
        if round_detector.has_new_round_started(block_number) {
            current_round.store(round_detector.get_current_round(), Ordering::Release);
            // If the transaction was not done we missed the call
            if !current_round_transaction_done.load(Ordering::Acquire) {
                // If we missed the reward call of the last round
                println!(
                    "Missed reward call for round {}!!",
                    round_detector.get_current_round() - 1
                );
            }
            // set the transaction as not done for this new round
            current_round_transaction_done.store(false, Ordering::Release);
        } else if round_detector.reached_security_window(block_number) {
            // Checks that the transaction has be done, otherwise triggers an alert
            if !current_round_transaction_done.load(Ordering::Acquire) {
                // Triggers an alert on standard output
                println!(
                    "Transaction has to be done for round {}!",
                    round_detector.get_current_round()
                );
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
                let current_round = current_round.load(Ordering::Acquire);

                let value = convert_amount(transaction.value, 18);
                let total_stake: U256 = bonding_manager_contract_interface
            .query("transcoderTotalStake", transcoder_address, None, contract::Options::default(), None)
            .wait()
            .unwrap();
                let total_stake = convert_amount(total_stake, 18);
                println!("Rewards claimed for round {} -> Transcoder {} received {} LPT for a total stake of {} LPT.", current_round, transcoder_address, value, total_stake);
                println!("{}", url);

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
