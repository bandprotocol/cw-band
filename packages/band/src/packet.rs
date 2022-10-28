use cosmwasm_std::{Binary, Coin, Uint64};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OracleRequestPacketData {
    pub client_id: String,
    pub oracle_script_id: Uint64,
    pub calldata: Vec<u8>,
    pub ask_count: Uint64,
    pub min_count: Uint64,
    pub fee_limit: Vec<Coin>,
    pub prepare_gas: Uint64,
    pub execute_gas: Uint64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OracleResponsePacketData {
    pub client_id: String,
    pub request_id: Uint64,
    pub ans_count: Uint64,
    pub request_time: Uint64,
    pub resolve_time: Uint64,
    pub resolve_status: String,
    pub result: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AcknowledgementMsg<S> {
    Result(S),
    /// An error type that every custom error created by contract developers can be converted to.
    /// This could potientially have more structure, but String is the easiest.
    #[serde(rename = "error")]
    Err(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BandAcknowledgement {
    pub request_id: Uint64,
}
