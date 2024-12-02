use cosmwasm_schema::cw_serde;
use cosmwasm_std::{IbcChannel, IbcEndpoint, Uint64};
use cw_controllers::Admin;
use cw_storage_plus::Map;

use cw_band::tunnel::packet::Price;

pub const ADMIN: Admin = Admin::new("admin");
pub const TUNNEL_CONFIG: Map<&str, TunnelConfig> = Map::new("tunnel_config");
pub const CHANNEL_INFO: Map<&str, ChannelInfo> = Map::new("channel_info");
pub const SIGNAL_PRICE: Map<&str, Price> = Map::new("signal_price");

#[cw_serde]
pub struct TunnelConfig {
    pub tunnel_id: Uint64,
    pub port_id: String,
    pub channel_id: String,
}

#[cw_serde]
pub struct ChannelInfo {
    /// id of this channel
    pub channel_id: String,
    /// the remote channel/port we connect to
    pub counterparty_endpoint: IbcEndpoint,
    /// the connection this exists on (you can use to query client/consensus info)
    pub connection_id: String,
}

impl ChannelInfo {
    pub fn new(
        channel_id: String,
        counterparty_endpoint: IbcEndpoint,
        connection_id: String,
    ) -> Self {
        ChannelInfo {
            channel_id,
            counterparty_endpoint,
            connection_id,
        }
    }
}

impl From<IbcChannel> for ChannelInfo {
    fn from(value: IbcChannel) -> Self {
        let channel_id = value.endpoint.channel_id;
        let counterparty_endpoint = value.counterparty_endpoint;
        let connection_id = value.connection_id;
        ChannelInfo {
            channel_id,
            counterparty_endpoint,
            connection_id,
        }
    }
}
