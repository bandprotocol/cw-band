# Tunnel Consumer IBC Hook Contract

## Overview

This contract is a demo implementation on how to consumer data from BandChain's tunnel through IBC Hook with CosmWasm.
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
