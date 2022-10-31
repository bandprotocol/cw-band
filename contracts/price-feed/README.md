# Price feed contract

## Overview

[//]: # (TODO)

## Build

### Contract

To compile all contracts, run the following script in the repo root: `/scripts/build_artifacts.sh` or the command below:
The optimized wasm code and its checksums can be found in the `/artifacts` directory

```
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.12.7
```

### Schema

To generate the JSON schema files for the contract call, queries and query responses, run the following script in the
repo root: `/scripts/build_schemas.sh` or run `cargo schema` in the smart contract directory.

## Messages
### Instantiate message
This contract accepts the following during instantiation

```rust
#[cw_serde]
pub struct InstantiateMsg {
    // A unique ID for the oracle request
    pub client_id: String,
    // The oracle script ID to query
    pub oracle_script_id: Uint64,
    // The number of validators that are requested to respond
    pub ask_count: Uint64,
    // The minimum number of validators that need to respond
    pub min_count: Uint64,
    // The maximum amount of band in uband to be paid to the data source providers
    // e.g. vec![Coin::new(100, "uband")]
    pub fee_limit: Vec<Coin>,
    // Amount of gas to pay to prepare raw requests
    pub prepare_gas: Uint64,
    // Amount of gas reserved for execution
    pub execute_gas: Uint64,
    // Minimum number of sources required to return a successful response
    pub minimum_sources: u8,
}
```

### Execute message

This is a simple version allow anyone can send transaction to update price immediately

- Request

```json
{
  "request": {
    "symbols": ["BTC", "ETH", "BAND"]
  }
}
```

### Query message

TODO
