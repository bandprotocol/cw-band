# To create account
## run "wasmd keys add test --recover". Then, paste mnemonic

# Prepare 
CHAIN_ID=wasmchain
DEPLOYER=test

# Build contract
./scripts/build_artifacts.sh

TX=$(wasmd tx wasm store ./artifacts/price_feed.wasm  --from $DEPLOYER --chain-id=$CHAIN_ID --gas-prices 0.1stake --gas auto --gas-adjustment 1.3 -b block --output json -y | jq -r '.txhash')
CODE_ID=$(wasmd query tx $TX --output json | jq -r '.logs[0].events[-1].attributes[1].value')
echo "Your contract code_id is $CODE_ID"

INITIAL_STATE='{"ask_count":"1","client_id":"arm","execute_gas":"500000","fee_limit":[{"amount":"100000","denom":"uband"}],"min_count":"1","minimum_sources":1,"oracle_script_id":"360","prepare_gas":"100000"}'
wasmd tx wasm instantiate $CODE_ID $INITIAL_STATE --amount 50000stake  --label "Counter Contract" --from $DEPLOYER --chain-id $CHAIN_ID --gas-prices 0.1stake --gas auto --gas-adjustment 1.3 -b block -y --no-admin
CONTRACT_ADDR=$(wasmd query wasm list-contract-by-code $CODE_ID --output json | jq -r '.contracts[0]')
echo "Your contract address is $CONTRACT_ADDR"

# Query tx hash
# wasmd query tx <TX_HASH>

# Execute message
# wasmd tx wasm execute $CONTRACT_ADDR '{"request":{"symbols": ["BTC"] }}' --from $DEPLOYER -y --chain-id=$CHAIN_ID -b block

# Query contract
# wasmd query wasm contract-state smart $CONTRACT_ADDR '{"get_rate":{"symbol":"BTC"}}' --chain-id $CHAIN_ID
