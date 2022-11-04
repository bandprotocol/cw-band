# Connect price feed to BandChain testnet

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

## Endpoint to connect with BandChain

https://docs.bandchain.org/technical-specifications/band-endpoints.html
