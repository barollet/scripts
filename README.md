# Scripts
Scripts for the validators

auto-delegate.sh: https://gist.github.com/bneiluj/38c1d2f0c05e3e9d33889f20e5d3637a

# Livepeer monitoring
The folder `livepeer monitor` contains a script that tracks LP rounds start and reward transaction, this triggers an alert (on stdout) if the transaction is not done within a given block frame.

For simple run, you can just put the config files and the compiled binaries in the same folder. See README.md.

# log dispatcher
The folder `log_dispatcher` contains a script that posts its standard input to a given webhook
