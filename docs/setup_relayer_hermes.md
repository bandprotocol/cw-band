# Setup relayer - Hermes

This document describes methods on how to set up [Hermes Relayer](https://github.com/informalsystems/hermes) to send IBC messages between wasm contract and BandChain.

**Note**: At the time of writing. Hermes can't automatically send an IBC message from CosmWasm contract with `hermes start` command (related issues [#2809](https://github.com/informalsystems/hermes/issues/2809), [#2815](https://github.com/informalsystems/hermes/pull/2815)). Therefore, we have built the temporary version of Hermes [here](https://github.com/bandprotocol/hermes).


## Step 1: Setup Config file
You can start with this [example config file](https://github.com/bandprotocol/hermes/blob/2c07633f234e06bb0fd2dd88ab97952c659497cd/config_example.toml) as a starting point

As for most parts of the config, you can see their description [here](https://hermes.informal.systems/documentation/configuration/description.html)](https://hermes.informal.systems/documentation/configuration/description.html) 

For the 'ignore_port_channel' config which is the one that we implemented, it is used to specify which destination port-channel pairs that we want to ignore their acknowledge packets 
e.g. `ignore_port_channel = [{ channel_id = 'channel-64', port_id = 'oracle'}]` means we want to ignore acknowledgment packet that its destination has channel_id 'channel-64' and port_id 'oracle'

## Step 2: Build Hermes binary

### Option 2.1: Build your own Hermes binary

```bash
# Clone Hermes version 1.1.0-band
git clone https://github.com/bandprotocol/hermes.git
cd hermes
git checkout v1.1.0-band

# Build Hermes
cd relayer-cli
cargo build --release
```

You should find Hermes binary at hermes/target/debug/

### Option 2.2: Download from Github (Not Recommended)

You can download Hermes binary directly from [here](https://github.com/bandprotocol/hermes/releases/tag/v1.1.0-band)

## Step 3: Add keys to both chains on Hermes

use `hermes keys add` command to add keys of chains you want to relay
e.g.
```
hermes [--config <CONFIG_FILE_PATH>] keys add --chain laozi-mainnet --mnemonic-file "<MNEMONIC_PATH>" --hd-path "m/44'/494'/0'/0/0"
```

**Note** 
Default Derivation Path is m/44'/118'/0'/0/0 but the keys that were generated from Bandd Derivation Path are m/44'/494'/0'/0/0

## Step 4: Create a channel

use `hermes create channel` command to create a channel that connects between two chains
e.g.
```
hermes [--config $CONFIG_DIR] create channel --a-chain laozi-mainnet --b-chain <YOUR_CHAIN_ID> --a-port oracle --b-port <YOUR_PORT> --order unordered --channel-version bandchain-1 --new-client-connection
```

**Note**
`<YOUR_PORT>` usually is a module name but in the case of a wasm contract, it is 'wasm.<CONTRACT_ADDRESS>'

## Step 5: Start Hermes Relayer

use `hermes start` command to start your Hermes relayer
e.g.
```
hermes [--config $CONFIG_DIR] start
```
And then..finished! Your relayer is running now.


