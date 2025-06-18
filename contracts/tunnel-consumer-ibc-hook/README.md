# Tunnel Consumer IBC Hook Contract

## Overview

This contract is a demo implementation on how to consume data from BandChain's tunnel through IBC Hook with CosmWasm.
The contract shown here is still under development and is **NOT** intended for production use.

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

## Usage

### Querying Prices

To query prices, the contract can be queried with QueryMsg::Prices where QueryMsg::Prices is defined as:

```rust
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<Option<cw_band::tunnel::packet::Price>>)]
    Prices { signal_ids: Vec<String> },
}
```

where signal_ids is a list of signal ids to query.
The contract will return a list of prices for the given signal ids.

An example QueryMsg for query signal CS:BTC-USD is:

```json
{
    "prices": {
        "signal_ids": [
            "CS:BTC-USD"
        ]
    }
}
```

