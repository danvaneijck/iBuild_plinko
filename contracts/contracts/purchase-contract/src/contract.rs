use cosmwasm_std::{
    entry_point, to_json_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128, WasmMsg,
};
use injective_cosmwasm::msg::{
    create_mint_tokens_msg, create_new_denom_msg, create_set_token_metadata_msg,
};
use injective_cosmwasm::InjectiveMsgWrapper;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, PreviewPurchaseResponse, QueryMsg, StatsResponse,
};
use crate::state::{Config, Stats, CONFIG, STATS};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
    let treasury_address = deps.api.addr_validate(&msg.treasury_address)?;

    if msg.exchange_rate.is_zero() {
        return Err(ContractError::InvalidExchangeRate {});
    }

    // Create the token denom: factory/{contract_address}/{subdenom}
    let token_denom = format!("factory/{}/{}", env.contract.address, msg.subdenom);

    let config = Config {
        token_denom: token_denom.clone(),
        token_name: msg.token_name.clone(),
        token_symbol: msg.token_symbol.clone(),
        token_decimals: msg.token_decimals,
        treasury_address,
        exchange_rate: msg.exchange_rate,
        admin: info.sender,
    };

    let stats = Stats {
        total_inj_received: Uint128::zero(),
        total_tokens_minted: Uint128::zero(),
        total_purchases: 0,
        total_house_funding: Uint128::zero(),
    };

    CONFIG.save(deps.storage, &config)?;
    STATS.save(deps.storage, &stats)?;

    // Create the new denom
    let create_denom_msg = create_new_denom_msg(env.contract.address.to_string(), msg.subdenom);

    // Set token metadata
    let metadata_msg = create_set_token_metadata_msg(
        token_denom.clone(),
        msg.token_name,
        msg.token_symbol,
        msg.token_decimals,
    );

    Ok(Response::new()
        .add_message(create_denom_msg)
        .add_message(metadata_msg)
        .add_attribute("action", "instantiate")
        .add_attribute("token_denom", token_denom)
        .add_attribute("exchange_rate", msg.exchange_rate))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
    match msg {
        ExecuteMsg::Purchase {} => execute_purchase(deps, env, info),
        ExecuteMsg::FundHouse {
            game_contract,
            amount,
        } => execute_fund_house(deps, env, info, game_contract, amount),
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
    env: Env,
    info: MessageInfo,
) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
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

    // Calculate token amount: INJ * exchange_rate
    let token_amount = inj_amount
        .checked_mul(config.exchange_rate)
        .map_err(|_| ContractError::OverflowError {})?;

    // Update stats
    stats.total_inj_received = stats
        .total_inj_received
        .checked_add(inj_amount)
        .map_err(|_| ContractError::OverflowError {})?;
    stats.total_tokens_minted = stats
        .total_tokens_minted
        .checked_add(token_amount)
        .map_err(|_| ContractError::OverflowError {})?;
    stats.total_purchases += 1;
    STATS.save(deps.storage, &stats)?;

    // Create mint message for the tokens
    let mint_msg = create_mint_tokens_msg(
        env.contract.address,
        Coin {
            denom: config.token_denom.clone(),
            amount: token_amount,
        },
        info.sender.to_string(),
    );

    // Send INJ to treasury
    let send_inj_msg: CosmosMsg<InjectiveMsgWrapper> = CosmosMsg::Bank(BankMsg::Send {
        to_address: config.treasury_address.to_string(),
        amount: vec![Coin {
            denom: "inj".to_string(),
            amount: inj_amount,
        }],
    });

    Ok(Response::new()
        .add_message(mint_msg)
        .add_message(send_inj_msg)
        .add_attribute("action", "purchase")
        .add_attribute("buyer", info.sender)
        .add_attribute("inj_amount", inj_amount)
        .add_attribute("token_amount", token_amount))
}

fn execute_fund_house(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    game_contract: String,
    amount: Uint128,
) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut stats = STATS.load(deps.storage)?;

    // Only admin can fund the house
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if amount.is_zero() {
        return Err(ContractError::InvalidAmount {});
    }

    // Validate game contract address
    let game_contract_addr = deps.api.addr_validate(&game_contract)?;

    // Update stats
    stats.total_tokens_minted = stats
        .total_tokens_minted
        .checked_add(amount)
        .map_err(|_| ContractError::OverflowError {})?;
    stats.total_house_funding = stats
        .total_house_funding
        .checked_add(amount)
        .map_err(|_| ContractError::OverflowError {})?;
    STATS.save(deps.storage, &stats)?;

    let mint_msg = create_mint_tokens_msg(
        env.contract.address.clone(),
        Coin {
            denom: config.token_denom.clone(),
            amount,
        },
        env.contract.address.to_string(),
    );

    let execute_msg = WasmMsg::Execute {
        contract_addr: game_contract_addr.to_string(),
        msg: to_json_binary(&serde_json::json!({"fund_house": {}}))?,

        funds: vec![Coin {
            denom: config.token_denom.clone(),
            amount,
        }],
    };

    Ok(Response::new()
        .add_message(mint_msg)
        .add_message(execute_msg)
        .add_attribute("action", "fund_house")
        .add_attribute("game_contract", game_contract)
        .add_attribute("amount", amount))
}

fn execute_update_exchange_rate(
    deps: DepsMut,
    info: MessageInfo,
    new_rate: Uint128,
) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
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
) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
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
        QueryMsg::PreviewPurchase { inj_amount } => {
            to_json_binary(&query_preview_purchase(deps, inj_amount)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        token_denom: config.token_denom,
        token_name: config.token_name,
        token_symbol: config.token_symbol,
        token_decimals: config.token_decimals,
        treasury_address: config.treasury_address,
        exchange_rate: config.exchange_rate,
        admin: config.admin,
    })
}

fn query_stats(deps: Deps) -> StdResult<StatsResponse> {
    let stats = STATS.load(deps.storage)?;
    Ok(StatsResponse {
        total_inj_received: stats.total_inj_received,
        total_tokens_minted: stats.total_tokens_minted,
        total_purchases: stats.total_purchases,
        total_house_funding: stats.total_house_funding,
    })
}

fn query_preview_purchase(deps: Deps, inj_amount: Uint128) -> StdResult<PreviewPurchaseResponse> {
    let config = CONFIG.load(deps.storage)?;

    let token_amount = inj_amount
        .checked_mul(config.exchange_rate)
        .unwrap_or(Uint128::zero());

    Ok(PreviewPurchaseResponse {
        inj_amount,
        token_amount,
        exchange_rate: config.exchange_rate,
    })
}
