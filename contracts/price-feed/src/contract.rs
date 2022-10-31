#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, from_slice, to_binary, Binary, Deps, DepsMut, Empty, Env, Ibc3ChannelOpenResponse,
    IbcBasicResponse, IbcChannel, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcChannelOpenResponse, IbcMsg, IbcOrder, IbcPacket, IbcPacketAckMsg, IbcPacketReceiveMsg,
    IbcPacketTimeoutMsg, IbcReceiveResponse, IbcTimeout, MessageInfo, Response, StdError,
    StdResult, Uint256, Uint64,
};
use cw2::set_contract_version;

use crate::error::{ContractError, Never};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Rate, ReferenceData, CONFIG, ENDPOINT, RATES};
use ::obi::dec::OBIDecode;
use ::obi::enc::OBIEncode;

use cw_band::{
    ack_fail, ack_success, Config, Input, OracleRequestPacketData, OracleResponsePacketData,
    Output, IBC_APP_VERSION,
};

const E9: Uint64 = Uint64::new(1_000_000_000u64);
const E18: Uint256 = Uint256::from_u128(1_000_000_000_000_000_000u128);

// Version info for migration
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

// TODO: Possible features
// - Bounty logic to incentivize relayer and no one can spam request for free
// - Whitelist who can call update price
pub fn try_request(
    deps: DepsMut,
    env: Env,
    symbols: Vec<String>,
) -> Result<Response, ContractError> {
    let endpoint = ENDPOINT.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    // TODO: Maybe helper function in cw-band for creating OracleRequestPacketData
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

/// this is a no-op
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> StdResult<Response> {
    Ok(Response::default())
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
    if symbol == "USD" {
        Ok(Rate::new(E9, Uint64::MAX, Uint64::new(0)))
    } else {
        RATES.load(deps.storage, symbol)
    }
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

#[cfg_attr(not(feature = "library"), entry_point)]
/// enforces ordering and versioning constraints
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, ContractError> {
    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;

    Ok(Some(Ibc3ChannelOpenResponse {
        version: msg.channel().version.clone(),
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// record the channel in ENDPOINT
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // we need to check the counter party version in try and ack (sometimes here)
    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;

    ENDPOINT.save(deps.storage, &msg.channel().endpoint)?;
    Ok(IbcBasicResponse::default())
}

fn enforce_order_and_version(
    channel: &IbcChannel,
    counterparty_version: Option<&str>,
) -> Result<(), ContractError> {
    if channel.version != IBC_APP_VERSION {
        return Err(ContractError::InvalidIbcVersion {
            version: channel.version.clone(),
        });
    }
    if let Some(version) = counterparty_version {
        if version != IBC_APP_VERSION {
            return Err(ContractError::InvalidIbcVersion {
                version: version.to_string(),
            });
        }
    }
    if channel.order != IbcOrder::Unordered {
        return Err(ContractError::OnlyUnorderedChannel {});
    }
    Ok(())
}

#[entry_point]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    unimplemented!();
}

#[entry_point]
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, Never> {
    let packet = msg.packet;

    do_ibc_packet_receive(deps, &packet).or_else(|err| {
        Ok(IbcReceiveResponse::new()
            .set_ack(ack_fail(err.to_string()))
            .add_attributes(vec![
                attr("action", "receive"),
                attr("success", "false"),
                attr("error", err.to_string()),
            ]))
    })
}

fn do_ibc_packet_receive(
    deps: DepsMut,
    packet: &IbcPacket,
) -> Result<IbcReceiveResponse, ContractError> {
    let resp: OracleResponsePacketData = from_slice(&packet.data)?;
    if resp.resolve_status.u64() != 1 {
        return Err(ContractError::RequestNotSuccess {});
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
    Ok(IbcReceiveResponse::new()
        .set_ack(ack_success())
        .add_attribute("action", "ibc_packet_received"))
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
#[cfg(test)]
mod tests {}
