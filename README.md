# cw-band

**Disclaimer** This library is in a very early stage of development. Please don't use it in production.

This repo contains `packages` and `contracts` folder to provide a library to build cosmwasm smart contract to interact with BandChain.

## Packages

Contain common data type that specific to BandChain oracle packet and some common input/output to request data to BandChain.

### OracleRequestPacketData
 - This is the type of MsgRequestData on BandChain. You have to specify each field of the mesages based on your requests.

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

### AcknowledgementMsg
 - AcknowledgeMsg of IBC will be either Result or Error. Result will be returned if the status of request is successful, otherwise, it'll be Error.

```
pub enum AcknowledgementMsg {
    Result(Binary),
    Error(String),
}
```

### BandAcknowledgement
 - The acknowledgement data that BandChain will return in the Result of AcknowledgementMsg. 

```
pub struct BandAcknowledgement {
    pub request_id: Uint64, // Request_id of the request.
}
```

### OracleResponsePacketData
 - The packet that BandChain will send to other chain after the request is resolved.

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

## Contracts

We provide sample contracts that either implement or consume these packages to both provide examples, and provide a basis for code you can extend for more custom contacts.

- Price feed
  - Update new rate by request data to BandChain via IBC
  - Allow other contracts to query data from this contract
  - You can learn more detail in this [document](https://github.com/bandprotocol/cw-band/blob/add-readme/docs/Overview.md)
- [TODO] Consumer 
  - Query price from price feed contract
