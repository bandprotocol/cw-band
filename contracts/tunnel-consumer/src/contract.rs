#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, IbcEndpoint, MessageInfo, Response,
    StdResult, Uint64,
};
use cw2::set_contract_version;

use cw_band::tunnel::packet::Price;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, UpdateTunnelConfigMsg};
use crate::state::{ADMIN, SIGNAL_PRICE, TUNNEL_CONFIG};

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
        ExecuteMsg::UpdateTunnelConfig(msg) => execute_set_tunnel_config(deps, info, msg),
    }
}

pub fn execute_set_tunnel_config(
    deps: DepsMut,
    info: MessageInfo,
    msg: UpdateTunnelConfigMsg,
) -> Result<Response, ContractError> {
    ADMIN
        .assert_admin(deps.as_ref(), &info.sender)
        .map_err(|_| ContractError::Unauthorized)?;

    let tunnel_id = msg.tunnel_id;
    let port_id = msg.port_id;
    let channel_id = msg.channel_id;

    let ibc_endpoint = IbcEndpoint {
        port_id,
        channel_id,
    };
    TUNNEL_CONFIG.save(deps.storage, &tunnel_id.to_string(), &ibc_endpoint)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin {} => to_json_binary(&query_admin(deps)?),
        QueryMsg::TunnelConfig { tunnel_id } => {
            to_json_binary(&query_tunnel_config(deps, tunnel_id)?)
        }
        QueryMsg::Price { signal_id } => to_json_binary(&query_price(deps, signal_id)?),
    }
}
fn query_admin(deps: Deps) -> StdResult<Option<Addr>> {
    ADMIN.get(deps)
}

fn query_tunnel_config(deps: Deps, tunnel_id: Uint64) -> StdResult<IbcEndpoint> {
    TUNNEL_CONFIG.load(deps.storage, &tunnel_id.to_string())
}

fn query_price(deps: Deps, signal_id: String) -> StdResult<Price> {
    SIGNAL_PRICE.load(deps.storage, &signal_id)
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
    use cw_band::tunnel::{TUNNEL_APP_VERSION, TUNNEL_ORDERING};

    use crate::ibc::{ibc_channel_connect, ibc_channel_open, ibc_packet_receive};
    use crate::msg::{InstantiateMsg, QueryMsg};

    use super::*;

    fn mock_channel() -> IbcChannel {
        let ibc_endpoint = IbcEndpoint {
            port_id: "tunnel".to_string(),
            channel_id: "channel-1".to_string(),
        };
        let counterparty_endpoint = IbcEndpoint {
            port_id: "tunnel".to_string(),
            channel_id: "channel-2".to_string(),
        };

        IbcChannel::new(
            ibc_endpoint,
            counterparty_endpoint,
            TUNNEL_ORDERING,
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

        let update_tunnel_config_msg = UpdateTunnelConfigMsg {
            tunnel_id: Uint64::from(1_u64),
            port_id: "tunnel".to_string(),
            channel_id: "channel-1".to_string(),
        };
        let exec_msg = ExecuteMsg::UpdateTunnelConfig(update_tunnel_config_msg);
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

        let ibc_endpoint = IbcEndpoint {
            port_id: "tunnel".to_string(),
            channel_id: "channel-1".to_string(),
        };
        let counterparty_endpoint = IbcEndpoint {
            port_id: "tunnel".to_string(),
            channel_id: "channel-2".to_string(),
        };

        let ibc_packet = IbcPacket::new(
            to_json_binary(&tunnel_packet).unwrap(),
            counterparty_endpoint,
            ibc_endpoint,
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
