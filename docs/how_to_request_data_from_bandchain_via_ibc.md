# How to Request Data on BandChain via IBC

In order to request data from BandChain’s Oracle via a CosmWasm contract, your contract will need to implement the functions listed below:

```
ibc_channel_open()
ibc_channel_connect()
ibc_packet_ack()
ibc_packet_receive()
ibc_packet_timeout()
ibc_channel_close()
```

## IBC Connection

The behavior of the functions mentioned can be simplified into 2 stages of the IBC protocol communication.

1. Create Channel
2. Request Message

## Create Channel

During this stage, a channel between your contract and our oracle module on BandChain needs to be created to relay an IBC message via a relayer. To do that, 2 entry points need to be provided.

**Note**: As creating a channel can be started from either the contract or the oracle; A function that accepts messages regardless of the initiator side needs to be provided.

### IBC Channel Open

`ibc_channel_open (OpenInit, OpenTry)`

```rust
#[entry_point]
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> StdResult<IbcChannelOpenResponse> {

    // logic to verify channel order and the counterparty's version
    // ...

    Ok(Some(Ibc3ChannelOpenResponse {
        version: msg.channel().version.clone(),
    }))
}
```

This function should verify the order type of the channel and the counterparty’s version and respond to its version to the counterparty’s module.

### IBC Channel Connect

`ibc_channel_connect (OpenAck, OpenConfirm)`

```rust
#[entry_point]
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {

    // logic to store channel_id for sending future IBC messages
    // ...

    Ok(IbcBasicResponse::default())
}
```

This function is called along with the channel detail so that the `channel_id` or other detail can be stored for any future IBC actions.

## Request Msg

In order to send an IBC message to request data from BandChain, cosmwasm_std::IbcMsg::SendPacket needs to be used with the following three parameters:

- channel_id: The channel_id that you want to send a packet to
- data: The binary data of the packet that you want to send contained in an OracleRequestPacketData structure.
- timeout: The timeout timestamp.

Where the structure for SendPacket and OracleRequestPacketData are shown below:

```rust
IbcMsg::SendPacket {
    channel_id: endpoint.channel_id,
    data: to_binary(&oracleRequestPacketData)?,
    timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(60)),
}
```

```rust
pub struct OracleRequestPacketData {
    pub client_id: String,
    pub oracle_script_id: Uint64,
    pub calldata: Vec<u8>,
    pub ask_count: Uint64,
    pub min_count: Uint64,
    pub fee_limit: Vec<Coin>,
    pub prepare_gas: Uint64,
    pub execute_gas: Uint64,
}
```

As the message has already been sent, three callback functions will need to be provided to accept the three outgoing messages from Oracle: `IbcPacketAckMsg`, `IbcPacketTimeoutMsg` and `IbcPacketReceiveMsg`.

### IBCPacketAckMsg

```rust
#[entry_point]
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_ack"))
}
```

The oracle will send an acknowledgement message with the corresponding request_id on BandChain if the request can be processed so that the sender’s side can process the data as needed.

### IBCPacketReceiveMsg

```rust
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
    if resp.resolve_status != "RESOLVE_STATUS_SUCCESS" {
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
```

After BandChain finishes your request, an OracleResponsePacketData packet will be sent to this function in your contract. The output of the oracle script that you requested will be contained in the result field.

```rust
pub struct OracleResponsePacketData {
    pub client_id: String,
    pub request_id: Uint64,
    pub ans_count: Uint64,
    pub request_time: Uint64,
    pub resolve_time: Uint64,
    pub resolve_status: String,
    pub result: Vec<u8>,
}
```

### IBCPacketTimeoutMsg

```rust
#[entry_point]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_timeout"))
}
```

In the case where an acknowledgement message from the destination module hasn’t been received, the relayers will call this function. Requests get that timeout can be handled within this function. e.g. emitting the event or marking request status.
