# Setup relayer - Hermes

This document describes methods on how to set up [Hermes Relayer](https://github.com/informalsystems/hermes) to send IBC messages between wasm contract and BandChain.

## Step 1: Install Hermes CLI

### Install via Cargo 

```
cargo install ibc-relayer-cli --bin hermes --locked
```

This will download and build the crate ibc-relayer-cli, and install the hermes binary in $HOME/.cargo/bin.

> Note: If you have not installed Rust and Cargo via rustup.rs, you may need to add the $HOME/.cargo/bin directory to your PATH environment variable. For most shells, this can be done by adding the following line to your .bashrc or .zshrc configuration file:
> 
> export PATH="$HOME/.cargo/bin:$PATH"

You should now be able to run Hermes by invoking the hermes executable.
```
hermes version
```

## Step 2: Setup Config file

As for most parts of the config, you can see their description [here](https://hermes.informal.systems/documentation/configuration/description.html)

create and write your hermes config base on this example file [example config](https://github.com/bandprotocol/hermes/blob/2c07633f234e06bb0fd2dd88ab97952c659497cd/config_example.toml)
```
nano $HOME/.hermes/config.toml
```



## Step 3: Add keys to both chains on Hermes

use `hermes keys add` command to add keys of chains you want to relay
e.g.

> Note: Default Derivation Path is m/44'/118'/0'/0/0 but the keys that were generated from Bandd Derivation Path are m/44'/494'/0'/0/0

```bash
hermes --config <CONFIG_FILE_PATH> keys add --chain band-laozi-testnet6 --mnemonic-file "<MNEMONIC_PATH>" --hd-path "m/44'/494'/0'/0/0"
```

and

```bash
hermes --config <CONFIG_FILE_PATH> keys add --chain wasmchain --mnemonic-file "<MNEMONIC_PATH>" 
```

## Step 4: Create a channel and connection

use `hermes create channel` command to create a channel that connects between two chains and  `--new-client-connection` command to create a new client on each chain and establish a connection
e.g.

```bash
hermes --config <CONFIG_FILE_PATH> create channel --a-chain band-laozi-testnet6 --b-chain <YOUR_CHAIN_ID> --a-port oracle --b-port <YOUR_PORT> --order unordered --channel-version bandchain-1 --new-client-connection
```

> Note: `<YOUR_PORT>` usually is a module name but in the case of a wasm contract, it is `wasm.<CONTRACT_ADDRESS>`

## Step 5: Start Hermes Relayer

use `hermes start` command to start your Hermes relayer
e.g.

```bash
hermes --config <CONFIG_FILE_PATH> start
```

Then, wait until you receive `Hermes has started`. Now, your relayer is running.
