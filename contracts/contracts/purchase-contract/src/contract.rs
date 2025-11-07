use cosmwasm_std::{
    entry_point, to_json_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, StatsResponse};
use crate::state::{Config, Stats, CONFIG, STATS};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let plink_token_address = deps.api.addr_validate(&msg.plink_token_address)?;
    let treasury_address = deps.api.addr_validate(&msg.treasury_address)?;

    if msg.exchange_rate.is_zero() {
        return Err(ContractError::InvalidExchangeRate {});
    }

    let config = Config {
        plink_token_address,
        treasury_address,
        exchange_rate: msg.exchange_rate,
        admin: info.sender,
    };

    let stats = Stats {
        total_inj_received: Uint128::zero(),
        total_plink_minted: Uint128::zero(),
        total_purchases: 0,
    };

    CONFIG.save(deps.storage, &config)?;
    STATS.save(deps.storage, &stats)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("exchange_rate", msg.exchange_rate))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Purchase {} => execute_purchase(deps, env, info),
        ExecuteMsg::UpdateExchangeRate { new_rate } => {
            execute_update_exchange_rate(deps, info, new_rate)
        }
        ExecuteMsg::UpdateTreasury { new_treasury } => {
            execute_update_treasury(deps, info, new_treasury)
        }
    }
}

fn execute_purchase(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut stats = STATS.load(deps.storage)?;

    // Get INJ amount sent
    let inj_amount = info
        .funds
        .iter()
        .find(|coin| coin.denom == "inj")
        .map(|coin| coin.amount)
        .unwrap_or_else(Uint128::zero);

    if inj_amount.is_zero() {
        return Err(ContractError::NoFundsSent {});
    }

    // Calculate PLINK amount: INJ * exchange_rate
    let plink_amount = inj_amount
        .checked_mul(config.exchange_rate)
        .map_err(|_| ContractError::OverflowError {})?;

    // Update stats
    stats.total_inj_received = stats
        .total_inj_received
        .checked_add(inj_amount)
        .map_err(|_| ContractError::OverflowError {})?;
    stats.total_plink_minted = stats
        .total_plink_minted
        .checked_add(plink_amount)
        .map_err(|_| ContractError::OverflowError {})?;
    stats.total_purchases += 1;
    STATS.save(deps.storage, &stats)?;

    // Create messages
    let mut messages: Vec<CosmosMsg> = vec![];

    // 1. Send INJ to treasury
    messages.push(CosmosMsg::Bank(BankMsg::Send {
        to_address: config.treasury_address.to_string(),
        amount: vec![Coin {
            denom: "inj".to_string(),
            amount: inj_amount,
        }],
    }));

    // 2. Mint PLINK to buyer
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.plink_token_address.to_string(),
        msg: to_json_binary(&Cw20ExecuteMsg::Mint {
            recipient: info.sender.to_string(),
            amount: plink_amount,
        })?,
        funds: vec![],
    }));

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "purchase")
        .add_attribute("buyer", info.sender)
        .add_attribute("inj_amount", inj_amount)
        .add_attribute("plink_amount", plink_amount))
}

fn execute_update_exchange_rate(
    deps: DepsMut,
    info: MessageInfo,
    new_rate: Uint128,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if new_rate.is_zero() {
        return Err(ContractError::InvalidExchangeRate {});
    }

    config.exchange_rate = new_rate;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_exchange_rate")
        .add_attribute("new_rate", new_rate))
}

fn execute_update_treasury(
    deps: DepsMut,
    info: MessageInfo,
    new_treasury: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    config.treasury_address = deps.api.addr_validate(&new_treasury)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_treasury")
        .add_attribute("new_treasury", new_treasury))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Stats {} => to_json_binary(&query_stats(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        plink_token_address: config.plink_token_address,
        treasury_address: config.treasury_address,
        exchange_rate: config.exchange_rate,
        admin: config.admin,
    })
}

fn query_stats(deps: Deps) -> StdResult<StatsResponse> {
    let stats = STATS.load(deps.storage)?;
    Ok(StatsResponse {
        total_inj_received: stats.total_inj_received,
        total_plink_minted: stats.total_plink_minted,
        total_purchases: stats.total_purchases,
    })
}
