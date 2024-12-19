use cw_controllers::Admin;
use cw_storage_plus::Map;

use cw_band::tunnel::packet::Price;

pub const ADMIN: Admin = Admin::new("admin");
pub const ALLOWABLE_TUNNEL_IDS: Map<&str, ()> = Map::new("allowable_tunnel_ids");
pub const SIGNAL_PRICE: Map<&str, Price> = Map::new("signal_price");
