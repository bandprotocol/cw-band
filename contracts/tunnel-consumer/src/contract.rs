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
mod tests {}
