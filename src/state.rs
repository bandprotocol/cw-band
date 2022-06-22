use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::IbcEndpoint;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Rate {
    pub rate: u64,
    pub resolved_time: u64,
}

pub const RATES: Map<String, Rate> = Map::new("rates");

pub const ENDPOINT: Item<IbcEndpoint> = Item::new("endpoint");

pub const REQUESTS: Map<u64, Vec<String>> = Map::new("requests");
