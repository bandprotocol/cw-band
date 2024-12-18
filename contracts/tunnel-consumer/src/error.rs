use cosmwasm_std::{IbcOrder, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Invalid tunnel version")]
    InvalidTunnelVersion {
        actual: String,
        expected: String,
    },

    #[error("Invalid channel order")]
    InvalidChannelOrder {
        actual: IbcOrder,
        expected: IbcOrder,
    },
}
