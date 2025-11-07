use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128, CosmosMsg, WasmMsg, Addr,
};
use cw20::Cw20ExecuteMsg;
use sha2::{Sha256, Digest};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ConfigResponse, GameHistoryResponse, GameResult};
use crate::state::{Config, GameHistory, CONFIG, GAME_HISTORY, NONCE};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        plink_token: deps.api.addr_validate(&msg.plink_token)?,
        admin: info.sender.clone(),
        house_balance: Uint128::zero(),
    };
    
    CONFIG.save(deps.storage, &config)?;
    NONCE.save(deps.storage, &0u64)?;
    
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", info.sender)
        .add_attribute("plink_token", msg.plink_token))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::PlayGame { bet_amount, difficulty, risk } => {
            execute_play_game(deps, env, info, bet_amount, difficulty, risk)
        }
        ExecuteMsg::FundHouse { amount } => execute_fund_house(deps, info, amount),
    }
}

fn execute_fund_house(
    deps: DepsMut,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    
    // Only admin can fund house
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    
    // Update house balance
    config.house_balance = config.house_balance.checked_add(amount)?;
    CONFIG.save(deps.storage, &config)?;
    
    // Transfer PLINK from admin to contract
    let transfer_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.plink_token.to_string(),
        msg: to_json_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: env.contract.address.to_string(),
            amount,
        })?,
        funds: vec![],
    });
    
    Ok(Response::new()
        .add_message(transfer_msg)
        .add_attribute("action", "fund_house")
        .add_attribute("amount", amount)
        .add_attribute("new_balance", config.house_balance))
}

fn execute_play_game(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bet_amount: Uint128,
    difficulty: u8,
    risk: u8,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    
    // Validate difficulty (8, 12, or 16 rows)
    if difficulty != 8 && difficulty != 12 && difficulty != 16 {
        return Err(ContractError::InvalidDifficulty {});
    }
    
    // Validate risk (0=Low, 1=Medium, 2=High)
    if risk > 2 {
        return Err(ContractError::InvalidRisk {});
    }
    
    // Generate provably fair random path
    let mut nonce = NONCE.load(deps.storage)?;
    nonce += 1;
    NONCE.save(deps.storage, &nonce)?;
    
    let path = generate_path(
        env.block.height,
        env.block.time.seconds(),
        &info.sender,
        nonce,
        difficulty,
    );
    
    // Calculate multiplier
    let multiplier = calculate_multiplier(&path, difficulty, risk);
    
    // Calculate payout
    let payout = bet_amount.checked_mul(Uint128::from(multiplier.0))?
        .checked_div(Uint128::from(multiplier.1))?;
    
    // Check house has enough balance
    if payout > config.house_balance {
        return Err(ContractError::InsufficientHouseBalance {});
    }
    
    // Update house balance
    config.house_balance = config.house_balance.checked_sub(payout)?;
    config.house_balance = config.house_balance.checked_add(bet_amount)?;
    CONFIG.save(deps.storage, &config)?;
    
    // Save game history
    let game_result = GameResult {
        player: info.sender.clone(),
        bet_amount,
        payout,
        multiplier: (multiplier.0, multiplier.1),
        path: path.clone(),
        difficulty,
        risk,
        timestamp: env.block.time.seconds(),
    };
    
    let history_key = (info.sender.as_bytes(), env.block.height);
    GAME_HISTORY.save(deps.storage, history_key, &game_result)?;
    
    // Create messages
    let mut messages: Vec<CosmosMsg> = vec![];
    
    // Transfer bet from player to contract
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.plink_token.to_string(),
        msg: to_json_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: env.contract.address.to_string(),
            amount: bet_amount,
        })?,
        funds: vec![],
    }));
    
    // Transfer payout to player
    if !payout.is_zero() {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.plink_token.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount: payout,
            })?,
            funds: vec![],
        }));
    }
    
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "play_game")
        .add_attribute("player", info.sender)
        .add_attribute("bet_amount", bet_amount)
        .add_attribute("payout", payout)
        .add_attribute("multiplier", format!("{}/{}", multiplier.0, multiplier.1))
        .add_attribute("house_balance", config.house_balance))
}

fn generate_path(
    block_height: u64,
    timestamp: u64,
    player: &Addr,
    nonce: u64,
    rows: u8,
) -> Vec<bool> {
    let mut hasher = Sha256::new();
    hasher.update(block_height.to_be_bytes());
    hasher.update(timestamp.to_be_bytes());
    hasher.update(player.as_bytes());
    hasher.update(nonce.to_be_bytes());
    let hash = hasher.finalize();
    
    let mut path = Vec::new();
    for i in 0..rows {
        let byte_index = (i / 8) as usize;
        let bit_index = i % 8;
        let bit = (hash[byte_index] >> bit_index) & 1;
        path.push(bit == 1);
    }
    
    path
}

