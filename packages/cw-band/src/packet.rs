use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Binary, Coin, Uint64};

#[cw_serde]
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

#[cw_serde]
pub struct OracleResponsePacketData {
    pub client_id: String,
    pub request_id: Uint64,
    pub ans_count: Uint64,
    pub request_time: Uint64,
    pub resolve_time: Uint64,
    pub resolve_status: String,
    pub result: Binary,
}

#[cw_serde]
pub enum AcknowledgementMsg {
    Result(Binary),
    Error(String),
}

// create a serialized success message
pub fn ack_success() -> Binary {
    let res = AcknowledgementMsg::Result(b"1".into());
    to_binary(&res).unwrap()
}

// create a serialized error message
pub fn ack_fail(err: String) -> Binary {
    let res = AcknowledgementMsg::Error(err);
    to_binary(&res).unwrap()
}

#[cw_serde]
pub struct BandAcknowledgement {
    pub request_id: Uint64,
}
