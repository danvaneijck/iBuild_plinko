use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid bet amount")]
    InvalidBetAmount {},

    #[error("Insufficient balance")]
    InsufficientBalance {},

    #[error("Overflow in calculation")]
    OverflowError {},

    #[error("Invalid multiplier index")]
    InvalidMultiplierIndex {},
}
