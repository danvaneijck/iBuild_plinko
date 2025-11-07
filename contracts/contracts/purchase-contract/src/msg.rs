use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    /// Address of the $PLINK token contract
    pub plink_token_address: String,
    /// Treasury wallet address that receives INJ
    pub treasury_address: String,
    /// Exchange rate: 1 INJ = X PLINK (e.g., 100)
    pub exchange_rate: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Purchase $PLINK with INJ (send INJ with this message)
    Purchase {},
    /// Update exchange rate (admin only)
    UpdateExchangeRate { new_rate: Uint128 },
    /// Update treasury address (admin only)
    UpdateTreasury { new_treasury: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get current configuration
    #[returns(ConfigResponse)]
    Config {},
    /// Get purchase statistics
    #[returns(StatsResponse)]
    Stats {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub plink_token_address: Addr,
    pub treasury_address: Addr,
    pub exchange_rate: Uint128,
    pub admin: Addr,
}

#[cw_serde]
pub struct StatsResponse {
    pub total_inj_received: Uint128,
    pub total_plink_minted: Uint128,
    pub total_purchases: u64,
}
