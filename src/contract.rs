#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Reply};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Buffer, BUFFER};


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    BUFFER.save(deps.storage, &Buffer::new());
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let ctx = ExecuteCtx(deps, env, info);
    match msg {
        ExecuteMsg::Push { data } => execute::push(ctx, data),
        ExecuteMsg::Pop {} => execute::pop(ctx),
        ExecuteMsg::Run { program } => execute::run(ctx, program),
        ExecuteMsg::Reset { } => execute::reset(ctx),
    }
}

pub struct ExecuteCtx<'a>(DepsMut<'a>, Env, MessageInfo);

pub mod execute {
    use cosmwasm_std::{CosmosMsg, Event, WasmMsg};
    use crate::msg::Command;
    use super::*;

    pub fn push(ctx: ExecuteCtx, data: String) -> Result<Response, ContractError> {
        println!("execute - push");
        let mut buffer = BUFFER.load(ctx.0.storage)?;
        buffer.push(data);
        BUFFER.save(ctx.0.storage, &buffer)?;
        Ok(Response::new().add_events(vec![Event::new("push").add_attribute("key", "value")]))
    }

    pub fn pop(ctx: ExecuteCtx) -> Result<Response, ContractError> {
        println!("execute - pop");
        let mut buffer = BUFFER.load(ctx.0.storage)?;
        let data = buffer.pop();
        BUFFER.save(ctx.0.storage, &buffer)?;
        Ok(Response::new().set_data(to_binary(&data)?))
    }

    pub fn run(ctx: ExecuteCtx, program: Vec<Command>) -> Result<Response, ContractError> {
        println!("execute - run");
        let mut res = Response::new();
        let contract_addr = ctx.1.contract.address.to_string();

        for cmd in program {
            match cmd {
                Command::Ev(ty, attrs) => {
                    res = res.add_event(Event::new(ty).add_attributes(attrs));
                }

                Command::Attr(k, v) => {
                    res = res.add_attribute(k, v);
                }

                Command::Data(d) => {
                    res = res.set_data(d);
                }

                Command::Msg(m) => {
                    res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: contract_addr.clone(),
                        msg: to_binary(&m)?,
                        funds: vec![]
                    }))
                }

                Command::BankMsg(m) => {
                    res = res.add_message(CosmosMsg::Bank(m))
                }

                Command::Sub(id, m, reply_on) => {
                    res = res.add_submessage(cosmwasm_std::SubMsg {
                        id,
                        msg: CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: contract_addr.clone(),
                            msg: to_binary(&m)?,
                            funds: vec![],
                        }),
                        gas_limit: None,
                        reply_on,
                    })
                }

                Command::Throw(s) => {
                    return Err(ContractError::Custom { msg: s });
                }
            }
        }

        Ok(res)
    }

    pub fn reset(ctx: ExecuteCtx) -> Result<Response, ContractError> {
        println!("execute - reset");
        BUFFER.save(ctx.0.storage, &Buffer::new())?;
        Ok(Response::new())
    }
}


pub struct ReplyCtx<'a>(DepsMut<'a>, Env);

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    let ctx = ReplyCtx(deps, env);
    match msg.id {
        1 => reply::reply_id(ctx, msg),
        2 => reply::reply_inv(ctx, msg),
        _ => Err(ContractError::Custom { msg: "Unknown reply id".to_string() }),
    }
}

pub mod reply {
    use cosmwasm_std::{Event, SubMsgResult};
    use super::*;

    /* There are 4 cases, implemented by 2 reply functions.
    reply_id(x) -> submessage success -> reply success & submessage fail -> reply fail
    reply_inv(x) -> submessage success -> reply error & submessage error -> reply success
    In addition, we need to test adding events and overwriting data.
    */

    pub fn reply_id(ctx: ReplyCtx, msg: Reply) -> Result<Response, ContractError> {
        let Reply { id, result } = msg;
        match result {
            SubMsgResult::Ok(res) => {
                Ok(Response::new().add_event(Event::new("reply_id").add_attribute("key1", "value1")))
            }
            SubMsgResult::Err(msg) => Err(ContractError::Custom { msg }),
        }
    }

    pub fn reply_inv(ctx: ReplyCtx, msg: Reply) -> Result<Response, ContractError> {
        let Reply { id, result } = msg;
        match result {
            SubMsgResult::Err(msg) => {
                Ok(Response::new().add_event(Event::new("reply_inv").add_attribute("err", msg.as_str())))
            }
            SubMsgResult::Ok(msg) => Err(ContractError::ReplyInv{ data: msg.data.map(|x| x.0), events: msg.events }),
        }
    }

}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetBuffer {} => to_binary(&query::buffer(deps)?),
    }
}

pub mod query {
    use crate::msg::GetBufferResponse;
    use crate::msg::QueryMsg::GetBuffer;
    use super::*;

    pub fn buffer(deps: Deps) -> StdResult<GetBufferResponse> {
        let buffer = BUFFER.load(deps.storage)?;
        Ok(GetBufferResponse { buffer })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};
    use crate::msg::GetBufferResponse;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetBuffer {}).unwrap();
        let value: GetBufferResponse = from_binary(&res).unwrap();
    }

    #[test]
    fn push() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Push { data: "test".to_string() };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetBuffer {}).unwrap();
        let value: GetBufferResponse = from_binary(&res).unwrap();
    }

}
