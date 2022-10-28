#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_slice, to_binary, Binary, Deps, DepsMut, Empty, Env, Ibc3ChannelOpenResponse,
    IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcChannelOpenResponse, IbcMsg, IbcOrder, IbcPacketAckMsg, IbcPacketReceiveMsg,
    IbcPacketTimeoutMsg, IbcReceiveResponse, IbcTimeout, MessageInfo, Response, StdError,
    StdResult, Uint256, Uint64,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Rate, ReferenceData, CONFIG, ENDPOINT, RATES};
use ::obi::dec::OBIDecode;
use ::obi::enc::OBIEncode;

use band::{
    Config, Input, OracleRequestPacketData, OracleResponsePacketData, Output, IBC_APP_VERSION,
};

pub static E18: Uint256 = Uint256::from_u128(1_000_000_000_000_000_000u128);

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:band-ibc-price-feed";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(
        deps.storage,
        &Config {
            client_id: msg.client_id,
            oracle_script_id: msg.oracle_script_id,
            ask_count: msg.ask_count,
            min_count: msg.min_count,
            fee_limit: msg.fee_limit,
            prepare_gas: msg.prepare_gas,
            execute_gas: msg.execute_gas,
            minimum_sources: msg.minimum_sources,
        },
    )?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Request { symbols } => try_request(deps, env, symbols),
    }
}

// TODO: Implement bounty logic to incentivize relayer and no one can spam request for free
pub fn try_request(
    deps: DepsMut,
    env: Env,
    symbols: Vec<String>,
) -> Result<Response, ContractError> {
    let endpoint = ENDPOINT.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    let raw_calldata = Input {
        symbols,
        minimum_sources: config.minimum_sources,
    }
    .try_to_vec()
    .map_err(|err| ContractError::CustomError {
        val: err.to_string(),
    })?;

    let packet = OracleRequestPacketData {
        client_id: config.client_id,
        ask_count: config.ask_count.into(),
        min_count: config.min_count.into(),
        calldata: raw_calldata,
        prepare_gas: config.prepare_gas.into(),
        execute_gas: config.execute_gas.into(),
        oracle_script_id: config.oracle_script_id.into(),
        fee_limit: config.fee_limit,
    };
    Ok(Response::new().add_message(IbcMsg::SendPacket {
        channel_id: endpoint.channel_id,
        data: to_binary(&packet)?,
        timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(60)),
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetRate { symbol } => to_binary(&query_rate(deps, &symbol)?),
        QueryMsg::GetReferenceData { symbol_pair } => {
            to_binary(&query_reference_data(deps, &symbol_pair)?)
        }
        QueryMsg::GetReferenceDataBulk { symbol_pairs } => {
            to_binary(&query_reference_data_bulk(deps, &symbol_pairs)?)
        }
    }
}

fn query_rate(deps: Deps, symbol: &str) -> StdResult<Rate> {
    RATES.load(deps.storage, symbol)
}

fn query_reference_data(deps: Deps, symbol_pair: &(String, String)) -> StdResult<ReferenceData> {
    let base = query_rate(deps, &symbol_pair.0)?;
    let quote = query_rate(deps, &symbol_pair.1)?;

    Ok(ReferenceData::new(
        Uint256::from(base.rate)
            .checked_mul(E18)?
            .checked_div(Uint256::from(quote.rate))?,
        base.resolve_time,
        quote.resolve_time,
    ))
}

fn query_reference_data_bulk(
    deps: Deps,
    symbol_pairs: &[(String, String)],
) -> StdResult<Vec<ReferenceData>> {
    symbol_pairs
        .iter()
        .map(|pair| query_reference_data(deps, pair))
        .collect()
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
    _deps: DepsMut,
    _env: Env,
    _msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new())
}

/// this is a no-op just to test how this integrates with wasmd
#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> StdResult<Response> {
    Ok(Response::default())
}

#[entry_point]
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> StdResult<IbcReceiveResponse> {
    let packet = msg.packet;

    let resp: OracleResponsePacketData = from_slice(&packet.data)?;
    if resp.resolve_status.u64() != 1 {
        // Prevent replay relay failed packet
        return Ok(IbcReceiveResponse::new().add_attribute("action", "ibc_packet_received"));
    }
    let result: Output =
        OBIDecode::try_from_slice(&resp.result).map_err(|err| StdError::ParseErr {
            target_type: "Oracle response packet".into(),
            msg: err.to_string(),
        })?;

    for r in result.responses {
        if r.response_code == 0 {
            let rate = RATES.may_load(deps.storage, &r.symbol)?;
            if rate.is_none() || rate.unwrap().resolve_time < resp.resolve_time {
                RATES.save(
                    deps.storage,
                    &r.symbol,
                    &Rate {
                        rate: Uint64::from(r.rate),
                        resolve_time: resp.resolve_time,
                        request_id: resp.request_id,
                    },
                )?;
            }
        }
    }
    Ok(IbcReceiveResponse::new().add_attribute("action", "ibc_packet_received"))
}

#[entry_point]
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
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

// TODO: Writing test
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
