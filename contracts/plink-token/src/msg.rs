use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw20::{Cw20Coin, MinterResponse};

#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Vec<Cw20Coin>,
    pub mint: Option<MinterResponse>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Transfer tokens to another address
    Transfer { recipient: String, amount: Uint128 },
    /// Burn tokens from sender's balance
    Burn { amount: Uint128 },
    /// Mint new tokens (only minter)
    Mint { recipient: String, amount: Uint128 },
    /// Increase allowance for spender
    IncreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<cw20::Expiration>,
    },
    /// Decrease allowance for spender
    DecreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<cw20::Expiration>,
    },
    /// Transfer tokens from another address (requires allowance)
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the current balance of the given address
    #[returns(cw20::BalanceResponse)]
    Balance { address: String },
    /// Returns metadata about the token
    #[returns(cw20::TokenInfoResponse)]
    TokenInfo {},
    /// Returns the minter address
    #[returns(MinterResponse)]
    Minter {},
    /// Returns allowance for spender
    #[returns(cw20::AllowanceResponse)]
    Allowance { owner: String, spender: String },
}
