use cosmwasm_std::{
    coin, entry_point, to_json_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128,
};

use crate::error::ContractError;
use crate::leaderboard::{should_reset_daily, update_leaderboard};
use crate::msg::{
    ConfigResponse, Difficulty, ExecuteMsg, GameRecord, HistoryResponse, InstantiateMsg,
    LeaderboardEntry as MsgLeaderboardEntry, LeaderboardResponse, LeaderboardType, QueryMsg,
    RiskLevel, StatsResponse, UserStatsResponse,
};
use crate::multipliers::{get_multipliers, get_rows};
use crate::rng::{calculate_bucket_index, generate_ball_path};
use crate::state::{
    Config, DailyLeaderboard, Stats, UserStats, CONFIG, DAILY_LEADERBOARD, DAILY_PLAYER_STATS,
    GAME_HISTORY, GLOBAL_BEST_WINS, GLOBAL_TOTAL_WAGERED, PLAYER_GAME_COUNT, STATS, USER_STATS,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let funder_addr = deps.api.addr_validate(&msg.funder_address)?;

    let config = Config {
        token_denom: msg.token_denom,
        admin: info.sender,
        funder_address: funder_addr,
    };

    let stats = Stats {
        total_games: 0,
        total_wagered: Uint128::zero(),
        total_won: Uint128::zero(),
        house_balance: Uint128::zero(),
    };

    let daily_leaderboard = DailyLeaderboard {
        last_reset: env.block.time.seconds(),
        entries_best_wins: vec![],
        entries_wagered: vec![],
    };

    CONFIG.save(deps.storage, &config)?;
    STATS.save(deps.storage, &stats)?;
    GLOBAL_BEST_WINS.save(deps.storage, &vec![])?;
    GLOBAL_TOTAL_WAGERED.save(deps.storage, &vec![])?;
    DAILY_LEADERBOARD.save(deps.storage, &daily_leaderboard)?;

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
        } => execute_play(deps, env, info, difficulty, risk_level),
        ExecuteMsg::WithdrawHouse { amount } => execute_withdraw_house(deps, info, amount),
        ExecuteMsg::FundHouse {} => execute_fund_house(deps, info),
        ExecuteMsg::SyncBalance {} => execute_sync_balance(deps, env, info),
    }
}

