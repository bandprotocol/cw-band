use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Int64, Uint64};
use cw_band::tunnel::packet::{Price, TunnelPacket};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    ReceivePacket {
        packet: TunnelPacket
    },
    SetTunnelConfig {
        msg: SetTunnelConfigMsg
    },

}

#[cw_serde]
pub struct SetTunnelConfigMsg {
    pub tunnel_id: Uint64,
    pub sender: Addr,
    pub port_id: String,
    pub channel_id: String,
}


#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    Prices {
        signal_id: String,
    },
}
