use cosmwasm_std::{
    coin, entry_point, to_json_binary, Addr, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128,
};

use crate::error::ContractError;
use crate::leaderboard::{should_reset_daily, update_leaderboard};
use crate::msg::{
    ConfigResponse, Difficulty, ExecuteMsg, GameRecord, HistoryResponse, InstantiateMsg,
    LeaderboardEntry as MsgLeaderboardEntry, LeaderboardResponse, LeaderboardType, QueryMsg,
    RiskLevel, StatsResponse, UserStatsResponse, WinnablePrizeResponse,
};
use crate::multipliers::{get_multipliers, get_rows};
use crate::rng::{calculate_bucket_index, generate_ball_path};
use crate::state::{
    Config, DailyLeaderboard, DailyPlayerStats, DailyStats, PrizePool, Stats, UserStats,
    CLAIMED_PRIZES, CONFIG, DAILY_LEADERBOARD, DAILY_PLAYER_STATS, DAILY_STATS, GAME_HISTORY,
    GLOBAL_BEST_WINS, GLOBAL_TOTAL_WAGERED, PLAYER_GAME_COUNT, PRIZE_POOL, STATS, USER_STATS,
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
        prize_pool_percentage: msg.prize_pool_percentage,
        claim_period_seconds: msg.claim_period_seconds,
        prize_leaderboard_type: msg.prize_leaderboard_type,
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
    DAILY_STATS.save(deps.storage, &DailyStats::default())?;

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
        ExecuteMsg::ClaimDailyPrize { day_index } => {
            execute_claim_daily_prize(deps, env, info, day_index)
        }
        ExecuteMsg::UpdatePrizeConfig {
            prize_pool_percentage,
            claim_period_seconds,
            prize_leaderboard_type,
        } => execute_update_prize_config(
            deps,
            info,
            prize_pool_percentage,
            claim_period_seconds,
            prize_leaderboard_type,
        ),
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

    // --- START: DAILY RESET & PRIZE SNAPSHOT LOGIC ---
    let mut daily = DAILY_LEADERBOARD.load(deps.storage)?;
    let current_day_index = env.block.time.seconds() / 86_400;
    let last_reset_day_index = daily.last_reset / 86_400;

    // is_new_day check is now based on the day index comparison
    let is_new_day = current_day_index > last_reset_day_index;

    if is_new_day {
        // It's a new day. Settle yesterday's results before proceeding.
        let yesterday_stats = DAILY_STATS.load(deps.storage)?;

        // Only create a prize pool if the house was profitable yesterday
        if yesterday_stats.total_wagered > yesterday_stats.total_won {
            let house_profit = yesterday_stats
                .total_wagered
                .saturating_sub(yesterday_stats.total_won);

            let prize_pool_amount = house_profit.multiply_ratio(
                Uint128::from(config.prize_pool_percentage as u128),
                Uint128::new(100),
            );

            if !prize_pool_amount.is_zero() {
                // Get winners from the configured leaderboard type
                let winners_list = match config.prize_leaderboard_type {
                    LeaderboardType::BestWins => &daily.entries_best_wins,
                    LeaderboardType::TotalWagered => &daily.entries_wagered,
                };

                let winners: Vec<Addr> = winners_list
                    .iter()
                    .take(3)
                    .map(|entry| entry.player.clone())
                    .collect();

                if !winners.is_empty() {
                    let prize_pool = PrizePool {
                        total_prize_amount: prize_pool_amount,
                        winners,
                        claim_deadline: env.block.time.seconds() + config.claim_period_seconds,
                    };
                    // Save the prize pool snapshot for yesterday
                    PRIZE_POOL.save(deps.storage, last_reset_day_index, &prize_pool)?;
                }
            }
        }

        // Now, reset everything for the new day
        daily.last_reset = env.block.time.seconds();
        daily.entries_best_wins = vec![];
        daily.entries_wagered = vec![];
        DAILY_STATS.save(deps.storage, &DailyStats::default())?;
    }
    // --- END: DAILY RESET & PRIZE SNAPSHOT LOGIC ---

    // --- Core Game Logic (Unchanged) ---
    let mut stats = STATS.load(deps.storage)?;
    let player_count = PLAYER_GAME_COUNT
        .may_load(deps.storage, &info.sender)?
        .unwrap_or(0);
    let rows = get_rows(&difficulty);
    let path = generate_ball_path(&env, &info, player_count, rows);
    let bucket_index = calculate_bucket_index(&path);
    let multipliers = get_multipliers(&difficulty, &risk_level);
    if bucket_index >= multipliers.len() {
        return Err(ContractError::InvalidMultiplierIndex {});
    }
    let (numerator, denominator) = multipliers[bucket_index];
    let win_amount = bet_amount
        .checked_mul(Uint128::from(numerator))
        .map_err(|_| ContractError::OverflowError {})?
        .checked_div(Uint128::from(denominator))
        .map_err(|_| ContractError::OverflowError {})?;
    let pnl = win_amount.saturating_sub(bet_amount);

    // --- Update Global & House Stats (Unchanged) ---
    stats.total_games += 1;
    stats.total_wagered = stats.total_wagered.checked_add(bet_amount)?;
    stats.total_won = stats.total_won.checked_add(win_amount)?;
    let new_house_balance_after_bet = stats.house_balance.checked_add(bet_amount)?;
    if win_amount > new_house_balance_after_bet {
        return Err(ContractError::InsufficientHouseBalance {});
    }
    stats.house_balance = new_house_balance_after_bet.checked_sub(win_amount)?;
    STATS.save(deps.storage, &stats)?;

    // --- START: UPDATE CURRENT DAILY STATS ---
    let mut daily_stats = DAILY_STATS.load(deps.storage)?;
    daily_stats.total_wagered = daily_stats.total_wagered.checked_add(bet_amount)?;
    daily_stats.total_won = daily_stats.total_won.checked_add(win_amount)?;
    DAILY_STATS.save(deps.storage, &daily_stats)?;
    // --- END: UPDATE CURRENT DAILY STATS ---

    // --- Update User Stats (Unchanged) ---
    let mut user_stats = USER_STATS
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_default();
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

    // --- Update Global Leaderboards (Unchanged) ---
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

    // --- Update Daily Leaderboards (Logic refined, reset handled above) ---
    let mut player_daily_stats = if is_new_day {
        DailyPlayerStats::default()
    } else {
        DAILY_PLAYER_STATS
            .may_load(deps.storage, &info.sender)?
            .unwrap_or_default()
    };
    player_daily_stats.total_wagered = player_daily_stats.total_wagered.checked_add(bet_amount)?;
    if pnl > player_daily_stats.best_win_pnl {
        player_daily_stats.best_win_pnl = pnl;
        player_daily_stats.best_win_multiplier = multiplier_str.clone();
    }
    DAILY_PLAYER_STATS.save(deps.storage, &info.sender, &player_daily_stats)?;
    update_leaderboard(
        &mut daily.entries_best_wins,
        info.sender.clone(),
        player_daily_stats.best_win_pnl,
        Some(player_daily_stats.best_win_multiplier.clone()),
    );
    update_leaderboard(
        &mut daily.entries_wagered,
        info.sender.clone(),
        player_daily_stats.total_wagered,
        None,
    );
    DAILY_LEADERBOARD.save(deps.storage, &daily)?;

    // --- Finalize and Save Game History (Unchanged) ---
    PLAYER_GAME_COUNT.save(deps.storage, &info.sender, &(player_count + 1))?;
    let path_bool: Vec<bool> = path.iter().map(|&b| b != 0).collect();
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

    // --- Create Bank Message and Response (Unchanged) ---
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

fn execute_claim_daily_prize(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    day_index: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if CLAIMED_PRIZES
        .may_load(deps.storage, (&info.sender, day_index))?
        .unwrap_or(false)
    {
        return Err(ContractError::PrizeAlreadyClaimed {});
    }

    let prize_pool = match PRIZE_POOL.may_load(deps.storage, day_index)? {
        Some(pool) => pool,
        None => return Err(ContractError::NoPrizeToClaim {}),
    };

    if env.block.time.seconds() > prize_pool.claim_deadline {
        return Err(ContractError::ClaimPeriodExpired {});
    }

    let rank = prize_pool
        .winners
        .iter()
        .position(|addr| addr == &info.sender);

    if let Some(rank_index) = rank {
        // Prize distribution: 50% for 1st, 30% for 2nd, 20% for 3rd
        let prize_amount = match rank_index {
            0 => prize_pool
                .total_prize_amount
                .multiply_ratio(Uint128::new(50), Uint128::new(100)),
            1 => prize_pool
                .total_prize_amount
                .multiply_ratio(Uint128::new(30), Uint128::new(100)),
            2 => prize_pool
                .total_prize_amount
                .multiply_ratio(Uint128::new(20), Uint128::new(100)),
            _ => Uint128::zero(),
        };

        if prize_amount.is_zero() {
            return Err(ContractError::NoPrizeToClaim {});
        }

        CLAIMED_PRIZES.save(deps.storage, (&info.sender, day_index), &true)?;

        let send_msg = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![coin(prize_amount.u128(), config.token_denom)],
        };

        Ok(Response::new()
            .add_message(send_msg)
            .add_attribute("action", "claim_daily_prize")
            .add_attribute("player", info.sender)
            .add_attribute("amount", prize_amount)
            .add_attribute("day_claimed", day_index.to_string()))
    } else {
        Err(ContractError::NotAWinner {})
    }
}

