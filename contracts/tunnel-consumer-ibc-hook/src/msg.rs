use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use cw_band::tunnel::packet::TunnelPacket;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    ReceivePacket { packet: TunnelPacket },
    AddSenders { msg: AddSendersMsg },
    RemoveSenders { msg: RemoveSendersMsg },
}

#[cw_serde]
pub struct AddSendersMsg {
    pub senders: Vec<Addr>,
}

#[cw_serde]
pub struct RemoveSendersMsg {
    pub senders: Vec<Addr>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<Option<cw_band::tunnel::packet::Price>>)]
    Prices { signal_ids: Vec<String> },
}
