#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use cw_band::tunnel::packet::Price;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ADMIN, ALLOWABLE_TUNNEL_IDS, SIGNAL_PRICE};

const CONTRACT_NAME: &str = "crates.io:tunnel-consumer";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ADMIN.set(deps.branch(), Some(info.sender))?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateAdmin { admin } => {
            let admin_addr = deps.api.addr_validate(&admin)?;
            ADMIN
                .execute_update_admin(deps, info, Some(admin_addr))
                .map_err(|_| ContractError::Unauthorized)
        }
        ExecuteMsg::AddAllowableChannelIds { channel_ids } => {
            add_allowable_channel_ids(deps, info, channel_ids)
        }
        ExecuteMsg::RemoveAllowableChannelIds { channel_ids } => {
            remove_allowable_channel_ids(deps, info, channel_ids)
        }
    }
}

pub fn add_allowable_channel_ids(
    deps: DepsMut,
    info: MessageInfo,
    channel_ids: Vec<String>,
) -> Result<Response, ContractError> {
    ADMIN
        .assert_admin(deps.as_ref(), &info.sender)
        .map_err(|_| ContractError::Unauthorized)?;

    for channel_id in channel_ids.iter() {
        ALLOWABLE_TUNNEL_IDS.save(deps.storage, channel_id, &())?;
    }

    Ok(Response::default())
}

pub fn remove_allowable_channel_ids(
    deps: DepsMut,
    info: MessageInfo,
    channel_ids: Vec<String>,
) -> Result<Response, ContractError> {
    ADMIN
        .assert_admin(deps.as_ref(), &info.sender)
        .map_err(|_| ContractError::Unauthorized)?;

    for channel_id in channel_ids.iter() {
        ALLOWABLE_TUNNEL_IDS.remove(deps.storage, channel_id);
    }

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin {} => to_json_binary(&query_admin(deps)?),
        QueryMsg::IsChannelIdAllowed { channel_id } => {
            to_json_binary(&query_is_channel_id_allowed(deps, channel_id))
        }
        QueryMsg::Price { signal_id } => to_json_binary(&query_price(deps, signal_id)?),
        QueryMsg::Prices { signal_ids } => {
            let prices = signal_ids
                .iter()
                .map(|signal_id| query_price(deps, signal_id.to_string()))
                .collect::<StdResult<Vec<Option<Price>>>>()?;
            to_json_binary(&prices)
        }
    }
}
fn query_admin(deps: Deps) -> StdResult<Option<Addr>> {
    ADMIN.get(deps)
}

fn query_is_channel_id_allowed(deps: Deps, channel_id: String) -> bool {
    ALLOWABLE_TUNNEL_IDS.has(deps.storage, &channel_id)
}

fn query_price(deps: Deps, signal_id: String) -> StdResult<Option<Price>> {
    SIGNAL_PRICE.may_load(deps.storage, &signal_id)
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        to_json_binary, Addr, DepsMut, IbcChannel, IbcChannelConnectMsg, IbcChannelOpenMsg,
        IbcEndpoint, IbcPacket, IbcPacketReceiveMsg, IbcReceiveResponse, IbcTimeout, Int64,
        OwnedDeps, Timestamp,
    };

    use cw_band::tunnel::packet::{ack_success, Price, Status, TunnelPacket};
    use cw_band::tunnel::{TUNNEL_APP_VERSION, TUNNEL_ORDER};

    use crate::ibc::{ibc_channel_connect, ibc_channel_open, ibc_packet_receive};
    use crate::msg::{InstantiateMsg, QueryMsg};

    use super::*;

    fn mock_channel() -> IbcChannel {
        let bandchain_endpoint = IbcEndpoint {
            port_id: "tunnel-1".to_string(),
            channel_id: "channel-1".to_string(),
        };
        let contract_endpoint = IbcEndpoint {
            port_id: format!("wasm.{}", mock_env().contract.address),
            channel_id: "channel-2".to_string(),
        };

        IbcChannel::new(
            bandchain_endpoint,
            contract_endpoint,
            TUNNEL_ORDER,
            TUNNEL_APP_VERSION,
            "connection-1",
        )
    }

    fn add_mock_channel(mut deps: DepsMut) {
        let channel = mock_channel();

        let open_msg = IbcChannelOpenMsg::new_init(channel.clone());
        ibc_channel_open(deps.branch(), mock_env(), open_msg).unwrap();
        let connect_msg = IbcChannelConnectMsg::new_ack(channel, TUNNEL_APP_VERSION);
        ibc_channel_connect(deps.branch(), mock_env(), connect_msg).unwrap();
    }

    fn setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();

        // instantiate an empty contract
        let instantiate_msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("admin"), &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        add_mock_channel(deps.as_mut());

        let exec_msg = ExecuteMsg::AddAllowableChannelIds {
            channel_ids: vec!["channel-2".to_string()],
        };
        execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();

        deps
    }

    #[test]
    fn test_recv_ibc_packet() {
        let mut deps = setup();

        let prices = vec![
            Price::new("CS:TEST1-USD", Status::Available, 1000_u64, 1000),
            Price::new("CS:TEST2-USD", Status::Available, 2000_u64, 1001),
            Price::new("CS:TEST3-USD", Status::Available, 3000_u64, 1002),
        ];
        let tunnel_packet = TunnelPacket::new(1_u64, 360_u64, prices.clone(), Int64::from(1600000));

        let mock_channel = mock_channel();
        let bandchain_ibc_endpoint = mock_channel.endpoint.clone();
        let contract_ibc_endpoint = mock_channel.counterparty_endpoint;
        
        let ibc_packet = IbcPacket::new(
            to_json_binary(&tunnel_packet).unwrap(),
            bandchain_ibc_endpoint,
            contract_ibc_endpoint,
            1,
            IbcTimeout::with_timestamp(Timestamp::from_nanos(1000000000)),
        );
        let ibc_packet_receive_msg =
            IbcPacketReceiveMsg::new(ibc_packet, Addr::unchecked("relayer"));
        let ibc_resp =
            ibc_packet_receive(deps.as_mut(), mock_env(), ibc_packet_receive_msg).unwrap();
        let expected_ibc_resp = IbcReceiveResponse::new(ack_success())
            .add_attribute("action", "receive")
            .add_attribute("success", "true");
        assert_eq!(ibc_resp, expected_ibc_resp);

        let test1_price = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Price {
                signal_id: "CS:TEST1-USD".to_string(),
            },
        )
        .unwrap();
        assert_eq!(test1_price, to_json_binary(&prices[0]).unwrap());

        let test2_price = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Price {
                signal_id: "CS:TEST2-USD".to_string(),
            },
        )
        .unwrap();
        assert_eq!(test2_price, to_json_binary(&prices[1]).unwrap());

        let test3_price = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Price {
                signal_id: "CS:TEST3-USD".to_string(),
            },
        )
        .unwrap();
        assert_eq!(test3_price, to_json_binary(&prices[2]).unwrap());
    }
}
