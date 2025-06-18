#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw_band::tunnel::packet::{Price, TunnelPacket};

use crate::error::ContractError;
use crate::msg::{AddSendersMsg, ExecuteMsg, InstantiateMsg, QueryMsg, RemoveSendersMsg};
use crate::state::{ADMIN, SENDERS, SIGNAL_PRICE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:tunnel-consumer-ibc-hook";
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
        ExecuteMsg::ReceivePacket { packet } => execute_receive_packet(deps, info, packet),
        ExecuteMsg::AddSenders { msg } => execute_add_senders(deps, info, msg),
        ExecuteMsg::RemoveSenders { msg } => execute_remove_senders(deps, info, msg),
    }
}

pub fn execute_add_senders(
    deps: DepsMut,
    info: MessageInfo,
    msg: AddSendersMsg,
) -> Result<Response, ContractError> {
    ADMIN
        .assert_admin(deps.as_ref(), &info.sender)
        .map_err(|_| ContractError::Unauthorized)?;

    for sender in msg.senders {
        SENDERS.save(deps.storage, sender, &())?;
    }

    let res = Response::new()
        .add_attribute("action", "add_senders")
        .add_attribute("success", "true");
    Ok(res)
}

pub fn execute_remove_senders(
    deps: DepsMut,
    info: MessageInfo,
    msg: RemoveSendersMsg,
) -> Result<Response, ContractError> {
    ADMIN
        .assert_admin(deps.as_ref(), &info.sender)
        .map_err(|_| ContractError::Unauthorized)?;

    for sender in msg.senders {
        SENDERS.remove(deps.storage, sender);
    }

    let res = Response::new()
        .add_attribute("action", "remove_senders")
        .add_attribute("success", "true");
    Ok(res)
}

pub fn execute_receive_packet(
    deps: DepsMut,
    info: MessageInfo,
    packet: TunnelPacket,
) -> Result<Response, ContractError> {
    if !SENDERS.has(deps.storage, info.sender) {
        return Err(ContractError::Unauthorized);
    }

    for price in packet.prices {
        let signal_id = &price.signal_id;
        match SIGNAL_PRICE.may_load(deps.storage, signal_id)? {
            // If there is no existing price for this signal, save it
            None => SIGNAL_PRICE.save(deps.storage, signal_id, &price)?,
            // If there is an existing price for this signal, save it only if it is newer
            Some(last_price) if last_price.timestamp < price.timestamp => {
                SIGNAL_PRICE.save(deps.storage, signal_id, &price)?
            }
            // Otherwise, do nothing
            _ => {}
        }
    }

    let res = Response::new()
        .add_attribute("action", "receive_packet")
        .add_attribute("success", "true");
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Prices { signal_ids } => to_json_binary(&query_prices(deps, signal_ids)?),
    }
}

fn query_prices(deps: Deps, signal_ids: Vec<String>) -> StdResult<Vec<Option<Price>>> {
    signal_ids
        .iter()
        .map(|id| SIGNAL_PRICE.may_load(deps.storage, id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{from_json, Addr, Int64, OwnedDeps, Uint64};
    use cw_band::tunnel::packet::Status;

    const E9: u64 = 1_000_000_000;

    fn setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();

        // instantiate an empty contract
        let instantiate_msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("admin"), &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();
        execute_add_senders(
            deps.as_mut(),
            info.clone(),
            AddSendersMsg {
                senders: vec![Addr::unchecked("sender")],
            },
        )
        .unwrap();

        deps
    }

    #[test]
    fn test_execute_receive_packet() {
        let mut deps = setup();

        let prices = vec![
            Price {
                signal_id: "CS:BTC-USD".to_string(),
                status: Status::Available,
                price: Uint64::new(100000 * E9),
                timestamp: Default::default(),
            },
            Price {
                signal_id: "CS:ETH-USD".to_string(),
                status: Status::Available,
                price: Uint64::new(3000 * E9),
                timestamp: Default::default(),
            },
        ];

        let packet = TunnelPacket {
            tunnel_id: Uint64::new(1),
            sequence: Uint64::new(0),
            prices,
            created_at: Int64::new(0),
        };

        let info = message_info(&Addr::unchecked("sender"), &[]);
        let msg = ExecuteMsg::ReceivePacket { packet };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(res.attributes[1].value, "true");
    }

    #[test]
    fn test_execute_receive_packet_with_invalid_sender() {
        let mut deps = setup();

        let prices = vec![
            Price {
                signal_id: "CS:BTC-USD".to_string(),
                status: Status::Available,
                price: Uint64::new(100000 * E9),
                timestamp: Default::default(),
            },
            Price {
                signal_id: "CS:ETH-USD".to_string(),
                status: Status::Available,
                price: Uint64::new(3000 * E9),
                timestamp: Default::default(),
            },
        ];

        let packet = TunnelPacket {
            tunnel_id: Uint64::new(1),
            sequence: Uint64::new(0),
            prices,
            created_at: Int64::new(0),
        };

        let info = message_info(&Addr::unchecked("random"), &[]);
        let msg = ExecuteMsg::ReceivePacket { packet };
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err, ContractError::Unauthorized);
    }

    #[test]
    fn test_query_msg() {
        let mut deps = setup();

        let prices = vec![Price {
            signal_id: "CS:BTC-USD".to_string(),
            status: Status::Available,
            price: Uint64::new(100000 * E9),
            timestamp: Default::default(),
        }];

        let packet = TunnelPacket {
            tunnel_id: Uint64::new(1),
            sequence: Uint64::new(0),
            prices: prices.clone(),
            created_at: Int64::new(0),
        };

        let info = message_info(&Addr::unchecked("sender"), &[]);
        let msg = ExecuteMsg::ReceivePacket { packet };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let res_bin = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Prices {
                signal_ids: vec!["CS:BTC-USD".to_string()],
            },
        )
        .unwrap();
        let res = from_json::<Vec<Option<Price>>>(&res_bin).unwrap();

        let expected = prices.into_iter().map(Some).collect::<Vec<Option<Price>>>();
        assert_eq!(res, expected);
    }

    #[test]
    fn test_query_msg_missing() {
        let deps = setup();

        let binary = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Prices {
                signal_ids: vec!["CS:BTC-USD".to_string()],
            },
        )
        .unwrap();
        let res = from_json::<Vec<Option<Price>>>(&binary).unwrap();
        assert_eq!(res, vec![None]);
    }
}
