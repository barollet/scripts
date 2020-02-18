The `build/` directory contains the round manager contract abi and bytecode.

The `Settings.toml` file contains all the configuration for the bot, the bot has to be restarted to take the configuration into account.

The `lpt_round_manager.sol` file contains the Livepeer round manager contract code: https://github.com/livepeer/wiki/blob/master/Deployed-Contract-Addresses.md

# Installation

You need to have Rust installed to compile and run the bot.
Nothing has to be done appart from cloning the repository.

# Running

Run `cargo run --release` in this directory to launch the bot.

If you want to redirect stdout to a discord webhook use the `log_dispatcher` script.
