# Packages - cw-band

Contain common data type that specific to BandChain oracle packet and some common input/output to request data to BandChain.

## OracleRequestPacketData
 - This is the type of MsgRequestData on BandChain. You have to specify each field of the messages based on your requests.

```
pub struct OracleRequestPacketData {
    pub client_id: String, // A unique ID for the oracle request.
    pub oracle_script_id: Uint64, // The oracle script ID to query.
    pub calldata: Vec<u8>, // Bytes of input data of oracle script.
    pub ask_count: Uint64, // The number of validators that are requested to respond.
    pub min_count: Uint64, // The minimum number of validators that need to respond.
    pub fee_limit: Vec<Coin>, // The maximum amount of band in uband to be paid to the data source providers.
    pub prepare_gas: Uint64, // Amount of gas to pay to prepare raw requests.
    pub execute_gas: Uint64, // Minimum number of sources required to return a successful response.
}
```

## AcknowledgementMsg
 - AcknowledgeMsg of IBC will be either Result or Error. The result will be returned if the status of the request is successful, otherwise, it'll be Error.

```
pub enum AcknowledgementMsg {
    Result(Binary),
    Error(String),
}
```

## BandAcknowledgement
The acknowledgment data that BandChain will return in the Result of AcknowledgementMsg. 

```
pub struct BandAcknowledgement {
    pub request_id: Uint64, // Request_id of the request.
}
```

## OracleResponsePacketData
 - The packet that BandChain will send to another chain after the request is resolved.

```
pub struct OracleResponsePacketData {
    pub client_id: String, // A unique ID for the oracle request (same value with client_id of the request).
    pub request_id: Uint64, // Request_id of the request.
    pub ans_count: Uint64, // The number of validators that are requested to respond.
    pub request_time: Uint64, // Timestamp of the request.
    pub resolve_time: Uint64, // Timestamp of resolving the request
    pub resolve_status: String, // Resolve status
    pub result: Binary, // Bytes of output from oracle script.
}
```
