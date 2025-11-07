#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::error::ContractError;
    use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, StatsResponse};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_json, BankMsg, Coin, CosmosMsg, DepsMut, Response, Uint128, WasmMsg};
    use cw20::Cw20ExecuteMsg;

    const ADMIN: &str = "admin";
    const PLINK_TOKEN: &str = "plink_token";
    const TREASURY: &str = "treasury";
    const USER: &str = "user";

    fn setup_contract(deps: DepsMut) -> Result<Response, ContractError> {
        let msg = InstantiateMsg {
            plink_token_address: PLINK_TOKEN.to_string(),
            treasury_address: TREASURY.to_string(),
            exchange_rate: Uint128::new(100), // 1 INJ = 100 PLINK
        };

        let info = mock_info(ADMIN, &[]);
        instantiate(deps, mock_env(), info, msg)
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let res = setup_contract(deps.as_mut()).unwrap();

        assert_eq!(res.messages.len(), 0);
        assert_eq!(
            res.attributes,
            vec![("action", "instantiate"), ("exchange_rate", "100"),]
        );

        // Check config
        let query_msg = QueryMsg::Config {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();

        assert_eq!(config.plink_token_address, PLINK_TOKEN);
        assert_eq!(config.treasury_address, TREASURY);
        assert_eq!(config.exchange_rate, Uint128::new(100));
        assert_eq!(config.admin, ADMIN);
    }

    #[test]
    fn test_instantiate_zero_exchange_rate() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            plink_token_address: PLINK_TOKEN.to_string(),
            treasury_address: TREASURY.to_string(),
            exchange_rate: Uint128::zero(),
        };

        let info = mock_info(ADMIN, &[]);
        let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::InvalidExchangeRate {}));
    }

    #[test]
    fn test_purchase() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        // Purchase with 10 INJ
        let msg = ExecuteMsg::Purchase {};
        let info = mock_info(USER, &coins(10_000000000000000000, "inj")); // 10 INJ with 18 decimals
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Should have 2 messages: send INJ to treasury, mint PLINK to user
        assert_eq!(res.messages.len(), 2);

        // Check first message (send INJ to treasury)
        match &res.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, TREASURY);
                assert_eq!(
                    amount,
                    &vec![Coin {
                        denom: "inj".to_string(),
                        amount: Uint128::new(10_000000000000000000),
                    }]
                );
            }
            _ => panic!("Expected BankMsg::Send"),
        }

        // Check second message (mint PLINK)
        match &res.messages[1].msg {
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr,
                msg,
                funds,
            }) => {
                assert_eq!(contract_addr, PLINK_TOKEN);
                assert_eq!(funds.len(), 0);

                let mint_msg: Cw20ExecuteMsg = from_json(msg).unwrap();
                match mint_msg {
                    Cw20ExecuteMsg::Mint { recipient, amount } => {
                        assert_eq!(recipient, USER);
                        assert_eq!(amount, Uint128::new(1000_000000000000000000)); // 1000 PLINK (10 * 100)
                    }
                    _ => panic!("Expected Mint message"),
                }
            }
            _ => panic!("Expected WasmMsg::Execute"),
        }

        // Check stats
        let query_msg = QueryMsg::Stats {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats: StatsResponse = from_json(&res).unwrap();

        assert_eq!(stats.total_inj_received, Uint128::new(10_000000000000000000));
        assert_eq!(stats.total_plink_minted, Uint128::new(1000_000000000000000000));
        assert_eq!(stats.total_purchases, 1);
    }

    #[test]
    fn test_purchase_no_funds() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        let msg = ExecuteMsg::Purchase {};
        let info = mock_info(USER, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::NoFundsSent {}));
    }

    #[test]
    fn test_purchase_wrong_denom() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        let msg = ExecuteMsg::Purchase {};
        let info = mock_info(USER, &coins(1000, "atom"));
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::NoFundsSent {}));
    }

    #[test]
    fn test_multiple_purchases() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        // First purchase
        let msg = ExecuteMsg::Purchase {};
        let info = mock_info(USER, &coins(5_000000000000000000, "inj"));
        execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

        // Second purchase
        let info = mock_info(USER, &coins(3_000000000000000000, "inj"));
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Check stats
        let query_msg = QueryMsg::Stats {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats: StatsResponse = from_json(&res).unwrap();

        assert_eq!(stats.total_inj_received, Uint128::new(8_000000000000000000));
        assert_eq!(stats.total_plink_minted, Uint128::new(800_000000000000000000));
        assert_eq!(stats.total_purchases, 2);
    }

    #[test]
    fn test_update_exchange_rate() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        // Update exchange rate
        let msg = ExecuteMsg::UpdateExchangeRate {
            new_rate: Uint128::new(200),
        };
        let info = mock_info(ADMIN, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(
            res.attributes,
            vec![("action", "update_exchange_rate"), ("new_rate", "200"),]
        );

        // Check config
        let query_msg = QueryMsg::Config {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();
        assert_eq!(config.exchange_rate, Uint128::new(200));
    }

    #[test]
    fn test_update_exchange_rate_unauthorized() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        let msg = ExecuteMsg::UpdateExchangeRate {
            new_rate: Uint128::new(200),
        };
        let info = mock_info(USER, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::Unauthorized {}));
    }

    #[test]
    fn test_update_exchange_rate_zero() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        let msg = ExecuteMsg::UpdateExchangeRate {
            new_rate: Uint128::zero(),
        };
        let info = mock_info(ADMIN, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::InvalidExchangeRate {}));
    }

    #[test]
    fn test_update_treasury() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        let new_treasury = "new_treasury";
        let msg = ExecuteMsg::UpdateTreasury {
            new_treasury: new_treasury.to_string(),
        };
        let info = mock_info(ADMIN, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(
            res.attributes,
            vec![("action", "update_treasury"), ("new_treasury", new_treasury),]
        );

        // Check config
        let query_msg = QueryMsg::Config {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();
        assert_eq!(config.treasury_address, new_treasury);
    }

    #[test]
    fn test_update_treasury_unauthorized() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        let msg = ExecuteMsg::UpdateTreasury {
            new_treasury: "new_treasury".to_string(),
        };
        let info = mock_info(USER, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::Unauthorized {}));
    }

    #[test]
    fn test_overflow_protection() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        // Try to purchase with max uint128
        let msg = ExecuteMsg::Purchase {};
        let info = mock_info(USER, &coins(u128::MAX, "inj"));
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::Overflow(_) | ContractError::OverflowError {}));
    }
}
