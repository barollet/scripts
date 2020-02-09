extern crate web3;

mod round_detector;

use web3::contract::Contract;
use web3::futures::{Future, Stream};
use web3::types::FilterBuilder;

use round_detector::RoundDetector;

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

    // Initializing round detector
    let mut round_detector = RoundDetector::from_contract(contract_interface);

    println!("Round detector initialized, runnning...");

    // Watching Livepeer rounds
    let round_detector_stream = (&mut new_block_subscription).for_each(|x| {
        round_detector.watch_block_number(x);
        Ok(())
    });

    let reward_stream = (&mut reward_subscription).for_each(|x| {
        println!("{:?}", x);
        Ok(())
    });

    let main_loop = round_detector_stream.select(reward_stream);

    // We wait indefinitely
    main_loop.wait();
}