fn execute_play(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    difficulty: Difficulty,
    risk_level: RiskLevel,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Get bet amount from sent funds
    let bet_amount = info
        .funds
        .iter()
        .find(|coin| coin.denom == config.token_denom)
        .map(|coin| coin.amount)
        .unwrap_or(Uint128::zero());

    if bet_amount.is_zero() {
        return Err(ContractError::InvalidBetAmount {});
    }

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

    // Calculate PnL (can be negative, but we store as Uint128 with saturating_sub)
    let pnl = win_amount.saturating_sub(bet_amount);

    // Update global stats
    stats.total_games += 1;
    stats.total_wagered = stats
        .total_wagered
        .checked_add(bet_amount)
        .map_err(|_| ContractError::OverflowError {})?;
    stats.total_won = stats
        .total_won
        .checked_add(win_amount)
        .map_err(|_| ContractError::OverflowError {})?;

    // Update house balance: receives bet, pays out winnings
    let new_house_balance_after_bet = stats.house_balance.checked_add(bet_amount)?;

    if win_amount > new_house_balance_after_bet {
        // The house cannot afford this payout. Return a specific error.
        return Err(ContractError::InsufficientHouseBalance {});
    }

    stats.house_balance = new_house_balance_after_bet.checked_sub(win_amount)?;

    STATS.save(deps.storage, &stats)?;

    // Update user stats
    let mut user_stats = USER_STATS
        .may_load(deps.storage, &info.sender)?
        .unwrap_or(UserStats {
            total_games: 0,
            total_wagered: Uint128::zero(),
            total_won: Uint128::zero(),
            best_win_pnl: Uint128::zero(),
            best_win_multiplier: "0.0x".to_string(),
        });

    user_stats.total_games += 1;
    user_stats.total_wagered = user_stats.total_wagered.checked_add(bet_amount)?;
    user_stats.total_won = user_stats.total_won.checked_add(win_amount)?;

    let multiplier_str = format!(
        "{}.{}x",
        numerator / denominator,
        (numerator % denominator) * 10 / denominator
    );

    if pnl > user_stats.best_win_pnl {
        user_stats.best_win_pnl = pnl;
        user_stats.best_win_multiplier = multiplier_str.clone();
    }

    USER_STATS.save(deps.storage, &info.sender, &user_stats)?;

    // Update global leaderboards
    let mut global_best_wins = GLOBAL_BEST_WINS.load(deps.storage)?;
    update_leaderboard(
        &mut global_best_wins,
        info.sender.clone(),
        user_stats.best_win_pnl,
        Some(user_stats.best_win_multiplier.clone()),
    );
    GLOBAL_BEST_WINS.save(deps.storage, &global_best_wins)?;

    let mut global_wagered = GLOBAL_TOTAL_WAGERED.load(deps.storage)?;
    update_leaderboard(
        &mut global_wagered,
        info.sender.clone(),
        user_stats.total_wagered,
        None,
    );
    GLOBAL_TOTAL_WAGERED.save(deps.storage, &global_wagered)?;

    // Update daily leaderboard (with reset check)
    let mut daily = DAILY_LEADERBOARD.load(deps.storage)?;

    // Check if the daily cycle needs to be reset.
    if should_reset_daily(daily.last_reset, env.block.time.seconds()) {
        daily.last_reset = env.block.time.seconds();
        daily.entries_best_wins = vec![];
        daily.entries_wagered = vec![];
        // NOTE: We don't need to manually clear DAILY_PLAYER_STATS.
        // The old data becomes irrelevant, and players will create new entries
        // on their first play of the new day.
    }

    // Load the player's current daily stats, or create new ones if it's their first game of the day.
    let mut player_daily_stats = DAILY_PLAYER_STATS
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_default();

    // Update the player's cumulative daily wagered amount.
    player_daily_stats.total_wagered = player_daily_stats.total_wagered.checked_add(bet_amount)?;

    // Check if the current game's PNL is their best for the day.
    if pnl > player_daily_stats.best_win_pnl {
        player_daily_stats.best_win_pnl = pnl;
        player_daily_stats.best_win_multiplier = multiplier_str.clone();
    }

    // Save the updated daily stats for the player.
    DAILY_PLAYER_STATS.save(deps.storage, &info.sender, &player_daily_stats)?;

    // Now, update the main daily leaderboards using the player's cumulative daily stats.
    update_leaderboard(
        &mut daily.entries_best_wins,
        info.sender.clone(),
        player_daily_stats.best_win_pnl, // Use cumulative best PNL for the day
        Some(player_daily_stats.best_win_multiplier),
    );
    update_leaderboard(
        &mut daily.entries_wagered,
        info.sender.clone(),
        player_daily_stats.total_wagered, // Use cumulative wagered for the day
        None,
    );

    DAILY_LEADERBOARD.save(deps.storage, &daily)?;

    // Update player game count
    PLAYER_GAME_COUNT.save(deps.storage, &info.sender, &(player_count + 1))?;

    // Convert Vec<u8> path to Vec<bool> for storage
    let path_bool: Vec<bool> = path.iter().map(|&b| b != 0).collect();

    // Save game record
    let game_record = GameRecord {
        player: info.sender.clone(),
        difficulty: difficulty.clone(),
        risk_level: risk_level.clone(),
        bet_amount,
        multiplier: multiplier_str.clone(),
        win_amount,
        pnl,
        timestamp: env.block.time.seconds(),
        path: path_bool.clone(),
    };

    GAME_HISTORY.save(deps.storage, (&info.sender, player_count), &game_record)?;

    // Create bank message to send winnings
    let mut messages = vec![];
    if !win_amount.is_zero() {
        messages.push(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![coin(win_amount.u128(), config.token_denom)],
        });
    }

    let path_str: String = path_bool
        .iter()
        .map(|&b| if b { '1' } else { '0' })
        .collect();

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "play")
        .add_attribute("player", info.sender)
        .add_attribute("bet_amount", bet_amount)
        .add_attribute("win_amount", win_amount)
        .add_attribute("pnl", pnl)
        .add_attribute("multiplier", multiplier_str)
        .add_attribute("bucket", bucket_index.to_string())
        .add_attribute("path", path_str))
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

    let msg = BankMsg::Send {
        to_address: config.admin.to_string(),
        amount: vec![coin(amount.u128(), config.token_denom)],
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "withdraw_house")
        .add_attribute("amount", amount))
}

fn execute_fund_house(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut stats = STATS.load(deps.storage)?;

    // Only the admin can fund the house
    if info.sender != config.funder_address {
        return Err(ContractError::Unauthorized {});
    }

    // Find the amount of the game's native token that was sent with this message
    let amount = info
        .funds
        .iter()
        .find(|c| c.denom == config.token_denom)
        .map(|c| c.amount)
        .unwrap_or_else(Uint128::zero);

    if amount.is_zero() {
        return Err(ContractError::NoFundsSent {}); // Or a more specific error
    }

    // Update the internal house balance state
    stats.house_balance = stats.house_balance.checked_add(amount)?;
    STATS.save(deps.storage, &stats)?;

    Ok(Response::new()
        .add_attribute("action", "fund_house")
        .add_attribute("amount", amount))
}

