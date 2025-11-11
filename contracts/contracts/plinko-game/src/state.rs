use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::{GameRecord, LeaderboardType};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub token_denom: String,
    pub admin: Addr,
    pub funder_address: Addr,
    pub prize_pool_percentage: u8, // 0-100, percentage of house profit for the prize pool
    pub claim_period_seconds: u64, // How long winners have to claim their prize
    pub prize_leaderboard_type: LeaderboardType, // Which leaderboard determines the winners
}

// To track daily house profit separately from global stats
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct DailyStats {
    pub total_wagered: Uint128,
    pub total_won: Uint128,
}

// A snapshot of a day's prize pool and winners
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PrizePool {
    pub total_prize_amount: Uint128,
    pub winners: Vec<Addr>,  // Top 3 addresses
    pub claim_deadline: u64, // Timestamp (in seconds) after which the prize expires
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Stats {
    pub total_games: u64,
    pub total_wagered: Uint128,
    pub total_won: Uint128,
    pub house_balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct UserStats {
    pub total_games: u64,
    pub total_wagered: Uint128,
    pub total_won: Uint128,
    pub best_win_pnl: Uint128,
    pub best_win_multiplier: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LeaderboardEntry {
    pub player: Addr,
    pub value: Uint128,
    pub multiplier: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DailyLeaderboard {
    pub last_reset: u64, // Timestamp of last reset
    pub entries_best_wins: Vec<LeaderboardEntry>,
    pub entries_wagered: Vec<LeaderboardEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct DailyPlayerStats {
    pub total_wagered: Uint128,
    pub best_win_pnl: Uint128,
    pub best_win_multiplier: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATS: Item<Stats> = Item::new("stats");
pub const GAME_HISTORY: Map<(&Addr, u64), GameRecord> = Map::new("game_history");
pub const PLAYER_GAME_COUNT: Map<&Addr, u64> = Map::new("player_game_count");

// User statistics
pub const USER_STATS: Map<&Addr, UserStats> = Map::new("user_stats");

// Global leaderboards (all-time)
pub const GLOBAL_BEST_WINS: Item<Vec<LeaderboardEntry>> = Item::new("global_best_wins");
pub const GLOBAL_TOTAL_WAGERED: Item<Vec<LeaderboardEntry>> = Item::new("global_total_wagered");

// Daily leaderboard (resets at 00:00 UTC)
pub const DAILY_LEADERBOARD: Item<DailyLeaderboard> = Item::new("daily_leaderboard");
pub const DAILY_PLAYER_STATS: Map<&Addr, DailyPlayerStats> = Map::new("daily_player_stats");

// Tracks the house's total wagered and won for the current day only. Resets daily.
pub const DAILY_STATS: Item<DailyStats> = Item::new("daily_stats");

// Stores the prize pool for a given day index (e.g., day 19800 since epoch)
pub const PRIZE_POOL: Map<u64, PrizePool> = Map::new("prize_pool");

// Stores whether a player has claimed their prize for a given day index
// Key: (player_address, day_index) -> Value: bool
pub const CLAIMED_PRIZES: Map<(&Addr, u64), bool> = Map::new("claimed_prizes");
