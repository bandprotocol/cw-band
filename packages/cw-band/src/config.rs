use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Uint64};

#[cw_serde]
pub struct Config {
    pub client_id: String,
    pub oracle_script_id: Uint64,
    pub ask_count: Uint64,
    pub min_count: Uint64,
    pub fee_limit: Vec<Coin>,
    pub prepare_gas: Uint64,
    pub execute_gas: Uint64,

    pub minimum_sources: u8,
}
