use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub client_id: String,
    pub oracle_script_id: u64,
    pub ask_count: u64,
    pub min_count: u64,
    pub fee_limit: Vec<Coin>,
    pub prepare_gas: u64,
    pub execute_gas: u64,

    pub minimum_sources: u8,
}
