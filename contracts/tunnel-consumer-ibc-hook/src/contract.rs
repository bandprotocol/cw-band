#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw_band::tunnel::packet::{Price, TunnelPacket};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SetTunnelConfigMsg};
use crate::state::{TunnelConfig, ADMIN, SIGNAL_PRICE, TUNNEL_CONFIG};

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
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ReceivePacket { packet } => execute_receive_packet(deps, info, packet),
        ExecuteMsg::SetTunnelConfig { msg } => execute_set_tunnel_config(deps, info, msg),
    }
}

pub fn execute_set_tunnel_config(
    deps: DepsMut,
    info: MessageInfo,
    msg: SetTunnelConfigMsg,
) -> Result<Response, ContractError> {
    ADMIN
        .assert_admin(deps.as_ref(), &info.sender)
        .map_err(|_| ContractError::Unauthorized)?;

    let config = TunnelConfig {
        tunnel_id: msg.tunnel_id,
        sender: msg.sender,
        port_id: msg.port_id,
        channel_id: msg.channel_id,
    };
    TUNNEL_CONFIG.save(deps.storage, &msg.tunnel_id.to_string(), &config)?;

    let res = Response::new()
        .add_attribute("action", "set_tunnel_config")
        .add_attribute("success", "true");
    Ok(res)
}

pub fn execute_receive_packet(
    deps: DepsMut,
    info: MessageInfo,
    packet: TunnelPacket,
) -> Result<Response, ContractError> {
    let config = TUNNEL_CONFIG
        .load(deps.storage, &packet.tunnel_id.to_string())
        .or(Err(ContractError::Unauthorized {}))?;

    if config.sender != info.sender || config.tunnel_id != packet.tunnel_id {
        return Err(ContractError::Unauthorized {});
    }

    // Add config
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
    signal_ids.iter().map(|id| SIGNAL_PRICE.may_load(deps.storage, id)).collect()
}

#[cfg(test)]
mod tests {}
