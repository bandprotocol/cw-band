use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Int64, StdAck, to_json_binary, Uint64};

#[cw_serde]
pub struct TunnelPacket {
    pub tunnel_id: Uint64,
    pub sequence: Uint64,
    pub prices: Vec<Price>,
    pub created_at: Int64,
}

impl TunnelPacket {
    pub fn new<T, U, V>(tunnel_id: T, sequence: U, prices: Vec<Price>, created_at: V) -> Self
    where
        T: Into<Uint64>,
        U: Into<Uint64>,
        V: Into<Int64>,
    {
        TunnelPacket {
            tunnel_id: tunnel_id.into(),
            sequence: sequence.into(),
            prices,
            created_at: created_at.into(),
        }
    }
}

#[cw_serde]
pub struct Price {
    pub signal_id: String,
    pub status: Status,
    pub price: Uint64,
    pub timestamp: Int64,
}

impl Price {
    pub fn new<T, U, V>(signal_id: T, status: Status, price: U, timestamp: V) -> Self
    where
        T: ToString,
        U: Into<Uint64>,
        V: Into<Int64>,
    {
        Price {
            signal_id: signal_id.to_string(),
            status,
            price: price.into(),
            timestamp: timestamp.into(),
        }
    }
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

#[cfg(test)]
mod tests {
    use cosmwasm_std::from_json;

    use super::*;

    #[test]
    fn test_serialize_tunnel_packet() {
        let packet = TunnelPacket::new(
            1u64,
            360u64,
            vec![Price::new(
                "CS:TEST-USD",
                Status::Available,
                4000000u64,
                9000000i64,
            )],
            1600000i64,
        );
        let serialized = to_json_binary(&packet).unwrap();

        let expected = "eyJ0dW5uZWxfaWQiOiIxIiwic2VxdWVuY2UiOiIzNjAiLCJwcmljZXMiOlt7InNpZ25hbF9pZCI6IkNTOlRFU1QtVVNEIiwic3RhdHVzIjoiUFJJQ0VfU1RBVFVTX0FWQUlMQUJMRSIsInByaWNlIjoiNDAwMDAwMCIsInRpbWVzdGFtcCI6IjkwMDAwMDAifV0sImNyZWF0ZWRfYXQiOiIxNjAwMDAwIn0=";
        assert_eq!(serialized.to_base64(), expected);
    }

    #[test]
    fn test_deserialize_tunnel_packet() {
        let binary = Binary::from_base64("eyJ0dW5uZWxfaWQiOiIxIiwic2VxdWVuY2UiOiIzNjAiLCJwcmljZXMiOlt7InNpZ25hbF9pZCI6IkNTOlRFU1QtVVNEIiwic3RhdHVzIjoiUFJJQ0VfU1RBVFVTX0FWQUlMQUJMRSIsInByaWNlIjoiNDAwMDAwMCIsInRpbWVzdGFtcCI6IjkwMDAwMDAifV0sImNyZWF0ZWRfYXQiOiIxNjAwMDAwIn0").unwrap();
        let decoded = from_json::<TunnelPacket>(binary).unwrap();

        let expected = TunnelPacket::new(
            1u64,
            360u64,
            vec![Price::new(
                "CS:TEST-USD",
                Status::Available,
                4000000u64,
                9000000i64,
            )],
            1600000i64,
        );
        assert_eq!(decoded, expected)
    }
}
