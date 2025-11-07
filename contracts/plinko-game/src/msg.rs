use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    /// Address of the $PLINK token contract
    pub plink_token_address: String,
    /// House wallet that receives losing bets
    pub house_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Play the game
    Play {
        difficulty: Difficulty,
        risk_level: RiskLevel,
        bet_amount: Uint128,
    },
    /// Update house address (admin only)
    UpdateHouse { new_house: String },
    /// Withdraw house funds (admin only)
    WithdrawHouse { amount: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get current configuration
    #[returns(ConfigResponse)]
    Config {},
    /// Get game statistics
    #[returns(StatsResponse)]
    Stats {},
    /// Get game history for a player
    #[returns(HistoryResponse)]
    History { player: String, limit: Option<u32> },
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
    pub path: Vec<u8>,
}
