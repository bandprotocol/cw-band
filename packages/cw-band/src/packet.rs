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
    pub resolve_status: ResolveStatus,
    pub result: Binary,
}

#[cw_serde]
pub enum ResolveStatus {
    #[serde(rename = "RESOLVE_STATUS_OPEN_UNSPECIFIED")]
    Open,
    #[serde(rename = "RESOLVE_STATUS_SUCCESS")]
    Success,
    #[serde(rename = "RESOLVE_STATUS_FAILURE")]
    Failure,
    #[serde(rename = "RESOLVE_STATUS_EXPIRED")]
    Expired,
}

#[cw_serde]
pub enum AcknowledgmentMsg {
    Result(Binary),
    Error(String),
}

// create a serialized success message
pub fn ack_success() -> Binary {
    let res = AcknowledgmentMsg::Result(b"1".into());
    to_binary(&res).unwrap()
}

// create a serialized error message
pub fn ack_fail(err: String) -> Binary {
    let res = AcknowledgmentMsg::Error(err);
    to_binary(&res).unwrap()
}

#[cw_serde]
pub struct BandAcknowledgment {
    pub request_id: Uint64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::decode;
    use cosmwasm_std::from_slice;

    #[test]
    fn test_deserialize_response_packet() {
        let packet = from_slice::<OracleResponsePacketData>(&decode("eyJhbnNfY291bnQiOiIxNiIsImNsaWVudF9pZCI6IjQxMTg2MTEiLCJyZXF1ZXN0X2lkIjoiMTM4NTk4OTkiLCJyZXF1ZXN0X3RpbWUiOiIxNjY5NzkwNDQ1IiwicmVzb2x2ZV9zdGF0dXMiOiJSRVNPTFZFX1NUQVRVU19TVUNDRVNTIiwicmVzb2x2ZV90aW1lIjoiMTY2OTc5MDQ1MSIsInJlc3VsdCI6IkFBQUFCQUFBQUFBdld4dWdBQUFBQW1jZHVIQUFBQUFBQTlzK1h3QUFBQUFQTHdtRyJ9").unwrap()).unwrap();

        assert_eq!(packet.resolve_status, ResolveStatus::Success);
        assert_eq!(packet.request_id, Uint64::from(13859899u64))
    }
}
