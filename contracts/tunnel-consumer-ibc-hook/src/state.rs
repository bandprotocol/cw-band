use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint64};
use cw_band::tunnel::packet::Price;
use cw_controllers::Admin;
use cw_storage_plus::Map;

pub const ADMIN: Admin = Admin::new("admin");
pub const TUNNEL_CONFIG: Map<&str, TunnelConfig> = Map::new("tunnel_config");
pub const SIGNAL_PRICE: Map<&str, Price> = Map::new("signal_price");

#[cw_serde]
pub struct TunnelConfig {
    pub tunnel_id: Uint64,
    pub sender: Addr,
    pub port_id: String,
    pub channel_id: String,
}