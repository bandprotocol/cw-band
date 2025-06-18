use cosmwasm_std::{IbcOrder, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Invalid tunnel version")]
    InvalidTunnelVersion { expected: String, actual: String },

    #[error("Invalid channel order")]
    InvalidChannelOrder {
        expected: IbcOrder,
        actual: IbcOrder,
    },
}
