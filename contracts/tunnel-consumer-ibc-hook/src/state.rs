use cosmwasm_std::Addr;
use cw_band::tunnel::packet::Price;
use cw_controllers::Admin;
use cw_storage_plus::Map;

pub const ADMIN: Admin = Admin::new("admin");
pub const SENDERS: Map<Addr, ()> = Map::new("tunnel_config");
pub const SIGNAL_PRICE: Map<&str, Price> = Map::new("signal_price");
