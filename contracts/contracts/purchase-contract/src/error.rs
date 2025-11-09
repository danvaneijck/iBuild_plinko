use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("No funds sent")]
    NoFundsSent {},

    #[error("Invalid exchange rate")]
    InvalidExchangeRate {},

    #[error("Invalid amount")]
    InvalidAmount {},

    #[error("Overflow in calculation")]
    OverflowError {},
}
