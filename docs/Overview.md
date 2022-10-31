# How to Request Data on BandChain from a CosmWasm contract via IBC

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

During this stage, a channel between your contract and our oracle module on BandChain needs to be created in order to relay an IBC message via a relayer. In order to do that, 2 entry points needs to be provided.

**Note**: As creating a channel can be started from either the contract or the oracle; A function that accept messages regardless of the initiator side needs to be provided.

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
        version: IBC_APP_VERSION.to_string(),
    }))

}
```

This function should verify the order type of channel and the counterparty’s version and respond its own version to the counterparty’s module.

### IBC Channel Connect

`ibc_channel_connect (OpenAck, OpenConfirm)`

```rust
#[entry_point]
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {

    // logic to store channel_id for sending future IBC messages
    // ...

    Ok(IbcBasicResponse::new())
}
```

This function is called along with the channel detail so that the `channel_id` or other detail can be stored for any future IBC actions.

## Request Msg

In order to send an IBC message to request data from BandChain, cosmwasm_std::IbcMsg::SendPacket needs to be used with the following three parameters:

- channel_id: The channel_id that you want to send packet to
- data: The binary data of the packet that you want to send contained in a OracleRequestPacketData structure.
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

As the message have already been sent, three callback functions will need to be provided to accept the three outgoing messages from Oracle: `IbcPacketAckMsg`, `IbcPacketTimeoutMsg` and `IbcPacketReceiveMsg`.

### IBCPacketAckMsg

```rust
#[entry_point]
pub fn ibc_packet_ack(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {

    let res: AcknowledgementMsg<Binary> = from_slice(&msg.acknowledgement.data)?;
    match res {
        AcknowledgementMsg::Result(bz) => {
            let ack: BandAcknowledgement = from_slice(&bz)?;
            Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_ack"))
        }
        AcknowledgementMsg::Err(err) => Err(StdError::GenericErr {
            msg: format!("Fail ack err: {:}", err),
        }),
    }
}
```

The oracle will send an acknowledgement message with the corresponding request_id on BandChain if the requested can be processed so that the sender’s side can process the data as needed.

### IBCPacketReceiveMsg

```rust
#[entry_point]
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

    let symbols = REQUESTS.load(deps.storage, resp.request_id.into())?;
    let result: BandResult =
        OBIDecode::try_from_slice(&resp.result).map_err(|err| StdError::ParseErr {
            target_type: "Oracle response packet".into(),
            msg: err.to_string(),
        })?;

    Ok(IbcReceiveResponse::new().set_ack(vec![1]))
}
```

After the oracle finishs your request, an OracleResponsePacketData packet will be sent to this function in your contract. The output of the oracle script that you requested will be contained in the result field.

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

In the case where an acknowledgement message from the destination module hasn’t being received, the relayers will call this function. Requests get that timeout can be handled within this function. e.g. emitting the event or marking request status.
