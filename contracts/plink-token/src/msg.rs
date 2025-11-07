use cosmwasm_schema::cw_serde;
use cw20::Cw20Coin;

#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Vec<Cw20Coin>,
    pub mint: Option<MinterResponse>,
}

#[cw_serde]
pub struct MinterResponse {
    pub minter: String,
    pub cap: Option<cosmwasm_std::Uint128>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Transfer { recipient: String, amount: cosmwasm_std::Uint128 },
    Burn { amount: cosmwasm_std::Uint128 },
    Send { contract: String, amount: cosmwasm_std::Uint128, msg: Binary },
    IncreaseAllowance { spender: String, amount: cosmwasm_std::Uint128, expires: Option<Expiration> },
    DecreaseAllowance { spender: String, amount: cosmwasm_std::Uint128, expires: Option<Expiration> },
    TransferFrom { owner: String, recipient: String, amount: cosmwasm_std::Uint128 },
    BurnFrom { owner: String, amount: cosmwasm_std::Uint128 },
    SendFrom { owner: String, contract: String, amount: cosmwasm_std::Uint128, msg: Binary },
    Mint { recipient: String, amount: cosmwasm_std::Uint128 },
}

#[cw_serde]
pub enum QueryMsg {
    Balance { address: String },
    TokenInfo {},
    Minter {},
    Allowance { owner: String, spender: String },
    AllAllowances { owner: String, start_after: Option<String>, limit: Option<u32> },
    AllAccounts { start_after: Option<String>, limit: Option<u32> },
}

use cosmwasm_std::Binary;
use cw20::Expiration;

impl From<InstantiateMsg> for cw20_base::msg::InstantiateMsg {
    fn from(msg: InstantiateMsg) -> Self {
        cw20_base::msg::InstantiateMsg {
            name: msg.name,
            symbol: msg.symbol,
            decimals: msg.decimals,
            initial_balances: msg.initial_balances,
            mint: msg.mint.map(|m| cw20::MinterResponse {
                minter: m.minter,
                cap: m.cap,
            }),
            marketing: None,
        }
    }
}

impl From<ExecuteMsg> for cw20_base::msg::ExecuteMsg {
    fn from(msg: ExecuteMsg) -> Self {
        match msg {
            ExecuteMsg::Transfer { recipient, amount } => cw20_base::msg::ExecuteMsg::Transfer { recipient, amount },
            ExecuteMsg::Burn { amount } => cw20_base::msg::ExecuteMsg::Burn { amount },
            ExecuteMsg::Send { contract, amount, msg } => cw20_base::msg::ExecuteMsg::Send { contract, amount, msg },
            ExecuteMsg::IncreaseAllowance { spender, amount, expires } => cw20_base::msg::ExecuteMsg::IncreaseAllowance { spender, amount, expires },
            ExecuteMsg::DecreaseAllowance { spender, amount, expires } => cw20_base::msg::ExecuteMsg::DecreaseAllowance { spender, amount, expires },
            ExecuteMsg::TransferFrom { owner, recipient, amount } => cw20_base::msg::ExecuteMsg::TransferFrom { owner, recipient, amount },
            ExecuteMsg::BurnFrom { owner, amount } => cw20_base::msg::ExecuteMsg::BurnFrom { owner, amount },
            ExecuteMsg::SendFrom { owner, contract, amount, msg } => cw20_base::msg::ExecuteMsg::SendFrom { owner, contract, amount, msg },
            ExecuteMsg::Mint { recipient, amount } => cw20_base::msg::ExecuteMsg::Mint { recipient, amount },
        }
    }
}

impl From<QueryMsg> for cw20_base::msg::QueryMsg {
    fn from(msg: QueryMsg) -> Self {
        match msg {
            QueryMsg::Balance { address } => cw20_base::msg::QueryMsg::Balance { address },
            QueryMsg::TokenInfo {} => cw20_base::msg::QueryMsg::TokenInfo {},
            QueryMsg::Minter {} => cw20_base::msg::QueryMsg::Minter {},
            QueryMsg::Allowance { owner, spender } => cw20_base::msg::QueryMsg::Allowance { owner, spender },
            QueryMsg::AllAllowances { owner, start_after, limit } => cw20_base::msg::QueryMsg::AllAllowances { owner, start_after, limit },
            QueryMsg::AllAccounts { start_after, limit } => cw20_base::msg::QueryMsg::AllAccounts { start_after, limit },
        }
    }
}
