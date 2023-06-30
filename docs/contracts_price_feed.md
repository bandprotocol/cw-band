# Contract example - Price Feed

## Supported symbols

You can request 4 symbols on [oracle script id: 360](https://laozi-testnet6.cosmoscan.io/oracle-script/360)

- BTC, ETH, USDT, BAND

## Config example

```
client_id: String -> Arbitary string for your request
oracle_script_id: Uint64, -> Please use oracle script id 360
ask_count: Uint64, -> The number of validator you want to ask (Recommend: 4 on testnet)
min_count: Uint64, -> The minimum number of validator need to answer to aggregate result (Recommend: 3 on testnet)
fee_limit: Vec<Coin>, -> Data source fee that you willing to pay (Recommend: 250000uband)
prepare_gas: Uint64, -> Gas for running prepare phrase (Recommend: 100000)
execute_gas: Uint64, -> Gas for running execute phrase (Recommend: 500000)
minimum_sources: u8, -> The minimum available sources to determine price is aggregated from at least minimum sources (for data integrity) 1 should be ok for testing
```


## Deploy, Execute, and Query Contract
**Note**: In this case use [wasmd](https://github.com/CosmWasm/wasmd) if you use other wasm chains please change your cli.

### Build contract
You can do further optimization using rust-optimizer. rust-optimizer produces reproducible builds of CosmWasm smart contracts and does heavy optimization on the build size, using binary stripping and wasm-opt.
```
scripts/build_artifacts.sh
```
Binary file will be at artifacts/price_feed.wasm folder, which is more smaller than when only RUTFLAGS was used.

### Set environment variables
```
CHAIN_ID=YOUR_CHAIN_ID
DEPLOYER=YOUR_WALLET_NAME
```

### Deploy contract

```
TX=$(wasmd tx wasm store ./artifacts/price_feed.wasm  --from $DEPLOYER --chain-id=$CHAIN_ID --gas-prices 0.1stake --gas auto --gas-adjustment 1.3 -b block --output json -y | jq -r '.txhash')
CODE_ID=$(wasmd query tx $TX --output json | jq -r '.logs[0].events[-1].attributes[1].value')
echo "Your contract code_id is $CODE_ID"

INITIAL_STATE='{"ask_count":"1","client_id":"cw-band-price-feed","execute_gas":"500000","fee_limit":[{"amount":"100000","denom":"uband"}],"min_count":"10","minimum_sources":16,"oracle_script_id":"360","prepare_gas":"100000"}'
wasmd tx wasm instantiate $CODE_ID $INITIAL_STATE --amount 50000stake  --label "Counter Contract" --from $DEPLOYER --chain-id $CHAIN_ID --gas-prices 0.1stake --gas auto --gas-adjustment 1.3 -b block -y --no-admin
CONTRACT_ADDR=$(wasmd query wasm list-contract-by-code $CODE_ID --output json | jq -r '.contracts[0]')
echo "Your contract address is $CONTRACT_ADDR"
```

### Request data from BandChain example
```
# Execute message
wasmd tx wasm execute $CONTRACT_ADDR '{"request":{"symbols": ["BTC"] }}' --from $DEPLOYER -y --chain-id=$CHAIN_ID -b bloc
```

### Query contract example
```
# Query contract
wasmd query wasm contract-state smart $CONTRACT_ADDR '{"get_rate":{"symbol":"BTC"}}' --chain-id $CHAIN_ID
```

## Endpoint to connect with BandChain

https://docs.bandchain.org/technical-specifications/band-endpoints.html
