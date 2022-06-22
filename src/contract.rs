#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_slice, to_binary, Binary, Deps, DepsMut, Empty, Env, Ibc3ChannelOpenResponse,
    IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcChannelOpenResponse, IbcMsg, IbcOrder, IbcPacketAckMsg, IbcPacketReceiveMsg,
    IbcPacketTimeoutMsg, IbcReceiveResponse, IbcTimeout, MessageInfo, Response, StdError,
    StdResult,
};
use cw2::set_contract_version;
use obi::{OBIDecode, OBIEncode};

use crate::band::{BandAcknowledgement, OracleRequestPacketData, OracleResponsePacketData};
use crate::error::ContractError;
use crate::msg::{BandCalldata, BandResult, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Rate, ENDPOINT, RATES, REQUESTS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:poc-price-feed";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const IBC_APP_VERSION: &str = "bandchain-1";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Request { symbols } => try_request(deps, env, symbols),
    }
}

pub fn try_request(
    deps: DepsMut,
    env: Env,
    symbols: Vec<String>,
) -> Result<Response, ContractError> {
    let endpoint = ENDPOINT.load(deps.storage)?;
    let raw_calldata =
        BandCalldata::new(symbols)
            .try_to_vec()
            .map_err(|err| ContractError::CustomError {
                val: err.to_string(),
            })?;
    Ok(Response::new().add_message(IbcMsg::SendPacket {
        channel_id: endpoint.channel_id,
        data: to_binary(&raw_calldata)?,
        timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(60)),
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetRate { symbol } => to_binary(&query_rate(deps, symbol)?),
    }
}

fn query_rate(deps: Deps, symbol: String) -> StdResult<Rate> {
    RATES.load(deps.storage, symbol)
}

#[entry_point]
/// enforces ordering and versioning constraints
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> StdResult<IbcChannelOpenResponse> {
    let channel = msg.channel();
    if channel.order != IbcOrder::Unordered {
        return Err(StdError::generic_err("Only supports unordered channels"));
    }

    // In ibcv3 we don't check the version string passed in the message
    // and only check the counterparty version.
    if let Some(counter_version) = msg.counterparty_version() {
        if counter_version != IBC_APP_VERSION {
            return Err(StdError::generic_err(format!(
                "Counterparty version must be `{}`",
                IBC_APP_VERSION
            )));
        }
    }

    // We return the version we need (which could be different than the counterparty version)
    Ok(Some(Ibc3ChannelOpenResponse {
        version: IBC_APP_VERSION.to_string(),
    }))
}

#[entry_point]
/// once it's established, we create the reflect contract
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    ENDPOINT.save(deps.storage, &msg.channel().endpoint)?;
    Ok(IbcBasicResponse::new())
}

#[entry_point]
pub fn ibc_channel_close(
    deps: DepsMut,
    env: Env,
    msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new())
}

/// this is a no-op just to test how this integrates with wasmd
#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> StdResult<Response> {
    Ok(Response::default())
}

// // this encode an error or error message into a proper acknowledgement to the recevier
// fn encode_ibc_error(msg: impl Into<String>) -> Binary {
//     // this cannot error, unwrap to keep the interface simple
//     to_binary(&AcknowledgementMsg::<()>::Err(msg.into())).unwrap()
// }

#[entry_point]
/// we look for a the proper reflect contract to relay to and send the message
/// We cannot return any meaningful response value as we do not know the response value
/// of execution. We just return ok if we dispatched, error if we failed to dispatch
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> StdResult<IbcReceiveResponse> {
    let packet = msg.packet;

    let resp: OracleResponsePacketData = from_slice(&packet.data)?;
    if resp.resolve_status != 1 {
        return Err(StdError::GenericErr {
            msg: "Resolve failed".into(),
        });
    }
    let symbols = REQUESTS.load(deps.storage, resp.request_id)?;
    let result: BandResult =
        OBIDecode::try_from_slice(&resp.result).map_err(|err| StdError::ParseErr {
            target_type: "Oracle response packet".into(),
            msg: "TODO".into(),
        })?;
    for (s, r) in symbols.iter().zip(result.rates.iter()) {
        RATES.save(
            deps.storage,
            s.into(),
            &Rate {
                rate: *r,
                resolved_time: resp.resolve_time,
            },
        )?;
    }
    Ok(IbcReceiveResponse::new())
}

#[entry_point]
/// TODO: Save request id to state
pub fn ibc_packet_ack(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    let packet: OracleRequestPacketData = from_slice(&msg.original_packet.data)?;
    let calldata: BandCalldata =
        OBIDecode::try_from_slice(&packet.calldata).map_err(|err| StdError::ParseErr {
            target_type: "Oracle request packet".into(),
            msg: "TODO".into(),
        })?;
    let ack: BandAcknowledgement = from_slice(&msg.acknowledgement.data)?;
    REQUESTS.save(deps.storage, ack.request_id, &calldata.symbols)?;
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_ack"))
}

#[entry_point]
/// TODO: Handle when didn't get response packet in time
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_timeout"))
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
//     use cosmwasm_std::{coins, from_binary};

//     #[test]
//     fn proper_initialization() {
//         let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(1000, "earth"));

//         // we can just call .unwrap() to assert this was a success
//         let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         // it worked, let's query the state
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(17, value.count);
//     }

//     #[test]
//     fn increment() {
//         let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // beneficiary can release it
//         let info = mock_info("anyone", &coins(2, "token"));
//         let msg = ExecuteMsg::Increment {};
//         let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // should increase counter by 1
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(18, value.count);
//     }

//     #[test]
//     fn reset() {
//         let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // beneficiary can release it
//         let unauth_info = mock_info("anyone", &coins(2, "token"));
//         let msg = ExecuteMsg::Reset { count: 5 };
//         let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
//         match res {
//             Err(ContractError::Unauthorized {}) => {}
//             _ => panic!("Must return unauthorized error"),
//         }

//         // only the original creator can reset the counter
//         let auth_info = mock_info("creator", &coins(2, "token"));
//         let msg = ExecuteMsg::Reset { count: 5 };
//         let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

//         // should now be 5
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(5, value.count);
//     }
// }
