use cosmwasm_std::Coin;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OracleRequestPacketData {
    pub client_id: String,
    pub oracle_script_id: u64,
    pub calldata: Vec<u8>,
    pub ask_count: u64,
    pub min_count: u64,
    pub fee_limit: Vec<Coin>,
    pub prepare_gas: u64,
    pub execute_gas: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OracleResponsePacketData {
    pub client_id: String,
    pub request_id: u64,
    pub ans_count: u64,
    pub request_time: u64,
    pub resolve_time: u64,
    pub resolve_status: i32,
    pub result: Vec<u8>,
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
    pub request_id: u64,
}
