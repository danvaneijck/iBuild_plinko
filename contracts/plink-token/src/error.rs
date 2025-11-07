use cosmwasm_std::StdError;
use cw20_base::ContractError as Cw20ContractError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Cw20(#[from] Cw20ContractError),
}
