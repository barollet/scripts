extern crate web3;

use web3::contract::Contract;
use web3::futures::{Future, Stream};
use web3::types::{BlockHeader, FilterBuilder};

#[derive(Default)]
struct RoundDetector {
    round_end_block: usize, // end of the current round in block, 0 means uninitialized
    current_round: usize,
    current_round_initialized: bool,
    current_round_locked: bool,

    transaction_has_to_be_done: bool,
}

impl RoundDetector {
    // Keeping track of block numbers and wait for new round initialization
    fn watch_block_number(&mut self, block: BlockHeader) {
        let block_number = block.number.unwrap().as_usize();
        println!("{}", block_number);

        // if the round detector is uninitialized, initialize it.
        if self.round_end_block == 0 {
            //
        }
        // else wait for the end of the round
        else if block_number >= self.round_end_block {
            // triggering end of round
        }
    }
}

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

    // Subscribing to new block header
    let mut new_block_subscription = web3
        .eth_subscribe()
        .subscribe_new_heads()
        .wait()
        .expect("Cannot subscribe to new block header");

    // Deploy round manager contract
    let contract_interface = Contract::from_json(
        web3.eth(),
        settings
            .get_str("proxy_address")
            .expect("Cannot load proxy address from config")
            .parse()
            .expect("Cannot parse proxy address"),
        include_bytes!("../build/RoundsManager.abi"),
    )
    .expect("Cannot deploy round manager interface");

    println!("Round manager contract deployed");

    // Initializing round detector
    let mut round_detector = RoundDetector::default();

    // TODO clean code
    // Watching Livepeer rounds
    let a = (&mut new_block_subscription).for_each(|x| {
        round_detector.watch_block_number(x);
        Ok(())
    });

    let b = (&mut reward_subscription).for_each(|x| {
        println!("{:?}", x);
        Ok(())
    });

    let c = a.select(b);

    c.wait();
}
