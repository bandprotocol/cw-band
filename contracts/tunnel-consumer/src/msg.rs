use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct AddAllowableChannelIdsMsg {
    pub channel_ids: Vec<String>,
}

#[cw_serde]
pub struct RemoveAllowableChannelIdsMsg {
    pub channel_ids: Vec<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateAdmin { admin: String },
    AddAllowableChannelIds { channel_ids: Vec<String> },
    RemoveAllowableChannelIds { channel_ids: Vec<String> },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Option<cosmwasm_std::Addr>)]
    Admin {},
    #[returns(bool)]
    IsChannelIdAllowed { channel_id: String },
    #[returns(Option<cw_band::tunnel::packet::Price>)]
    Price { signal_id: String },
    #[returns(Vec<Option<cw_band::tunnel::packet::Price>>)]
    Prices { signal_ids: Vec<String> },
}
