#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, from_json, DepsMut, Env, Ibc3ChannelOpenResponse, IbcBasicResponse, IbcChannel,
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse, IbcPacket,
    IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, Never,
};

use cw_band::tunnel::packet::{ack_fail, ack_success, TunnelPacket};
use cw_band::tunnel::{TUNNEL_APP_VERSION, TUNNEL_ORDER};

use crate::state::{ALLOWABLE_TUNNEL_IDS, SIGNAL_PRICE};
use crate::ContractError;

#[cfg_attr(not(feature = "library"), entry_point)]
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
pub fn ibc_channel_connect(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;

    Ok(IbcBasicResponse::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _channel: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    Ok(IbcBasicResponse::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, Never> {
    let packet = msg.packet;

    do_ibc_packet_receive(deps, env, &packet).or_else(|err| {
        Ok(
            IbcReceiveResponse::new(ack_fail(err.to_string())).add_attributes(vec![
                attr("action", "receive"),
                attr("success", "false"),
                attr("error", err.to_string()),
            ]),
        )
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    Ok(IbcBasicResponse::default()
        .add_attribute("action", "acknowledge")
        .add_attribute("success", "true"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    Ok(IbcBasicResponse::default()
        .add_attribute("action", "acknowledge")
        .add_attribute("success", "false")
        .add_attribute("error", "timeout"))
}

fn enforce_order_and_version(
    channel: &IbcChannel,
    counterparty_version: Option<&str>,
) -> Result<(), ContractError> {
    // Check channel version
    if channel.version != TUNNEL_APP_VERSION {
        return Err(ContractError::InvalidTunnelVersion {
            actual: channel.version.clone(),
            expected: TUNNEL_APP_VERSION.to_string(),
        });
    }
    if let Some(version) = counterparty_version {
        if version != TUNNEL_APP_VERSION {
            return Err(ContractError::InvalidTunnelVersion {
                actual: version.to_string(),
                expected: TUNNEL_APP_VERSION.to_string(),
            });
        }
    }

    // IBC channel must be unordered
    if channel.order != TUNNEL_ORDER {
        return Err(ContractError::InvalidChannelOrder {
            actual: channel.order.clone(),
            expected: TUNNEL_ORDER,
        });
    }

    Ok(())
}

fn do_ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    packet: &IbcPacket,
) -> Result<IbcReceiveResponse, ContractError> {
    let tunnel_packet: TunnelPacket = from_json(&packet.data)?;

    let contract_addr = env.contract.address.to_string();
    if packet.dest.port_id != format!("wasm.{}", contract_addr)
        || !ALLOWABLE_TUNNEL_IDS.has(deps.storage, &packet.dest.channel_id)
    {
        return Err(ContractError::Unauthorized {});
    }

    for price in tunnel_packet.prices {
        let signal_id = &price.signal_id;
        match SIGNAL_PRICE.may_load(deps.storage, signal_id)? {
            // If there is no price for this signal, save it
            None => SIGNAL_PRICE.save(deps.storage, signal_id, &price)?,
            // If there is an existing price for this signal, save it only if it is newer
            Some(last_price) if last_price.timestamp < price.timestamp => {
                SIGNAL_PRICE.save(deps.storage, signal_id, &price)?
            }
            // Otherwise, do nothing
            _ => {}
        }
    }
    let res = IbcReceiveResponse::new(ack_success())
        .add_attribute("action", "receive")
        .add_attribute("success", "true");

    Ok(res)
}
