use cosmwasm_std::{
    entry_point, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, Difficulty, ExecuteMsg, GameRecord, HistoryResponse, InstantiateMsg, QueryMsg,
    RiskLevel, StatsResponse,
};
use crate::multipliers::{get_multipliers, get_rows};
use crate::rng::{calculate_bucket_index, generate_ball_path};
use crate::state::{Config, Stats, CONFIG, GAME_HISTORY, PLAYER_GAME_COUNT, STATS};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let plink_token_address = deps.api.addr_validate(&msg.plink_token_address)?;
    let house_address = deps.api.addr_validate(&msg.house_address)?;

    let config = Config {
        plink_token_address,
        house_address,
        admin: info.sender,
    };

    let stats = Stats {
        total_games: 0,
        total_wagered: Uint128::zero(),
        total_won: Uint128::zero(),
        house_balance: Uint128::zero(),
    };

    CONFIG.save(deps.storage, &config)?;
    STATS.save(deps.storage, &stats)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Play {
            difficulty,
            risk_level,
            bet_amount,
        } => execute_play(deps, env, info, difficulty, risk_level, bet_amount),
        ExecuteMsg::UpdateHouse { new_house } => execute_update_house(deps, info, new_house),
        ExecuteMsg::WithdrawHouse { amount } => execute_withdraw_house(deps, info, amount),
        ExecuteMsg::FundHouse { amount } => execute_fund_house(deps, env, info, amount),
    }
}

fn execute_play(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    difficulty: Difficulty,
    risk_level: RiskLevel,
    bet_amount: Uint128,
) -> Result<Response, ContractError> {
    if bet_amount.is_zero() {
        return Err(ContractError::InvalidBetAmount {});
    }

    let config = CONFIG.load(deps.storage)?;
    let mut stats = STATS.load(deps.storage)?;

    // Get player's game count for nonce
    let player_count = PLAYER_GAME_COUNT
        .may_load(deps.storage, &info.sender)?
        .unwrap_or(0);

    // Generate provably fair random path
    let rows = get_rows(&difficulty);
    let path = generate_ball_path(&env, &info, player_count, rows);
    let bucket_index = calculate_bucket_index(&path);

    // Get multiplier for this bucket
    let multipliers = get_multipliers(&difficulty, &risk_level);
    if bucket_index >= multipliers.len() {
        return Err(ContractError::InvalidMultiplierIndex {});
    }

    let (numerator, denominator) = multipliers[bucket_index];

    // Calculate win amount: bet_amount * (numerator / denominator)
    let win_amount = bet_amount
        .checked_mul(Uint128::from(numerator))
        .map_err(|_| ContractError::OverflowError {})?
        .checked_div(Uint128::from(denominator))
        .map_err(|_| ContractError::OverflowError {})?;

    // Update stats
    stats.total_games += 1;
    stats.total_wagered = stats
        .total_wagered
        .checked_add(bet_amount)
        .map_err(|_| ContractError::OverflowError {})?;
    stats.total_won = stats
        .total_won
        .checked_add(win_amount)
        .map_err(|_| ContractError::OverflowError {})?;

    // Update house balance
    // House receives the bet amount
    stats.house_balance = stats.house_balance.checked_add(bet_amount)?;
    
    // House pays out the win amount
    // Use checked_sub to ensure we don't underflow - if this fails, house is insolvent
    stats.house_balance = stats.house_balance.checked_sub(win_amount)?;

    STATS.save(deps.storage, &stats)?;

    // Update player game count
    PLAYER_GAME_COUNT.save(deps.storage, &info.sender, &(player_count + 1))?;

    // Save game record
    let game_record = GameRecord {
        player: info.sender.clone(),
        difficulty: difficulty.clone(),
        risk_level: risk_level.clone(),
        bet_amount,
        multiplier: format!("{}.{}x", numerator / denominator, (numerator % denominator) * 10 / denominator),
        win_amount,
        timestamp: env.block.time.seconds(),
        path: path.clone(),
    };

    GAME_HISTORY.save(deps.storage, (&info.sender, player_count), &game_record)?;

    // Create messages
    let mut messages: Vec<CosmosMsg> = vec![];

    // 1. Transfer bet amount from player to contract
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.plink_token_address.to_string(),
        msg: to_json_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: env.contract.address.to_string(),
            amount: bet_amount,
        })?,
        funds: vec![],
    }));

    // 2. Transfer winnings to player
    if !win_amount.is_zero() {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.plink_token_address.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount: win_amount,
            })?,
            funds: vec![],
        }));
    }

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "play")
        .add_attribute("player", info.sender)
        .add_attribute("bet_amount", bet_amount)
        .add_attribute("win_amount", win_amount)
        .add_attribute("multiplier", format!("{}.{}", numerator / denominator, (numerator % denominator) * 10 / denominator))
        .add_attribute("bucket", bucket_index.to_string()))
}

fn execute_update_house(
    deps: DepsMut,
    info: MessageInfo,
    new_house: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    config.house_address = deps.api.addr_validate(&new_house)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_house")
        .add_attribute("new_house", new_house))
}

fn execute_withdraw_house(
    deps: DepsMut,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut stats = STATS.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if amount > stats.house_balance {
        return Err(ContractError::InsufficientBalance {});
    }

    stats.house_balance = stats.house_balance.checked_sub(amount)?;
    STATS.save(deps.storage, &stats)?;

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.plink_token_address.to_string(),
        msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
            recipient: config.house_address.to_string(),
            amount,
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "withdraw_house")
        .add_attribute("amount", amount))
}

fn execute_fund_house(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut stats = STATS.load(deps.storage)?;

    // Only admin can fund the house
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if amount.is_zero() {
        return Err(ContractError::InvalidBetAmount {});
    }

    // Update house balance
    stats.house_balance = stats.house_balance.checked_add(amount)?;
    STATS.save(deps.storage, &stats)?;

    // Transfer tokens from admin to contract
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.plink_token_address.to_string(),
        msg: to_json_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: env.contract.address.to_string(),
            amount,
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "fund_house")
        .add_attribute("amount", amount))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Stats {} => to_json_binary(&query_stats(deps)?),
        QueryMsg::History { player, limit } => to_json_binary(&query_history(deps, player, limit)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        plink_token_address: config.plink_token_address,
        house_address: config.house_address,
        admin: config.admin,
    })
}

fn query_stats(deps: Deps) -> StdResult<StatsResponse> {
    let stats = STATS.load(deps.storage)?;
    Ok(StatsResponse {
        total_games: stats.total_games,
        total_wagered: stats.total_wagered,
        total_won: stats.total_won,
        house_balance: stats.house_balance,
    })
}

fn query_history(deps: Deps, player: String, limit: Option<u32>) -> StdResult<HistoryResponse> {
    let player_addr = deps.api.addr_validate(&player)?;
    let player_count = PLAYER_GAME_COUNT
        .may_load(deps.storage, &player_addr)?
        .unwrap_or(0);

    let limit = limit.unwrap_or(10).min(100) as u64;
    let start = if player_count > limit {
        player_count - limit
    } else {
        0
    };

    let mut games = Vec::new();
    for i in start..player_count {
        if let Some(game) = GAME_HISTORY.may_load(deps.storage, (&player_addr, i))? {
            games.push(game);
        }
    }

    Ok(HistoryResponse { games })
}
