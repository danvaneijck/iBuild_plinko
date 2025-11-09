use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub token_denom: String,
    pub funder_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Play {
        difficulty: Difficulty,
        risk_level: RiskLevel,
    },
    /// Withdraw house winnings (admin only)
    WithdrawHouse {
        amount: Uint128,
    },
    /// This message must be sent with the native tokens to be funded.
    FundHouse {},
    SyncBalance {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(StatsResponse)]
    Stats {},
    #[returns(HistoryResponse)]
    History { player: String, limit: Option<u32> },
    #[returns(UserStatsResponse)]
    UserStats { player: String },
    #[returns(LeaderboardResponse)]
    GlobalLeaderboard {
        leaderboard_type: LeaderboardType,
        limit: Option<u32>,
    },
    #[returns(LeaderboardResponse)]
    DailyLeaderboard {
        leaderboard_type: LeaderboardType,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub enum Difficulty {
    Easy,   // 8 rows
    Medium, // 12 rows
    Hard,   // 16 rows
}

#[cw_serde]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[cw_serde]
pub enum LeaderboardType {
    BestWins,     // Sorted by best single game PnL
    TotalWagered, // Sorted by cumulative wagered amount
}

#[cw_serde]
pub struct ConfigResponse {
    pub token_denom: String,
    pub admin: Addr,
}

#[cw_serde]
pub struct StatsResponse {
    pub total_games: u64,
    pub total_wagered: Uint128,
    pub total_won: Uint128,
    pub house_balance: Uint128,
}

#[cw_serde]
pub struct UserStatsResponse {
    pub player: Addr,
    pub total_games: u64,
    pub total_wagered: Uint128,
    pub total_won: Uint128,
    pub best_win_pnl: Uint128,
    pub best_win_multiplier: String,
}

#[cw_serde]
pub struct LeaderboardEntry {
    pub player: Addr,
    pub value: Uint128,
    pub multiplier: Option<String>, // Only for BestWins
}

#[cw_serde]
pub struct LeaderboardResponse {
    pub entries: Vec<LeaderboardEntry>,
    pub leaderboard_type: LeaderboardType,
}

#[cw_serde]
pub struct HistoryResponse {
    pub games: Vec<GameRecord>,
}

#[cw_serde]
pub struct GameRecord {
    pub player: Addr,
    pub difficulty: Difficulty,
    pub risk_level: RiskLevel,
    pub bet_amount: Uint128,
    pub multiplier: String,
    pub win_amount: Uint128,
    pub pnl: Uint128, // Profit/Loss (win_amount - bet_amount)
    pub timestamp: u64,
    pub path: Vec<bool>,
}
