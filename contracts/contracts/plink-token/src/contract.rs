use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw20_base::contract::{
    execute as cw20_execute, instantiate as cw20_instantiate, query as cw20_query,
};
use cw20_base::ContractError as Cw20ContractError;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::MINTER;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Cw20ContractError> {
    // Store minter if provided
    if let Some(ref minter) = msg.mint {
        let minter_addr = deps.api.addr_validate(&minter.minter)?;
        MINTER.save(deps.storage, &minter_addr)?;
    }

    // Use cw20-base instantiate
    let cw20_msg = cw20_base::msg::InstantiateMsg {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        initial_balances: msg.initial_balances,
        mint: msg.mint,
        marketing: None,
    };

    cw20_instantiate(deps, env, info, cw20_msg)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Cw20ContractError> {
    match msg {
        ExecuteMsg::Transfer { recipient, amount } => {
            let cw20_msg = cw20_base::msg::ExecuteMsg::Transfer { recipient, amount };
            cw20_execute(deps, env, info, cw20_msg)
        }
        ExecuteMsg::Burn { amount } => {
            let cw20_msg = cw20_base::msg::ExecuteMsg::Burn { amount };
            cw20_execute(deps, env, info, cw20_msg)
        }
        ExecuteMsg::Mint { recipient, amount } => {
            let cw20_msg = cw20_base::msg::ExecuteMsg::Mint { recipient, amount };
            cw20_execute(deps, env, info, cw20_msg)
        }
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => {
            let cw20_msg = cw20_base::msg::ExecuteMsg::IncreaseAllowance {
                spender,
                amount,
                expires,
            };
            cw20_execute(deps, env, info, cw20_msg)
        }
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => {
            let cw20_msg = cw20_base::msg::ExecuteMsg::DecreaseAllowance {
                spender,
                amount,
                expires,
            };
            cw20_execute(deps, env, info, cw20_msg)
        }
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => {
            let cw20_msg = cw20_base::msg::ExecuteMsg::TransferFrom {
                owner,
                recipient,
                amount,
            };
            cw20_execute(deps, env, info, cw20_msg)
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { address } => {
            let cw20_msg = cw20_base::msg::QueryMsg::Balance { address };
            cw20_query(deps, env, cw20_msg)
        }
        QueryMsg::TokenInfo {} => {
            let cw20_msg = cw20_base::msg::QueryMsg::TokenInfo {};
            cw20_query(deps, env, cw20_msg)
        }
        QueryMsg::Minter {} => {
            let cw20_msg = cw20_base::msg::QueryMsg::Minter {};
            cw20_query(deps, env, cw20_msg)
        }
        QueryMsg::Allowance { owner, spender } => {
            let cw20_msg = cw20_base::msg::QueryMsg::Allowance { owner, spender };
            cw20_query(deps, env, cw20_msg)
        }
    }
}
