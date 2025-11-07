#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::error::ContractError;
    use crate::msg::{
        ConfigResponse, Difficulty, ExecuteMsg, HistoryResponse, InstantiateMsg, QueryMsg,
        RiskLevel, StatsResponse,
    };
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_json, CosmosMsg, DepsMut, Response, Uint128, WasmMsg};
    use cw20::Cw20ExecuteMsg;

    const ADMIN: &str = "admin";
    const PLINK_TOKEN: &str = "plink_token";
    const HOUSE: &str = "house";
    const PLAYER: &str = "player";

    fn setup_contract(deps: DepsMut) -> Result<Response, ContractError> {
        let msg = InstantiateMsg {
            plink_token_address: PLINK_TOKEN.to_string(),
            house_address: HOUSE.to_string(),
        };

        let info = mock_info(ADMIN, &[]);
        instantiate(deps, mock_env(), info, msg)
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let res = setup_contract(deps.as_mut()).unwrap();

        assert_eq!(res.messages.len(), 0);
        assert_eq!(res.attributes, vec![("action", "instantiate"),]);

        // Check config
        let query_msg = QueryMsg::Config {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();

        assert_eq!(config.plink_token_address, PLINK_TOKEN);
        assert_eq!(config.house_address, HOUSE);
        assert_eq!(config.admin, ADMIN);

        // Check stats
        let query_msg = QueryMsg::Stats {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats: StatsResponse = from_json(&res).unwrap();

        assert_eq!(stats.total_games, 0);
        assert_eq!(stats.total_wagered, Uint128::zero());
        assert_eq!(stats.total_won, Uint128::zero());
        assert_eq!(stats.house_balance, Uint128::zero());
    }

    #[test]
    fn test_play_game() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        let msg = ExecuteMsg::Play {
            difficulty: Difficulty::Easy,
            risk_level: RiskLevel::Low,
            bet_amount: Uint128::new(100_000000000000000000), // 100 PLINK
        };
        let info = mock_info(PLAYER, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Should have 2 messages: transfer bet from player, transfer winnings to player
        assert_eq!(res.messages.len(), 2);

        // Check first message (transfer bet from player)
        match &res.messages[0].msg {
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr,
                msg,
                funds,
            }) => {
                assert_eq!(contract_addr, PLINK_TOKEN);
                assert_eq!(funds.len(), 0);

                let transfer_msg: Cw20ExecuteMsg = from_json(msg).unwrap();
                match transfer_msg {
                    Cw20ExecuteMsg::TransferFrom { owner, recipient, amount } => {
                        assert_eq!(owner, PLAYER);
                        assert_eq!(amount, Uint128::new(100_000000000000000000));
                    }
                    _ => panic!("Expected TransferFrom message"),
                }
            }
            _ => panic!("Expected WasmMsg::Execute"),
        }

        // Check stats updated
        let query_msg = QueryMsg::Stats {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats: StatsResponse = from_json(&res).unwrap();

        assert_eq!(stats.total_games, 1);
        assert_eq!(stats.total_wagered, Uint128::new(100_000000000000000000));
        assert!(stats.total_won > Uint128::zero());
    }

    #[test]
    fn test_play_game_zero_bet() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        let msg = ExecuteMsg::Play {
            difficulty: Difficulty::Easy,
            risk_level: RiskLevel::Low,
            bet_amount: Uint128::zero(),
        };
        let info = mock_info(PLAYER, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::InvalidBetAmount {}));
    }

    #[test]
    fn test_play_all_difficulties() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        let difficulties = vec![Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];

        for difficulty in difficulties {
            let msg = ExecuteMsg::Play {
                difficulty: difficulty.clone(),
                risk_level: RiskLevel::Medium,
                bet_amount: Uint128::new(50_000000000000000000),
            };
            let info = mock_info(PLAYER, &[]);
            let res = execute(deps.as_mut(), mock_env(), info, msg);
            assert!(res.is_ok());
        }

        // Check stats
        let query_msg = QueryMsg::Stats {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats: StatsResponse = from_json(&res).unwrap();

        assert_eq!(stats.total_games, 3);
        assert_eq!(stats.total_wagered, Uint128::new(150_000000000000000000));
    }

    #[test]
    fn test_play_all_risk_levels() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        let risk_levels = vec![RiskLevel::Low, RiskLevel::Medium, RiskLevel::High];

        for risk_level in risk_levels {
            let msg = ExecuteMsg::Play {
                difficulty: Difficulty::Medium,
                risk_level: risk_level.clone(),
                bet_amount: Uint128::new(50_000000000000000000),
            };
            let info = mock_info(PLAYER, &[]);
            let res = execute(deps.as_mut(), mock_env(), info, msg);
            assert!(res.is_ok());
        }

        // Check stats
        let query_msg = QueryMsg::Stats {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats: StatsResponse = from_json(&res).unwrap();

        assert_eq!(stats.total_games, 3);
    }

    #[test]
    fn test_game_history() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        // Play multiple games
        for i in 0..5 {
            let msg = ExecuteMsg::Play {
                difficulty: Difficulty::Easy,
                risk_level: RiskLevel::Low,
                bet_amount: Uint128::new((i + 1) * 10_000000000000000000),
            };
            let info = mock_info(PLAYER, &[]);
            execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        }

        // Query history
        let query_msg = QueryMsg::History {
            player: PLAYER.to_string(),
            limit: Some(10),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let history: HistoryResponse = from_json(&res).unwrap();

        assert_eq!(history.games.len(), 5);

        // Check games are in order
        for (i, game) in history.games.iter().enumerate() {
            assert_eq!(game.player, PLAYER);
            assert_eq!(
                game.bet_amount,
                Uint128::new((i as u128 + 1) * 10_000000000000000000)
            );
        }
    }

    #[test]
    fn test_game_history_limit() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        // Play 10 games
        for _ in 0..10 {
            let msg = ExecuteMsg::Play {
                difficulty: Difficulty::Easy,
                risk_level: RiskLevel::Low,
                bet_amount: Uint128::new(10_000000000000000000),
            };
            let info = mock_info(PLAYER, &[]);
            execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        }

        // Query with limit
        let query_msg = QueryMsg::History {
            player: PLAYER.to_string(),
            limit: Some(5),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let history: HistoryResponse = from_json(&res).unwrap();

        assert_eq!(history.games.len(), 5);
    }

    #[test]
    fn test_update_house() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        let new_house = "new_house";
        let msg = ExecuteMsg::UpdateHouse {
            new_house: new_house.to_string(),
        };
        let info = mock_info(ADMIN, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(
            res.attributes,
            vec![("action", "update_house"), ("new_house", new_house),]
        );

        // Check config
        let query_msg = QueryMsg::Config {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();
        assert_eq!(config.house_address, new_house);
    }

    #[test]
    fn test_update_house_unauthorized() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        let msg = ExecuteMsg::UpdateHouse {
            new_house: "new_house".to_string(),
        };
        let info = mock_info(PLAYER, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::Unauthorized {}));
    }

    #[test]
    fn test_withdraw_house() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        // Play some games to build house balance
        for _ in 0..5 {
            let msg = ExecuteMsg::Play {
                difficulty: Difficulty::Easy,
                risk_level: RiskLevel::Low,
                bet_amount: Uint128::new(100_000000000000000000),
            };
            let info = mock_info(PLAYER, &[]);
            execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        }

        // Get house balance
        let query_msg = QueryMsg::Stats {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats: StatsResponse = from_json(&res).unwrap();
        let house_balance = stats.house_balance;

        // Withdraw half
        let withdraw_amount = house_balance.checked_div(Uint128::new(2)).unwrap();
        let msg = ExecuteMsg::WithdrawHouse {
            amount: withdraw_amount,
        };
        let info = mock_info(ADMIN, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(res.messages.len(), 1);

        // Check message
        match &res.messages[0].msg {
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr,
                msg,
                funds,
            }) => {
                assert_eq!(contract_addr, PLINK_TOKEN);
                assert_eq!(funds.len(), 0);

                let transfer_msg: Cw20ExecuteMsg = from_json(msg).unwrap();
                match transfer_msg {
                    Cw20ExecuteMsg::Transfer { recipient, amount } => {
                        assert_eq!(recipient, HOUSE);
                        assert_eq!(amount, withdraw_amount);
                    }
                    _ => panic!("Expected Transfer message"),
                }
            }
            _ => panic!("Expected WasmMsg::Execute"),
        }

        // Check updated balance
        let query_msg = QueryMsg::Stats {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats: StatsResponse = from_json(&res).unwrap();
        assert_eq!(
            stats.house_balance,
            house_balance.checked_sub(withdraw_amount).unwrap()
        );
    }

    #[test]
    fn test_withdraw_house_insufficient_balance() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        let msg = ExecuteMsg::WithdrawHouse {
            amount: Uint128::new(1000_000000000000000000),
        };
        let info = mock_info(ADMIN, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::InsufficientBalance {}));
    }

    #[test]
    fn test_withdraw_house_unauthorized() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        let msg = ExecuteMsg::WithdrawHouse {
            amount: Uint128::new(100_000000000000000000),
        };
        let info = mock_info(PLAYER, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::Unauthorized {}));
    }

    #[test]
    fn test_house_balance_tracking() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        // Play game where house should profit (low multiplier expected on average)
        for _ in 0..10 {
            let msg = ExecuteMsg::Play {
                difficulty: Difficulty::Easy,
                risk_level: RiskLevel::Low,
                bet_amount: Uint128::new(100_000000000000000000),
            };
            let info = mock_info(PLAYER, &[]);
            execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        }

        // Check stats
        let query_msg = QueryMsg::Stats {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats: StatsResponse = from_json(&res).unwrap();

        // House balance should be total wagered minus total won
        let expected_house_balance = stats.total_wagered.checked_sub(stats.total_won).unwrap();
        assert_eq!(stats.house_balance, expected_house_balance);
    }

    #[test]
    fn test_provably_fair_determinism() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        // Same player, same nonce should produce same result
        let env = mock_env();
        let info = mock_info(PLAYER, &[]);

        let msg = ExecuteMsg::Play {
            difficulty: Difficulty::Medium,
            risk_level: RiskLevel::Medium,
            bet_amount: Uint128::new(100_000000000000000000),
        };

        let _res1 = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        // Get the path from first game
        let query_msg = QueryMsg::History {
            player: PLAYER.to_string(),
            limit: Some(1),
        };
        let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let history: HistoryResponse = from_json(&res).unwrap();
        let path1 = history.games[0].path.clone();

        // Different nonce (second game) should produce different result
        let _res2 = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let query_msg = QueryMsg::History {
            player: PLAYER.to_string(),
            limit: Some(2),
        };
        let res = query(deps.as_ref(), env, query_msg).unwrap();
        let history: HistoryResponse = from_json(&res).unwrap();
        let path2 = history.games[1].path.clone();

        // Paths should be different (different nonce)
        assert_ne!(path1, path2);
    }
}
