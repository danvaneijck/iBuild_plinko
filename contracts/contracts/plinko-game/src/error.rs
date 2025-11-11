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

    #[error("Invalid bet amount")]
    InvalidBetAmount {},

    #[error("Insufficient balance")]
    InsufficientBalance {},

    #[error("Overflow in calculation")]
    OverflowError {},

    #[error("Invalid multiplier index")]
    InvalidMultiplierIndex {},

    #[error("Invalid funds sent")]
    NoFundsSent {},

    #[error("Insufficient house balance to pay out winnings")]
    InsufficientHouseBalance {},

    #[error("Invalid percentage: must be between 0 and 100")]
    InvalidPercentage {},

    #[error("There is no prize available to claim for the specified day")]
    NoPrizeToClaim {},

    #[error("Prize for this day has already been claimed")]
    PrizeAlreadyClaimed {},

    #[error("You were not a top 3 winner on the specified day")]
    NotAWinner {},

    #[error("The claim period for this prize has expired")]
    ClaimPeriodExpired {},
}
