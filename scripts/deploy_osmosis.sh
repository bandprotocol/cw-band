# To create account
## run "osmosisd keys add test --recover". Then, paste mnemonic

# Prepare 
CHAIN_ID=osmo-test-4
DEPLOYER=test

# Build contract
cargo run-script optimize

TX=$(osmosisd tx wasm store ./artifacts/poc_price_feed.wasm  --from $DEPLOYER --chain-id=$CHAIN_ID --gas-prices 0.1uosmo --gas auto --gas-adjustment 1.3 -b block --output json -y | jq -r '.txhash')
CODE_ID=$(osmosisd query tx $TX --output json | jq -r '.logs[0].events[-1].attributes[0].value')
echo "Your contract code_id is $CODE_ID"

INITIAL_STATE='{"ask_count":"1","client_id":"arm","execute_gas":"500000","fee_limit":[{"amount":"100000","denom":"uband"}],"min_count":"1","minimum_sources":1,"oracle_script_id":"360","prepare_gas":"100000"}'
osmosisd tx wasm instantiate $CODE_ID $INITIAL_STATE --amount 50000uosmo  --label "Counter Contract" --from $DEPLOYER --chain-id $CHAIN_ID --gas-prices 0.1uosmo --gas auto --gas-adjustment 1.3 -b block -y --no-admin
CONTRACT_ADDR=$(osmosisd query wasm list-contract-by-code $CODE_ID --output json | jq -r '.contracts[0]')
echo "Your contract address is $CONTRACT_ADDR"

# Query tx hash
# osmosisd query tx <TX_HASH>

# Execute message
# osmosisd tx wasm execute $CONTRACT_ADDR '{"request":{"symbols": ["BTC"] }}' --from $DEPLOYER -y --chain-id=$CHAIN_ID

# Query contract
# osmosisd query wasm contract-state smart $CONTRACT_ADDR '{"get_rate":{"symbol":"BTC"}}' --chain-id $CHAIN_ID

