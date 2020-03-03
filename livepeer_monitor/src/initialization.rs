use web3::api::SubscriptionStream;
use web3::contract::Contract;
use web3::futures::Future;
use web3::transports::EventLoopHandle;
use web3::types::{Address, BlockHeader, U256};

use config::Config;

type WS = web3::transports::WebSocket;
pub type Web3Itf = web3::Web3<WS>;

pub struct Init {
    // TODO with event loop
    _eloop: EventLoopHandle, // event loop handle to keep transport in scope
    settings: Config,
    web3: Web3Itf,
}

impl Init {
    pub fn load_config() -> Self {
        let mut settings = config::Config::new();
        settings
            .merge(config::File::with_name("LivepeerMonitorSettings"))
            .expect("Cannot load config from Settings.toml");

        let websocket_endpoint = settings
            .get_str("node_endpoint")
            .expect("Cannot load node endpoint from config.");
        // Creating event loop and websocket connection
        let (_eloop, transport) = web3::transports::WebSocket::new(&websocket_endpoint).unwrap();

        // Instanciating the web3 interface
        let web3 = web3::Web3::new(transport);

        Self {
            _eloop,
            settings,
            web3,
        }
    }

    pub fn web3(&self) -> &Web3Itf {
        &self.web3
    }

    pub fn load_transcoder_address(&self) -> Address {
        let transcoder_address: String = self
            .settings
            .get_str("recipient_address")
            .expect("Cannot load recipient address from config");
        transcoder_address
            .parse()
            .expect("Cannot parse transcoder address")
    }

    pub fn new_block_subscription(&self) -> SubscriptionStream<WS, BlockHeader> {
        self.web3
            .eth_subscribe()
            .subscribe_new_heads()
            .wait()
            .expect("Cannot subscribe to new block header")
    }

    pub fn round_manager_contract_interface(&self) -> Contract<WS> {
        let proxy_address: String = self
            .settings
            .get_str("round_manager_proxy_address")
            .expect("Cannot load round manager proxy address from config");
        let proxy_address: Address = proxy_address.parse().expect("Cannot parse proxy address");

        Contract::from_json(
            self.web3.eth(),
            proxy_address,
            include_bytes!("../build/RoundsManager.abi"),
        )
        .expect("Cannot instanciate round manager interface")
    }

    pub fn bonding_manager_contract_interface(&self) -> Contract<WS> {
        let proxy_address = self
            .settings
            .get_str("bonding_manager_proxy_address")
            .expect("Cannot load bonding manager proxy address from config");
        let proxy_address = proxy_address.parse().expect("Cannot parse proxy address");

        Contract::from_json(
            self.web3.eth(),
            proxy_address,
            include_bytes!("../build/BondingManager.abi"),
        )
        .expect("Cannot instanciate round manager interface")
    }

    pub fn transaction_state(&self) -> bool {
        self.settings
            .get_bool("current_round_transaction_done")
            .expect("Cannot load transaction initial state from config.")
    }

    pub fn safety_window(&self) -> U256 {
        self.settings
            .get_int("block_alert_delay")
            .unwrap_or(50)
            .into()
    }
}
