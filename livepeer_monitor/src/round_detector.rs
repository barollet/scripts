use web3::contract::{self, Contract};
use web3::futures::Future;
use web3::types::U256;

// For now we are restricted to websocket, we will have to do a generic RoundDetector if we want to
// use other methods
pub type ContractItf = Contract<web3::transports::WebSocket>;

pub struct RoundDetector {
    round_end_block: U256, // end of the current round in block
    safety_window_end_block: U256,
    current_round: U256,

    contract_interface: ContractItf, // round manager smartcontract interface to keep track of the contract state
    safety_window: U256,
}

impl RoundDetector {
    pub fn from_contract(contract_interface: ContractItf, safety_window: U256) -> Self {
        let mut round_detector = RoundDetector {
            round_end_block: U256::zero(),
            safety_window_end_block: U256::zero(),
            current_round: U256::zero(),

            contract_interface,
            safety_window,
        };

        round_detector.compute_round_security_window_end();
        round_detector.current_round = round_detector.fetch_current_round();

        round_detector
    }

    pub fn get_current_round(&self) -> usize {
        self.current_round.as_usize()
    }

    fn compute_round_security_window_end(&mut self) {
        let start_block: U256 = self.query_constant_value("currentRoundStartBlock");
        let round_length: U256 = self.query_constant_value("roundLength");

        self.round_end_block = start_block + round_length;
        self.safety_window_end_block = start_block + self.safety_window;
    }

    fn fetch_current_round(&self) -> U256 {
        self.query_constant_value("currentRound")
    }

    fn query_constant_value<T: contract::tokens::Tokenizable>(&self, value: &str) -> T {
        self.contract_interface
            .query(value, (), None, contract::Options::default(), None)
            .wait()
            .unwrap()
    }

    // Keeping track of block numbers and wait for new round initialization
    // Returns wether a new round start
    pub fn has_new_round_started(&mut self, block_number: U256) -> bool {
        // wait for the end of the round
        if block_number >= self.round_end_block {
            // we wait for the round number to increment, panic if the number is not incremented
            if self.fetch_current_round() == self.current_round + 1 {
                // A new round started
                self.current_round = self.current_round + 1;
                self.compute_round_security_window_end();

                println!("Started round {}", self.current_round);

                return true;
            } else if self.fetch_current_round() > self.current_round {
                panic!("Skipped a round in LPT rewards");
            }
        }
        false
    }

    // Returns if the security window is finished
    pub fn reached_security_window(&self, block_number: U256) -> bool {
        block_number == self.safety_window_end_block
    }
}