fn execute_sync_balance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut stats = STATS.load(deps.storage)?;

    // Only the admin can perform this action
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    // Use the querier to get the contract's ACTUAL on-chain balance of its native token
    let actual_balance = deps
        .querier
        .query_balance(env.contract.address, config.token_denom)?;

    // Check if the actual balance is greater than what our internal ledger says
    if actual_balance.amount > stats.house_balance {
        let surplus = actual_balance.amount.checked_sub(stats.house_balance)?;

        // Update the internal ledger to match reality
        stats.house_balance = actual_balance.amount;
        STATS.save(deps.storage, &stats)?;

        Ok(Response::new()
            .add_attribute("action", "sync_balance")
            .add_attribute("funds_recovered", surplus)
            .add_attribute("new_house_balance", stats.house_balance))
    } else {
        // If there's no surplus, there's nothing to do.
        Ok(Response::new()
            .add_attribute("action", "sync_balance")
            .add_attribute("funds_recovered", "0")
            .add_attribute("message", "No surplus funds detected."))
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Stats {} => to_json_binary(&query_stats(deps)?),
        QueryMsg::History { player, limit } => to_json_binary(&query_history(deps, player, limit)?),
        QueryMsg::UserStats { player } => to_json_binary(&query_user_stats(deps, player)?),
        QueryMsg::GlobalLeaderboard {
            leaderboard_type,
            limit,
        } => to_json_binary(&query_global_leaderboard(deps, leaderboard_type, limit)?),
        QueryMsg::DailyLeaderboard {
            leaderboard_type,
            limit,
        } => to_json_binary(&query_daily_leaderboard(
            deps,
            env,
            leaderboard_type,
            limit,
        )?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        token_denom: config.token_denom,
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
    let start = player_count.saturating_sub(limit);

    let mut games = Vec::new();
    for i in start..player_count {
        if let Some(game) = GAME_HISTORY.may_load(deps.storage, (&player_addr, i))? {
            games.push(game);
        }
    }

    Ok(HistoryResponse { games })
}

fn query_user_stats(deps: Deps, player: String) -> StdResult<UserStatsResponse> {
    let player_addr = deps.api.addr_validate(&player)?;
    let user_stats = USER_STATS
        .may_load(deps.storage, &player_addr)?
        .unwrap_or(UserStats {
            total_games: 0,
            total_wagered: Uint128::zero(),
            total_won: Uint128::zero(),
            best_win_pnl: Uint128::zero(),
            best_win_multiplier: "0.0x".to_string(),
        });

    Ok(UserStatsResponse {
        player: player_addr,
        total_games: user_stats.total_games,
        total_wagered: user_stats.total_wagered,
        total_won: user_stats.total_won,
        best_win_pnl: user_stats.best_win_pnl,
        best_win_multiplier: user_stats.best_win_multiplier,
    })
}

fn query_global_leaderboard(
    deps: Deps,
    leaderboard_type: LeaderboardType,
    limit: Option<u32>,
) -> StdResult<LeaderboardResponse> {
    let limit = limit.unwrap_or(10).min(100) as usize;

    let entries = match leaderboard_type {
        LeaderboardType::BestWins => GLOBAL_BEST_WINS.load(deps.storage)?,
        LeaderboardType::TotalWagered => GLOBAL_TOTAL_WAGERED.load(deps.storage)?,
    };

    let limited_entries: Vec<MsgLeaderboardEntry> = entries
        .into_iter()
        .take(limit)
        .map(|e| MsgLeaderboardEntry {
            player: e.player,
            value: e.value,
            multiplier: e.multiplier,
        })
        .collect();

    Ok(LeaderboardResponse {
        entries: limited_entries,
        leaderboard_type,
    })
}

fn query_daily_leaderboard(
    deps: Deps,
    env: Env,
    leaderboard_type: LeaderboardType,
    limit: Option<u32>,
) -> StdResult<LeaderboardResponse> {
    let limit = limit.unwrap_or(10).min(100) as usize;
    let daily = DAILY_LEADERBOARD.load(deps.storage)?;

    // Check if reset is needed (for query consistency)
    let entries = if should_reset_daily(daily.last_reset, env.block.time.seconds()) {
        vec![] // Return empty if reset is due
    } else {
        match leaderboard_type {
            LeaderboardType::BestWins => daily.entries_best_wins,
            LeaderboardType::TotalWagered => daily.entries_wagered,
        }
    };

    let limited_entries: Vec<MsgLeaderboardEntry> = entries
        .into_iter()
        .take(limit)
        .map(|e| MsgLeaderboardEntry {
            player: e.player,
            value: e.value,
            multiplier: e.multiplier,
        })
        .collect();

    Ok(LeaderboardResponse {
        entries: limited_entries,
        leaderboard_type,
    })
}
