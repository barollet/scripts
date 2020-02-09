use web3::contract::Contract;
use web3::types::BlockHeader;

/*
*

   let a: U256 = contract_interface
       .query("currentRound", (), None, contract::Options::default(), None)
       .wait()
       .unwrap();
   println!("{:?}", a);
*/

// For now we are restricted to websocket, we will have to do a generic RoundDetector if we want to
// use other methods
pub type ContractItf = Contract<web3::transports::WebSocket>;

pub struct RoundDetector {
    round_end_block: usize, // end of the current round in block, 0 means uninitialized
    current_round: usize,

    transaction_has_to_be_done: bool,

    contract_interface: ContractItf, // round manager smartcontract interface to keep track of the contract state
}

impl RoundDetector {
    pub fn from_contract(contract_interface: ContractItf) -> Self {
        RoundDetector {
            round_end_block: 0,
            current_round: 0,
            transaction_has_to_be_done: false,
            contract_interface,
        }
    }

    // Keeping track of block numbers and wait for new round initialization
    pub fn watch_block_number(&mut self, block: BlockHeader) {
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
