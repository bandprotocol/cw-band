use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, Binary, Int64, StdAck, Uint64};

#[cw_serde]
pub struct TunnelPacketData {
    pub tunnel_id: Uint64,
    pub sequence: Uint64,
    pub prices: Vec<Price>,
    pub created_at: Int64,
}

#[cw_serde]
pub struct Price {
    pub signal_id: String,
    pub status: Status,
    pub price: Uint64,
    pub timestamp: Int64,
}

// create a serialized success message
pub fn ack_success() -> Binary {
    let res = StdAck::Success(b"1".into());
    to_json_binary(&res).unwrap()
}

// create a serialized error message
pub fn ack_fail(err: String) -> Binary {
    let res = StdAck::Error(err);
    to_json_binary(&res).unwrap()
}

#[cw_serde]
pub enum Status {
    #[serde(rename = "PRICE_STATUS_UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "PRICE_STATUS_UNKNOWN_SIGNAL_ID")]
    UnknownSignalID,
    #[serde(rename = "PRICE_STATUS_NOT_READY")]
    NotReady,
    #[serde(rename = "PRICE_STATUS_AVAILABLE")]
    Available,
    #[serde(rename = "PRICE_STATUS_NOT_IN_CURRENT_FEEDS")]
    NotInCurrentFeeds,
}
