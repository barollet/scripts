mod initialization;
mod round_detector;
mod transaction_detector;

use web3::futures::{Future, Stream};
use web3::types::U256;

use web3::contract;

use initialization::*;
use round_detector::RoundDetector;
use transaction_detector::TransactionDetector;

// Converts a given amount to a token amount without decimals (as a owned string)
// panic! if decimal is smaller than the string lenght (amount below 1)
fn convert_amount(amount: U256, decimal: usize) -> String {
    // converts the amount to string
    let mut string_amount = amount.to_string();
    let lenght = string_amount.len();
    // index of the end of the integer amount, 0 if the amount is only decimal
    let amount_end = lenght.checked_sub(decimal).unwrap_or(0);

    // integer part
    let mut output = if amount_end > 0 {
        string_amount.drain(0..amount_end).collect()
    } else {
        "0".to_owned()
    };
    // point
    output.push('.');
    // decimal part
    output.push_str(&string_amount);

    output
}

#[tokio::main]
async fn main() {
    // Loading config file and instanciating websocket transport
    let init = Init::load_config();

    let transcoder_address = init.load_transcoder_address();

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

    // Block delay after round start before triggering an alert
    let safety_window = init.safety_window();

    // Initializing transaction detector
    let transaction_detector = TransactionDetector::new(transcoder_address);
    eprintln!(
        "Transaction detector initialized on address {}",
        transcoder_address
    );

    let mut current_round_transaction_done = init.transaction_state();

    // Initializing round detector
    let mut round_detector =
        RoundDetector::from_contract(round_manager_contract_interface, safety_window);
    eprintln!("Round detector initialized");
    eprintln!("Running...");

    // Test stdout
    println!("Testing standard output");

    // Watching Livepeer rounds
    (&mut new_block_subscription).for_each(|block_header| {
        // Round detector
        let block_number: U256 = block_header.number.unwrap().as_usize().into();
        if round_detector.has_new_round_started(block_number) {
            // If the transaction was not done we missed the call
            if !current_round_transaction_done {
                // If we missed the reward call of the last round
                println!(
                    "Missed reward call for round {}!!",
                    round_detector.get_current_round() - 1
                );
            }
            // set the transaction as not done for this new round
            current_round_transaction_done = false;
        } else if round_detector.reached_security_window(block_number) {
            // Checks that the transaction has be done, otherwise triggers an alert
            if !current_round_transaction_done {
                // Triggers an alert on standard output
                println!(
                    "Transaction has to be done for round {}!",
                    round_detector.get_current_round()
                );
            }
        }

        // Transaction detector
        if let Some(transaction) = transaction_detector.contains_reward_transaction(init.web3(), block_header) {
            // prints the transaction on standard output
            let url = format!("https://etherscan.io/tx/{:#x}", transaction.hash);
            let current_round = round_detector.get_current_round();

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
                current_round_transaction_done = true;
            }
        }

        Ok(())
    }).wait().unwrap();
}
