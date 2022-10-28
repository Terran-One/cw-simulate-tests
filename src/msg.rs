use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{ReplyOn, CosmosMsg, BankMsg};
use crate::state::Buffer;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Push { data: String },
    Pop {},
    Run { program: Vec<Command> },
    Reset {},
}

#[cw_serde]
pub enum Command {
    Ev(String, Vec<(String, String)>),
    Attr(String, String),
    Msg(ExecuteMsg),
    BankMsg(BankMsg),
    Sub(u64, ExecuteMsg, ReplyOn),
    Data(Vec<u8>),
    Throw(String),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetBufferResponse)]
    GetBuffer {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetBufferResponse {
    pub buffer: Buffer
}
