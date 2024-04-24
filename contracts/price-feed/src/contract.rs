#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    has_coins, to_json_binary, Binary, Coin, Deps, DepsMut, Empty, Env, IbcMsg, IbcTimeout,
    MessageInfo, Response, StdResult, Uint256, Uint64,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, Rate, ReferenceData, RequestConfig, CONFIG, ENDPOINT, RATES};
use obi::enc::OBIEncode;

use cw_band::{Input, OracleRequestPacketData};

// WARNING /////////////////////////////////////////////////////////////////////////
// THIS CONTRACT IS AN EXAMPLE HOW TO USE CW_BAND TO WRITE CONTRACT.              //
// PLEASE USE THIS CODE AS THE REFERENCE AND NOT USE THIS CODE IN PRODUCTION.     //
////////////////////////////////////////////////////////////////////////////////////

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
            band_request: RequestConfig {
                client_id: msg.client_id,
                oracle_script_id: msg.oracle_script_id,
                ask_count: msg.ask_count,
                min_count: msg.min_count,
                fee_limit: msg.fee_limit,
                prepare_gas: msg.prepare_gas,
                execute_gas: msg.execute_gas,
                minimum_sources: msg.minimum_sources,
            },
            fee: msg.fee,
        },
    )?;

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
        ExecuteMsg::Request { symbols } => try_request(deps, env, symbols, info.funds),
    }
}

// TODO: Possible features
// - Request fee + Bounty logic to prevent request spam and incentivize relayer
// - Whitelist who can call update price
pub fn try_request(
    deps: DepsMut,
    env: Env,
    symbols: Vec<String>,
) -> Result<Response, ContractError> {
    let endpoint = ENDPOINT.load(deps.storage)?;
    let config = BAND_CONFIG.load(deps.storage)?;

    // TODO: Maybe helper function in cw-band for creating OracleRequestPacketData
    let raw_calldata = Input {
        symbols,
        minimum_sources: config.minimum_sources,
    }
    .try_to_vec()
    .map(Binary::new)
    .map_err(|err| ContractError::CustomError {
        val: err.to_string(),
    })?;

    let packet = OracleRequestPacketData {
        client_id: config.client_id,
        oracle_script_id: config.oracle_script_id,
        calldata: raw_calldata,
        ask_count: config.ask_count,
        min_count: config.min_count,
        prepare_gas: config.prepare_gas,
        execute_gas: config.execute_gas,
        fee_limit: config.fee_limit,
    };

    Ok(Response::new().add_message(IbcMsg::SendPacket {
        channel_id: endpoint.channel_id,
        data: to_json_binary(&packet)?,
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
        QueryMsg::GetRate { symbol } => to_json_binary(&query_rate(deps, &symbol)?),
        QueryMsg::GetReferenceData { symbol_pair } => {
            to_json_binary(&query_reference_data(deps, &symbol_pair)?)
        }
        QueryMsg::GetReferenceDataBulk { symbol_pairs } => {
            to_json_binary(&query_reference_data_bulk(deps, &symbol_pairs)?)
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

#[cfg(test)]
mod tests {

    use super::*;
    use cosmwasm_std::{
        coin, coins, Addr, AllBalanceResponse, Api, BankQuery, CustomMsg, CustomQuery, Empty,
        Storage,
    };
    use cw_multi_test::{
        App, AppBuilder, Bank, Contract, ContractWrapper, Distribution, Executor, Ibc,
        IbcAcceptingModule, IntoAddr, Module, Staking, Wasm,
    };
    use serde::de::DeserializeOwned;

    fn get_std_price_ref_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    /// Utility function for generating user addresses.
    fn addr_make(addr: &str) -> Addr {
        addr.into_addr()
    }

    fn query_app<BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT>(
        app: &App<BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT>,
        rcpt: &Addr,
    ) -> Vec<Coin>
    where
        CustomT::ExecT: CustomMsg + DeserializeOwned + 'static,
        CustomT::QueryT: CustomQuery + DeserializeOwned + 'static,
        WasmT: Wasm<CustomT::ExecT, CustomT::QueryT>,
        BankT: Bank,
        ApiT: Api,
        StorageT: Storage,
        CustomT: Module,
        StakingT: Staking,
        DistrT: Distribution,
        IbcT: Ibc,
    {
        let query = BankQuery::AllBalances {
            address: rcpt.into(),
        }
        .into();
        let val: AllBalanceResponse = app.wrap().query(&query).unwrap();
        val.amount
    }

    #[test]
    fn deploy_free_contract() {
        let owner = addr_make("owner");
        let init_funds = coins(300000000, "uband");

        // let mut app = App::new();

        let mut app =
            AppBuilder::new()
                .with_ibc(IbcAcceptingModule::new())
                .build(|router, _, storage| {
                    router
                        .bank
                        .init_balance(storage, &owner, init_funds)
                        .unwrap();
                });

        // set up contract
        let code_id = app.store_code(get_std_price_ref_contract());
        let msg = InstantiateMsg {
            client_id: "test-price-feed".into(),
            oracle_script_id: Uint64::from(304u64),
            ask_count: Uint64::from(16u64),
            min_count: Uint64::from(10u64),
            fee_limit: vec![coin(10000, "uband")],
            prepare_gas: Uint64::from(10000u64),
            execute_gas: Uint64::from(250000u64),
            minimum_sources: 3,
            fee: vec![],
        };
        let contract_addr = app
            .instantiate_contract(code_id, owner.clone(), &msg, &vec![], "Payout", None)
            .unwrap();

        // sender funds must be the same
        let sender: Vec<Coin> = query_app(&app, &owner);
        assert_eq!(sender, coins(300000000, "uband"));
        // get contract address, has funds
        let funds = query_app(&app, &contract_addr);
        assert_eq!(funds, vec![]);

        // create empty account
        let random = addr_make("random");
        let funds = query_app(&app, &random);
        assert_eq!(funds, vec![]);

        // do one request
        let res = app
            .execute_contract(
                random.clone(),
                contract_addr.clone(),
                &ExecuteMsg::Request {
                    symbols: vec![
                        "BTC".to_string(),
                        "ETH".to_string(),
                        "ATOM".to_string(),
                        "BAND".to_string(),
                    ],
                },
                &vec![],
            )
            .unwrap();
        assert_eq!(3, res.events.len());

        // the call to payout does emit this as well as custom attributes
        let payout_exec = &res.events[0];
        assert_eq!(payout_exec.ty.as_str(), "execute");
        assert_eq!(payout_exec.attributes, [("_contract_addr", &contract_addr)]);

        // next is a custom wasm event
        let custom_attrs = res.custom_attrs(1);
        assert_eq!(custom_attrs, [("action", "payout")]);
    }
}
