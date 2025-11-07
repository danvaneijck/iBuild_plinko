use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub plink_token_address: String,
    pub house_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Play {
        difficulty: Difficulty,
        risk_level: RiskLevel,
        bet_amount: Uint128,
    },
    UpdateHouse {
        new_house: String,
    },
    WithdrawHouse {
        amount: Uint128,
    },
    /// Fund the house with PLINK tokens (admin only)
    /// Requires prior approval for the contract to transfer tokens
    FundHouse {
        amount: Uint128,
    },
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
}

#[cw_serde]
#[derive(Clone)]
pub enum Difficulty {
    Easy,   // 8 rows
    Medium, // 12 rows
    Hard,   // 16 rows
}

#[cw_serde]
#[derive(Clone)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[cw_serde]
pub struct ConfigResponse {
    pub plink_token_address: Addr,
    pub house_address: Addr,
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
    pub timestamp: u64,
    pub path: Vec<bool>,
}
