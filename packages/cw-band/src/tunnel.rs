use cosmwasm_std::IbcOrder;

pub mod packet;

pub const TUNNEL_APP_VERSION: &str = "tunnel-1";
pub const TUNNEL_ORDERING: IbcOrder = IbcOrder::Unordered;
