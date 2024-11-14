use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint64;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct UpdateTunnelConfigMsg {
    pub tunnel_id: Uint64,
    pub port_id: String,
    pub channel_id: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateAdmin { admin: String },
    UpdateTunnelConfig(UpdateTunnelConfigMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Option<cosmwasm_std::Addr>)]
    Admin {},
    #[returns(cosmwasm_std::IbcEndpoint)]
    TunnelConfig { tunnel_id: Uint64 },
}
