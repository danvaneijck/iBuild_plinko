use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("No INJ sent with purchase")]
    NoFundsSent {},

    #[error("Invalid exchange rate")]
    InvalidExchangeRate {},

    #[error("Overflow in calculation")]
    OverflowError {},
}
