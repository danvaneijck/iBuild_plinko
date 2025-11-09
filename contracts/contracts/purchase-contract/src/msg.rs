use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    /// Subdenom for the token (e.g., "plink")
    /// Full denom will be: factory/{contract_address}/{subdenom}
    pub subdenom: String,
    /// Token name for metadata (e.g., "Plink Token")
    pub token_name: String,
    /// Token symbol for metadata (e.g., "PLINK")
    pub token_symbol: String,
    /// Token decimals (typically 6 or 18)
    pub token_decimals: u8,
    /// Treasury wallet address that receives INJ
    pub treasury_address: String,
    /// Exchange rate: 1 INJ = X tokens (e.g., 100)
    pub exchange_rate: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Purchase tokens with INJ (send INJ with this message)
    Purchase {},
    /// Fund the house (game contract) with newly minted tokens (admin only)
    FundHouse {
        game_contract: String,
        amount: Uint128,
    },
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
    /// Calculate how many tokens you'd get for a given INJ amount
    #[returns(PreviewPurchaseResponse)]
    PreviewPurchase { inj_amount: Uint128 },
}

#[cw_serde]
pub struct ConfigResponse {
    pub token_denom: String,
    pub token_name: String,
    pub token_symbol: String,
    pub token_decimals: u8,
    pub treasury_address: Addr,
    pub exchange_rate: Uint128,
    pub admin: Addr,
}

#[cw_serde]
pub struct StatsResponse {
    pub total_inj_received: Uint128,
    pub total_tokens_minted: Uint128,
    pub total_purchases: u64,
    pub total_house_funding: Uint128,
}

#[cw_serde]
pub struct PreviewPurchaseResponse {
    pub inj_amount: Uint128,
    pub token_amount: Uint128,
    pub exchange_rate: Uint128,
}
