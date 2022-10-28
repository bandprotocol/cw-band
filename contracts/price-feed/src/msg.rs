use crate::state::{Rate, ReferenceData};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

#[cw_serde]
pub struct InstantiateMsg {
    pub client_id: String,
    pub oracle_script_id: u64,
    pub ask_count: u64,
    pub min_count: u64,
    pub fee_limit: Vec<Coin>,
    pub prepare_gas: u64,
    pub execute_gas: u64,

    pub minimum_sources: u8,
}

// TODO: Add more message who can request new price (whitelist requester)
#[cw_serde]
pub enum ExecuteMsg {
    Request { symbols: Vec<String> },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Rate)]
    // Returns the RefData of a given symbol
    GetRate {
        // Symbol to query
        symbol: String,
    },
    #[returns(ReferenceData)]
    // Returns the ReferenceData of a given asset pairing
    GetReferenceData {
        // Symbol pair to query where:
        // symbol_pair := (base_symbol, quote_symbol)
        // e.g. BTC/USD ≡ ("BTC", "USD")
        symbol_pair: (String, String),
    },
    #[returns(Vec<ReferenceData>)]
    // Returns the ReferenceDatas of the given asset pairings
    GetReferenceDataBulk {
        // Vector of Symbol pair to query
        // e.g. <BTC/USD ETH/USD, BAND/BTC> ≡ <("BTC", "USD"), ("ETH", "USD"), ("BAND", "BTC")>
        symbol_pairs: Vec<(String, String)>,
    },
}