fn calculate_multiplier(path: &[bool], difficulty: u8, risk: u8) -> (u128, u128) {
    let final_position = path.iter().filter(|&&x| x).count() as i32;
    
    // Multiplier tables (numerator, denominator)
    match (difficulty, risk) {
        // Easy (8 rows)
        (8, 0) => match final_position { // Low risk
            0 | 8 => (5, 10),
            1 | 7 => (2, 1),
            2 | 6 => (15, 10),
            3 | 5 => (12, 10),
            4 => (10, 10),
            _ => (0, 1),
        },
        (8, 1) => match final_position { // Medium risk
            0 | 8 => (13, 1),
            1 | 7 => (3, 1),
            2 | 6 => (14, 10),
            3 | 5 => (12, 10),
            4 => (10, 10),
            _ => (0, 1),
        },
        (8, 2) => match final_position { // High risk
            0 | 8 => (29, 1),
            1 | 7 => (4, 1),
            2 | 6 => (15, 10),
            3 | 5 => (3, 10),
            4 => (2, 10),
            _ => (0, 1),
        },
        
        // Medium (12 rows)
        (12, 0) => match final_position { // Low risk
            0 | 12 => (8, 1),
            1 | 11 => (3, 1),
            2 | 10 => (16, 10),
            3 | 9 => (13, 10),
            4 | 8 => (11, 10),
            5 | 7 => (10, 10),
            6 => (10, 10),
            _ => (0, 1),
        },
        (12, 1) => match final_position { // Medium risk
            0 | 12 => (18, 1),
            1 | 11 => (4, 1),
            2 | 10 => (15, 10),
            3 | 9 => (12, 10),
            4 | 8 => (11, 10),
            5 | 7 => (10, 10),
            6 => (9, 10),
            _ => (0, 1),
        },
        (12, 2) => match final_position { // High risk
            0 | 12 => (110, 1),
            1 | 11 => (41, 1),
            2 | 10 => (10, 1),
            3 | 9 => (5, 1),
            4 | 8 => (3, 1),
            5 | 7 => (15, 10),
            6 => (10, 10),
            _ => (0, 1),
        },
        
        // Hard (16 rows)
        (16, 0) => match final_position { // Low risk
            0 | 16 => (16, 1),
            1 | 15 => (9, 1),
            2 | 14 => (2, 1),
            3 | 13 => (14, 10),
            4 | 12 => (12, 10),
            5 | 11 => (11, 10),
            6 | 10 => (10, 10),
            7 | 9 => (5, 10),
            8 => (3, 10),
            _ => (0, 1),
        },
        (16, 1) => match final_position { // Medium risk
            0 | 16 => (33, 1),
            1 | 15 => (11, 1),
            2 | 14 => (4, 1),
            3 | 13 => (2, 1),
            4 | 12 => (15, 10),
            5 | 11 => (12, 10),
            6 | 10 => (11, 10),
            7 | 9 => (10, 10),
            8 => (8, 10),
            _ => (0, 1),
        },
        (16, 2) => match final_position { // High risk
            0 | 16 => (1000, 1), // 1000x max multiplier!
            1 | 15 => (130, 1),
            2 | 14 => (26, 1),
            3 | 13 => (9, 1),
            4 | 12 => (4, 1),
            5 | 11 => (2, 1),
            6 | 10 => (2, 10),
            7 | 9 => (15, 100),
            8 => (10, 100),
            _ => (0, 1),
        },
        
        _ => (0, 1),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::GameHistory { player, limit } => {
            to_json_binary(&query_game_history(deps, player, limit)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        plink_token: config.plink_token.to_string(),
        admin: config.admin.to_string(),
        house_balance: config.house_balance,
    })
}

fn query_game_history(
    deps: Deps,
    player: String,
    limit: Option<u32>,
) -> StdResult<GameHistoryResponse> {
    let player_addr = deps.api.addr_validate(&player)?;
    let limit = limit.unwrap_or(10).min(100) as usize;
    
    let games: Vec<GameResult> = GAME_HISTORY
        .prefix(player_addr.as_bytes())
        .range(deps.storage, None, None, cosmwasm_std::Order::Descending)
        .take(limit)
        .map(|item| item.map(|(_, game)| game))
        .collect::<StdResult<Vec<_>>>()?;
    
    Ok(GameHistoryResponse { games })
}
