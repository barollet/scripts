use web3::contract::{self, Contract};
use web3::futures::Future;
use web3::types::{BlockHeader, U256};

// For now we are restricted to websocket, we will have to do a generic RoundDetector if we want to
// use other methods
pub type ContractItf = Contract<web3::transports::WebSocket>;

pub struct RoundDetector {
    round_end_block: U256, // end of the current round in block
    current_round: U256,

    contract_interface: ContractItf, // round manager smartcontract interface to keep track of the contract state
}

impl RoundDetector {
    pub fn from_contract(contract_interface: ContractItf) -> Self {
        let mut round_detector = RoundDetector {
            round_end_block: U256::zero(),
            current_round: U256::zero(),

            contract_interface,
        };

        round_detector.compute_round_end();
        round_detector.current_round = round_detector.query_constant_value("currentRound");

        round_detector
    }

    fn compute_round_end(&mut self) {
        let start_block: U256 = self.query_constant_value("currentRoundStartBlock");
        let round_length: U256 = self.query_constant_value("roundLength");

        self.round_end_block = start_block + round_length;
    }

    fn query_constant_value<T: contract::tokens::Tokenizable>(&self, value: &str) -> T {
        self.contract_interface
            .query(value, (), None, contract::Options::default(), None)
            .wait()
            .unwrap()
    }

    // Keeping track of block numbers and wait for new round initialization
    pub fn watch_block_number(&mut self, block: BlockHeader) {
        let block_number: U256 = block.number.unwrap().as_usize().into();
        println!("{}", block_number);

        // else wait for the end of the round
        if block_number >= self.round_end_block {
            // triggering end of round
        }
    }
}
