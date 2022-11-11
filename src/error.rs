use cosmwasm_std::{Event, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.

    #[error("custom: {msg}")]
    Custom { msg: String },

    #[error("reply to success")]
    ReplyInv { data: Option<Vec<u8>>, events: Vec<Event> }
}