fn execute_update_prize_config(
    deps: DepsMut,
    info: MessageInfo,
    prize_pool_percentage: Option<u8>,
    claim_period_seconds: Option<u64>,
    prize_leaderboard_type: Option<LeaderboardType>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    let mut res = Response::new().add_attribute("action", "update_prize_config");

    if let Some(percentage) = prize_pool_percentage {
        if percentage > 100 {
            return Err(ContractError::InvalidPercentage {});
        }
        config.prize_pool_percentage = percentage;
        res = res.add_attribute("prize_pool_percentage", percentage.to_string());
    }

    if let Some(period) = claim_period_seconds {
        config.claim_period_seconds = period;
        res = res.add_attribute("claim_period_seconds", period.to_string());
    }

    if let Some(lb_type) = prize_leaderboard_type {
        config.prize_leaderboard_type = lb_type.clone();
        res = res.add_attribute("prize_leaderboard_type", format!("{:?}", lb_type));
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(res)
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
        QueryMsg::WinnablePrize { player, day_index } => {
            to_json_binary(&query_winnable_prize(deps, env, player, day_index)?)
        }
    }
}

fn query_winnable_prize(
    deps: Deps,
    env: Env,
    player: String,
    day_index: u64,
) -> StdResult<WinnablePrizeResponse> {
    let player_addr = deps.api.addr_validate(&player)?;
    let has_claimed = CLAIMED_PRIZES
        .may_load(deps.storage, (&player_addr, day_index))?
        .unwrap_or(false);

    let prize_pool = PRIZE_POOL.may_load(deps.storage, day_index)?;

    if prize_pool.is_none() {
        return Ok(WinnablePrizeResponse {
            is_winner: false,
            prize_amount: Uint128::zero(),
            has_claimed,
            claim_expired: false,
            rank: 0,
        });
    }

    let pool = prize_pool.unwrap();
    let claim_expired = env.block.time.seconds() > pool.claim_deadline;
    let rank_opt = pool.winners.iter().position(|addr| addr == &player_addr);

    if let Some(rank_index) = rank_opt {
        let rank = (rank_index + 1) as u8;
        let prize_amount = match rank_index {
            0 => pool
                .total_prize_amount
                .multiply_ratio(Uint128::new(50), Uint128::new(100)),
            1 => pool
                .total_prize_amount
                .multiply_ratio(Uint128::new(30), Uint128::new(100)),
            2 => pool
                .total_prize_amount
                .multiply_ratio(Uint128::new(20), Uint128::new(100)),
            _ => Uint128::zero(),
        };

        Ok(WinnablePrizeResponse {
            is_winner: true,
            prize_amount,
            has_claimed,
            claim_expired,
            rank,
        })
    } else {
        Ok(WinnablePrizeResponse {
            is_winner: false,
            prize_amount: Uint128::zero(),
            has_claimed,
            claim_expired,
            rank: 0,
        })
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        token_denom: config.token_denom,
        admin: config.admin,
        prize_pool_percentage: config.prize_pool_percentage,
        claim_period_seconds: config.claim_period_seconds,
        prize_leaderboard_type: config.prize_leaderboard_type,
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
