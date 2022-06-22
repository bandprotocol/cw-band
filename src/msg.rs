use obi::{OBIDecode, OBIEncode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Request { symbols: Vec<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetRate { symbol: String },
}
#[derive(OBIEncode, OBIDecode)]
pub struct BandCalldata {
    pub symbols: Vec<String>,
    pub multiplier: u64,
}

impl BandCalldata {
    pub fn new(symbols: Vec<String>) -> Self {
        BandCalldata {
            symbols,
            multiplier: 1000000000,
        }
    }
}

#[derive(OBIDecode)]
pub struct BandResult {
    pub rates: Vec<u64>,
}
