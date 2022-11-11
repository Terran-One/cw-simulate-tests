#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::InstantiateMsg;
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ).with_reply(crate::contract::reply);
        Box::new(contract)
    }

    const USER: &str = "USER";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "denom";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1),
                    }],
                )
                .unwrap();
        })
    }

    fn proper_instantiate() -> (App, CwTemplateContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());

        let msg = InstantiateMsg { };
        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        let cw_template_contract = CwTemplateContract(cw_template_contract_addr);

        (app, cw_template_contract)
    }

    mod tests {
        use cosmwasm_std::ReplyOn;
        use super::*;
        use crate::msg::{Command, ExecuteMsg, GetBufferResponse, QueryMsg};

        #[test]
        fn buffer() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let msg = ExecuteMsg::Push { data: "test".to_string() };
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            let res = app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
            println!("{:#?}", res);
        }

        // helpers
        fn attr(key: &str, value: &str) -> Command {
            Command::Attr(key.to_string(), value.to_string())
        }

        fn ev(ty: &str, attrs: Vec<(&str, &str)>) -> Command {
            Command::Ev(ty.to_string(), attrs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect())
        }

        fn msg(m: ExecuteMsg) -> Command {
            Command::Msg(m)
        }

        fn sub(id: u64, m: ExecuteMsg, reply_on: ReplyOn) -> Command {
            Command::Sub(id, m, reply_on)
        }

        fn push(s: &str) -> ExecuteMsg {
           ExecuteMsg::Push { data: s.to_string() }
        }

        fn run(cmds: Vec<ExecuteMsg>) -> ExecuteMsg {
            let program = cmds.into_iter().map(|c| msg(c)).collect();
            ExecuteMsg::Run { program }
        }

        fn throw(s: &str) -> Command {
            Command::Throw(s.to_string())
        }

        fn data(v: Vec<u8>) -> Command {
            Command::Data(v)
        }

        #[test]
        fn test_run() {
            let (mut app, cw_template_contract) = proper_instantiate();

            // { push: { data: string }} -> adds 1 to end of buffer
            // { run: { program: Vec<Command> }} -> runs the commands listed in program
            let msg = ExecuteMsg::Run { 
                program: vec![
                    Command::Msg(ExecuteMsg::Run {
                        program: vec![
                            Command::Msg(ExecuteMsg::Push { data: "A".to_string() }),
                            Command::Msg(ExecuteMsg::Push { data: "B".to_string() }),
                        ]
                    }),
                    Command::Msg(ExecuteMsg::Push { data: "C".to_string() })
                ]
            };
            
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            let res = app.execute(Addr::unchecked(USER), cosmos_msg);
            println!("{:#?}", res.unwrap().data);

            let res: GetBufferResponse = app.wrap().query_wasm_smart(&cw_template_contract.0, &QueryMsg::GetBuffer {}).unwrap();
            println!("{:?}", res);
        }
    }
}
