use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128, CosmosMsg, WasmMsg, BankMsg, Coin,
};
use cw20::Cw20ExecuteMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ConfigResponse};
use crate::state::{Config, CONFIG};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        plink_token: deps.api.addr_validate(&msg.plink_token)?,
        treasury: deps.api.addr_validate(&msg.treasury)?,
        exchange_rate: msg.exchange_rate,
    };
    
    CONFIG.save(deps.storage, &config)?;
    
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("plink_token", msg.plink_token)
        .add_attribute("treasury", msg.treasury)
        .add_attribute("exchange_rate", msg.exchange_rate.to_string()))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Purchase {} => execute_purchase(deps, info),
    }
}

fn execute_purchase(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    // Get INJ amount sent
    let inj_amount = info
        .funds
        .iter()
        .find(|c| c.denom == "inj")
        .map(|c| c.amount)
        .unwrap_or_else(Uint128::zero);
    
    if inj_amount.is_zero() {
        return Err(ContractError::NoFundsSent {});
    }
    
    // Calculate PLINK amount
    let plink_amount = inj_amount.checked_mul(config.exchange_rate)?;
    
    // Create messages
    let mut messages: Vec<CosmosMsg> = vec![];
    
    // Send INJ to treasury
    messages.push(CosmosMsg::Bank(BankMsg::Send {
        to_address: config.treasury.to_string(),
        amount: vec![Coin {
            denom: "inj".to_string(),
            amount: inj_amount,
        }],
    }));
    
    // Mint PLINK to buyer
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.plink_token.to_string(),
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

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        plink_token: config.plink_token.to_string(),
        treasury: config.treasury.to_string(),
        exchange_rate: config.exchange_rate,
    })
}
