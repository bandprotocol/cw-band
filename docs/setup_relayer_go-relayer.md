# Setup relayer - Go-Relayer

This document describes methods on how to set up [go-relayer](https://github.com/cosmos/relayer) to send IBC messages between wasm contract and BandChain.

## Step 1: Build go-relayer binary

```bash
# Clone and install relayer binary
git clone https://github.com/cosmos/relayer.git
cd relayer
git checkout v2.2.0-rc1
make install

# Verify binary
rly version # version: 2.2.0-rc1
```

## Step 2: Setup chains

Initialize config for a relayer

```bash
rly config init
```

Next, you have to add chains that you want to relay so that relayer can know the detail of the chains. There are two ways of doing this.
- Fetching meta-data of chains from [chain-registry](https://github.com/cosmos/chain-registry).
    - `rly chains add <CHAIN1> <CHAIN2> <...>`
- Manually add chain from a config file.
    - `rly chains add --file <FILE_PATH> <CHAIN_NAME>`

For band-laozi-testnet6, you can use this config file.
```json
{
    "type": "cosmos",
    "value": {
        "key": "default",
        "chain-id": "band-laozi-testnet6",
        "rpc-addr": "https://rpc.laozi-testnet6.bandchain.org:443",
        "account-prefix": "band",
        "keyring-backend": "test",
        "gas-adjustment": 1.2,
        "gas-prices": "0.01uband",
        "min-gas-amount": 1,
        "debug": true,
        "timeout": "30s",
        "output-format": "json",
        "sign-mode": "direct"
    }
}
```

## Step 3: Setup keys

use `rly keys restore <CHAIN_NAME> <KEY_NAME> <MNEMONIC> --coin-type <COIN_TYPE>` command to add the key of chains that you want to relay. e.g.

```bash
rly keys restore band-laozi-testnet6 default "<MNEMONIC>" --coin-type 494
rly q balance band-laozi-testnet6
```

**Note**: Default coin type is 118 but the coin type of keys that were generated from `Bandd` binary is 494.

## Step 4: Setup path

In this step, you will have to create a path for the relayer by this command. 
`rly paths new <CHAIN_ID_1> <CHAIN_ID_2> <PATH_NAME>` e.g.

```
rly paths new band-laozi-testnet6 wasmchain bandchain-wasmchain
```

## Step 5: Create a client, connection, and channel between the chains

`Note`:  If you already have the channel between the chains, you can skip this step. 

You can create a client, connection, or channel with `rly` transact link <PATH_NAME> --src-port <SRC_PORT> --dst-port <DST_PORT>` command. e.g.

```bash
rly transact link bandchain-wasmchain --version bandchain-1 --src-port oracle --dst-port wasm.<YOUR_CONTRACT_ADDRESS>
```

## Step 6: Start Go Relayer

use `rly start` command to start your Go relayer.
```
rly start
```

And then..finished! Your relayer is running now.


