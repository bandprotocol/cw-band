use band::Config;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::IbcEndpoint;
use cosmwasm_std::Uint256;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Rate {
    pub rate: u64,
    pub resolve_time: u64,
    pub request_id: u64,
}

pub const RATES: Map<&str, Rate> = Map::new("rates");

pub const ENDPOINT: Item<IbcEndpoint> = Item::new("endpoint");

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct ReferenceData {
    // Pair rate e.g. rate of BTC/USD
    pub rate: Uint256,
    // Unix time of when the base asset was last updated. e.g. Last update time of BTC in Unix time
    pub last_updated_base: u64,
    // Unix time of when the quote asset was last updated. e.g. Last update time of USD in Unix time
    pub last_updated_quote: u64,
}

impl ReferenceData {
    pub fn new(rate: Uint256, last_updated_base: u64, last_updated_quote: u64) -> Self {
        ReferenceData {
            rate,
            last_updated_base,
            last_updated_quote,
        }
    }
}
