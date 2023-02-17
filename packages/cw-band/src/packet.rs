use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Binary, Coin, Uint64};

#[cw_serde]
pub struct OracleRequestPacketData {
    pub client_id: String,
    pub oracle_script_id: Uint64,
    pub calldata: Binary,
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

#[cfg(test)]
mod tests {
    use super::*;
    use base64::decode;
    use cosmwasm_std::from_slice;

    #[test]
    fn test_serialize_request_packet() {
        let calldata = base64::decode("AAAAAgAAAANCVEMAAAADRVRIAQ==").unwrap();
        let packet = OracleRequestPacketData {
            client_id: "1".into(),
            oracle_script_id: Uint64::from(360u64),
            calldata: Binary(calldata),
            ask_count: Uint64::from(1u64),
            min_count: Uint64::from(1u64),
            fee_limit: vec![Coin::new(1000, "uband")],
            prepare_gas: Uint64::from(900000u64),
            execute_gas: Uint64::from(4000000u64),
        };

        assert_eq!(hex::encode(to_binary(&packet).unwrap().to_vec()), "7b22636c69656e745f6964223a2231222c226f7261636c655f7363726970745f6964223a22333630222c2263616c6c64617461223a22414141414167414141414e4356454d41414141445256524941513d3d222c2261736b5f636f756e74223a2231222c226d696e5f636f756e74223a2231222c226665655f6c696d6974223a5b7b2264656e6f6d223a227562616e64222c22616d6f756e74223a2231303030227d5d2c22707265706172655f676173223a22393030303030222c22657865637574655f676173223a2234303030303030227d");
    }

    #[test]
    fn test_deserialize_response_packet() {
        let packet = from_slice::<OracleResponsePacketData>(&decode("eyJhbnNfY291bnQiOiIxNiIsImNsaWVudF9pZCI6IjQxMTg2MTEiLCJyZXF1ZXN0X2lkIjoiMTM4NTk4OTkiLCJyZXF1ZXN0X3RpbWUiOiIxNjY5NzkwNDQ1IiwicmVzb2x2ZV9zdGF0dXMiOiJSRVNPTFZFX1NUQVRVU19TVUNDRVNTIiwicmVzb2x2ZV90aW1lIjoiMTY2OTc5MDQ1MSIsInJlc3VsdCI6IkFBQUFCQUFBQUFBdld4dWdBQUFBQW1jZHVIQUFBQUFBQTlzK1h3QUFBQUFQTHdtRyJ9").unwrap()).unwrap();

        assert_eq!(packet.resolve_status, ResolveStatus::Success);
        assert_eq!(packet.request_id, Uint64::from(13859899u64))
    }
}
