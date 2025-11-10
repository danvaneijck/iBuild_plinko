#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::error::ContractError;
    use crate::msg::{
        ConfigResponse, Difficulty, ExecuteMsg, HistoryResponse, InstantiateMsg,
        LeaderboardResponse, LeaderboardType, QueryMsg, RiskLevel, StatsResponse,
        UserStatsResponse,
    };
    use cosmwasm_std::testing::{message_info, mock_env, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::{
        coin, coins, from_json, Addr, BankMsg, DepsMut, OwnedDeps, Response, Uint128,
    };

    const TOKEN_DENOM: &str = "factory/inj1contract/plink";

    fn mock_deps() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        // Use MockApi and configure it with the correct address prefix for Injective ("inj")
        let api = MockApi::default();

        OwnedDeps {
            storage: MockStorage::default(),
            api,
            querier: MockQuerier::default(),
            custom_query_type: std::marker::PhantomData,
        }
    }

    fn setup_contract(deps: DepsMut, admin: &Addr) -> Result<Response, ContractError> {
        let msg = InstantiateMsg {
            token_denom: TOKEN_DENOM.to_string(),
            funder_address: admin.to_string(),
        };

        let info = message_info(admin, &[]);
        instantiate(deps, mock_env(), info, msg)
    }

    fn fund_contract(deps: DepsMut, amount: Uint128) {
        // Simulate contract receiving tokens (e.g., from purchase contract's fund_house)
        // In real scenario, this would be done via BankMsg from purchase contract
        let mut stats = crate::state::STATS.load(deps.storage).unwrap();
        stats.house_balance = stats.house_balance.checked_add(amount).unwrap();
        crate::state::STATS.save(deps.storage, &stats).unwrap();
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let res = setup_contract(deps.as_mut(), &admin).unwrap();

        assert_eq!(res.messages.len(), 0);
        assert_eq!(res.attributes, vec![("action", "instantiate"),]);

        // Check config
        let query_msg = QueryMsg::Config {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();

        assert_eq!(config.token_denom, TOKEN_DENOM);
        assert_eq!(config.admin, admin);

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
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let player = deps.api.addr_make("player");
        setup_contract(deps.as_mut(), &admin).unwrap();
        // Fund contract (simulating purchase contract's fund_house)
        fund_contract(deps.as_mut(), Uint128::new(100_000_000000000000000000));

        let msg = ExecuteMsg::Play {
            difficulty: Difficulty::Easy,
            risk_level: RiskLevel::Low,
        };
        let info = message_info(&player, &coins(100_000000000000000000, TOKEN_DENOM));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Should have 1 message for sending winnings (if any)
        assert!(res.messages.len() <= 1);

        // Check stats updated
        let query_msg = QueryMsg::Stats {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats: StatsResponse = from_json(&res).unwrap();

        assert_eq!(stats.total_games, 1);
        assert_eq!(stats.total_wagered, Uint128::new(100_000000000000000000));
        assert!(stats.total_won >= Uint128::zero());

        // Check user stats
        let query_msg = QueryMsg::UserStats {
            player: player.to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let user_stats: UserStatsResponse = from_json(&res).unwrap();

        assert_eq!(user_stats.total_games, 1);
        assert_eq!(
            user_stats.total_wagered,
            Uint128::new(100_000000000000000000)
        );
    }

    #[test]
    fn test_play_game_no_funds() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let player = deps.api.addr_make("player");

        setup_contract(deps.as_mut(), &admin).unwrap();
        let msg = ExecuteMsg::Play {
            difficulty: Difficulty::Easy,
            risk_level: RiskLevel::Low,
        };
        let info = message_info(&player, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::InvalidBetAmount {}));
    }

    #[test]
    fn test_play_game_wrong_denom() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let player = deps.api.addr_make("player");

        setup_contract(deps.as_mut(), &admin).unwrap();
        let msg = ExecuteMsg::Play {
            difficulty: Difficulty::Easy,
            risk_level: RiskLevel::Low,
        };
        let info = message_info(&player, &coins(100, "wrong_denom"));
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::InvalidBetAmount {}));
    }

    #[test]
    fn test_multiple_purchases() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        setup_contract(deps.as_mut(), &admin).unwrap();
        // Fund contract
        fund_contract(deps.as_mut(), Uint128::new(500_000_000000000000000000));

        // Play multiple games with different players
        for i in 0..5 {
            let player = Addr::unchecked(format!("player{}", i));
            let msg = ExecuteMsg::Play {
                difficulty: Difficulty::Easy,
                risk_level: RiskLevel::Low,
            };
            let info = message_info(&player, &coins(100_000000000000000000, TOKEN_DENOM));
            execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        }

        // Check stats
        let query_msg = QueryMsg::Stats {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats: StatsResponse = from_json(&res).unwrap();

        assert_eq!(stats.total_games, 5);
        assert_eq!(stats.total_wagered, Uint128::new(500_000000000000000000));
    }

    #[test]
    fn test_global_leaderboard_best_wins() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        setup_contract(deps.as_mut(), &admin).unwrap();
        // Fund contract
        fund_contract(deps.as_mut(), Uint128::new(500_000_000000000000000000));

        // Play multiple games with different players
        for i in 0..5 {
            let player = Addr::unchecked(format!("player{}", i));
            let msg = ExecuteMsg::Play {
                difficulty: Difficulty::Easy,
                risk_level: RiskLevel::Low,
            };
            let info = message_info(&player, &coins(100_000000000000000000, TOKEN_DENOM));
            execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        }

        // Query global leaderboard
        let query_msg = QueryMsg::GlobalLeaderboard {
            leaderboard_type: LeaderboardType::BestWins,
            limit: Some(10),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let leaderboard: LeaderboardResponse = from_json(&res).unwrap();

        assert!(leaderboard.entries.len() <= 5);
        // Entries should be sorted by value (descending)
        for i in 0..leaderboard.entries.len().saturating_sub(1) {
            assert!(leaderboard.entries[i].value >= leaderboard.entries[i + 1].value);
        }
    }

    #[test]
    fn test_global_leaderboard_total_wagered() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        setup_contract(deps.as_mut(), &admin).unwrap();
        // Fund contract
        fund_contract(deps.as_mut(), Uint128::new(500_000_000000000000000000));

        let player1 = Addr::unchecked("player1");
        let player2 = Addr::unchecked("player2");

        // Player 1 plays multiple games
        for _ in 0..3 {
            let msg = ExecuteMsg::Play {
                difficulty: Difficulty::Easy,
                risk_level: RiskLevel::Low,
            };
            let info = message_info(&player1, &coins(100_000000000000000000, TOKEN_DENOM));
            execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        }

        // Player 2 plays one game
        let msg = ExecuteMsg::Play {
            difficulty: Difficulty::Easy,
            risk_level: RiskLevel::Low,
        };
        let info = message_info(&player2, &coins(50_000000000000000000, TOKEN_DENOM));
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Query global leaderboard
        let query_msg = QueryMsg::GlobalLeaderboard {
            leaderboard_type: LeaderboardType::TotalWagered,
            limit: Some(10),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let leaderboard: LeaderboardResponse = from_json(&res).unwrap();

        assert_eq!(leaderboard.entries.len(), 2);
        // Player 1 should be first (300 total wagered)
        assert_eq!(leaderboard.entries[0].player, player1);
        assert_eq!(
            leaderboard.entries[0].value,
            Uint128::new(300_000000000000000000)
        );
    }

    #[test]
    fn test_daily_leaderboard_reset() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        setup_contract(deps.as_mut(), &admin).unwrap();
        // Fund contract
        fund_contract(deps.as_mut(), Uint128::new(500_000_000000000000000000));

        let player1 = Addr::unchecked("player1");
        let player2 = Addr::unchecked("player2");

        // Play a game
        let msg = ExecuteMsg::Play {
            difficulty: Difficulty::Easy,
            risk_level: RiskLevel::Low,
        };
        let info = message_info(&player1, &coins(100_000000000000000000, TOKEN_DENOM));
        let mut env = mock_env();
        execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();

        // Query daily leaderboard
        let query_msg = QueryMsg::DailyLeaderboard {
            leaderboard_type: LeaderboardType::TotalWagered,
            limit: Some(10),
        };
        let res = query(deps.as_ref(), env.clone(), query_msg.clone()).unwrap();
        let leaderboard: LeaderboardResponse = from_json(&res).unwrap();
        assert_eq!(leaderboard.entries.len(), 1);

        // Advance time by 1 day
        env.block.time = env.block.time.plus_seconds(86400);

        // Query again - should be empty due to reset
        let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let leaderboard: LeaderboardResponse = from_json(&res).unwrap();
        assert_eq!(leaderboard.entries.len(), 0);

        // Play another game after reset
        let info = message_info(&player2, &coins(100_000000000000000000, TOKEN_DENOM));
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Query again - should have new entry
        let query_msg = QueryMsg::DailyLeaderboard {
            leaderboard_type: LeaderboardType::TotalWagered,
            limit: Some(10),
        };
        let res = query(deps.as_ref(), env, query_msg).unwrap();
        let leaderboard: LeaderboardResponse = from_json(&res).unwrap();
        assert_eq!(leaderboard.entries.len(), 1);
        assert_eq!(leaderboard.entries[0].player, player2);
    }

    #[test]
    fn test_user_stats_tracking() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let player = deps.api.addr_make("player");

        setup_contract(deps.as_mut(), &admin).unwrap();
        // Fund contract
        fund_contract(deps.as_mut(), Uint128::new(500_000_000000000000000000));

        // Play multiple games
        for _ in 0..3 {
            let msg = ExecuteMsg::Play {
                difficulty: Difficulty::Easy,
                risk_level: RiskLevel::Low,
            };
            let info = message_info(&player, &coins(100_000000000000000000, TOKEN_DENOM));
            execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        }

        // Query user stats
        let query_msg = QueryMsg::UserStats {
            player: player.to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let user_stats: UserStatsResponse = from_json(&res).unwrap();

        assert_eq!(user_stats.total_games, 3);
        assert_eq!(
            user_stats.total_wagered,
            Uint128::new(300_000000000000000000)
        );
        assert!(user_stats.best_win_pnl >= Uint128::zero());
        assert_ne!(user_stats.best_win_multiplier, "0.0x");
    }

    #[test]
    fn test_game_history() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let player = deps.api.addr_make("player");

        setup_contract(deps.as_mut(), &admin).unwrap();
        // Fund contract
        fund_contract(deps.as_mut(), Uint128::new(150_000_000000000000000000));

        // Play multiple games
        for i in 0..5 {
            let msg = ExecuteMsg::Play {
                difficulty: Difficulty::Easy,
                risk_level: RiskLevel::Low,
            };
            let info = message_info(
                &player,
                &coins((i + 1) * 10_000000000000000000, TOKEN_DENOM),
            );
            execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        }

        // Query history
        let query_msg = QueryMsg::History {
            player: player.to_string(),
            limit: Some(10),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let history: HistoryResponse = from_json(&res).unwrap();

        assert_eq!(history.games.len(), 5);

        // Check games are in order and have PnL
        for (i, game) in history.games.iter().enumerate() {
            assert_eq!(game.player, player);
            assert_eq!(
                game.bet_amount,
                Uint128::new((i as u128 + 1) * 10_000000000000000000)
            );
            // PnL should be calculated (win_amount - bet_amount)
            assert_eq!(game.pnl, game.win_amount.saturating_sub(game.bet_amount));
        }
    }

    #[test]
    fn test_withdraw_house() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let player = deps.api.addr_make("player");

        setup_contract(deps.as_mut(), &admin).unwrap();
        // Fund contract
        fund_contract(deps.as_mut(), Uint128::new(500_000000000000000000));

        // Play some games to build house balance
        for _ in 0..5 {
            let msg = ExecuteMsg::Play {
                difficulty: Difficulty::Easy,
                risk_level: RiskLevel::Low,
            };
            let info = message_info(&player, &coins(100_000000000000000000, TOKEN_DENOM));
            execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        }

        // Get house balance
        let query_msg = QueryMsg::Stats {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats: StatsResponse = from_json(&res).unwrap();
        let house_balance = stats.house_balance;

        // Only test withdrawal if house has positive balance
        if house_balance > Uint128::zero() {
            // Withdraw half
            let withdraw_amount = house_balance.checked_div(Uint128::new(2)).unwrap();
            let msg = ExecuteMsg::WithdrawHouse {
                amount: withdraw_amount,
            };
            let info = message_info(&admin, &[]);
            let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

            assert_eq!(res.messages.len(), 1);

            // Check message
            match &res.messages[0].msg {
                cosmwasm_std::CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                    assert_eq!(to_address, &admin.to_string());
                    assert_eq!(amount, &vec![coin(withdraw_amount.u128(), TOKEN_DENOM)]);
                }
                _ => panic!("Expected BankMsg::Send"),
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
    }

    #[test]
    fn test_withdraw_house_insufficient_balance() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        setup_contract(deps.as_mut(), &admin).unwrap();
        let msg = ExecuteMsg::WithdrawHouse {
            amount: Uint128::new(1000_000000000000000000),
        };
        let info = message_info(&admin, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::InsufficientBalance {}));
    }

    #[test]
    fn test_withdraw_house_unauthorized() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        setup_contract(deps.as_mut(), &admin).unwrap();
        let player = Addr::unchecked("player");
        let msg = ExecuteMsg::WithdrawHouse {
            amount: Uint128::new(100_000000000000000000),
        };
        let info = message_info(&player, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::Unauthorized {}));
    }

    #[test]
    fn test_house_balance_tracking() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        setup_contract(deps.as_mut(), &admin).unwrap();
        // Fund contract with enough capital
        fund_contract(deps.as_mut(), Uint128::new(1000_000000000000000000));

        let player = Addr::unchecked("player");

        // Play game where house should profit (low multiplier expected on average)
        for _ in 0..10 {
            let msg = ExecuteMsg::Play {
                difficulty: Difficulty::Easy,
                risk_level: RiskLevel::Low,
            };
            let info = message_info(&player, &coins(100_000000000000000000, TOKEN_DENOM));
            execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        }

        // Check stats
        let query_msg = QueryMsg::Stats {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats: StatsResponse = from_json(&res).unwrap();

        // House balance should be: initial_funding + total_wagered - total_won
        let expected_house_balance = Uint128::new(1000_000000000000000000)
            .checked_add(stats.total_wagered)
            .unwrap()
            .saturating_sub(stats.total_won);

        assert_eq!(stats.house_balance, expected_house_balance);
    }
    
    #[test]
    fn test_daily_leaderboard_logic_and_reset() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        setup_contract(deps.as_mut(), &admin).unwrap();
        fund_contract(deps.as_mut(), Uint128::new(1000_000000000000000000));
    
        let player1 = Addr::unchecked("player1");
        let player2 = Addr::unchecked("player2");
        let mut env = mock_env();
    
        // --- Day 1 ---
        // Player 1 plays, wagering 100
        let msg = ExecuteMsg::Play { difficulty: Difficulty::Easy, risk_level: RiskLevel::Low };
        let info = message_info(&player1, &coins(100_000000000000000000, TOKEN_DENOM));
        execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();
    
        // Player 1 plays again, wagering 50
        let info = message_info(&player1, &coins(50_000000000000000000, TOKEN_DENOM));
        execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();
    
        // Check Daily Leaderboard for Day 1
        let query_daily_msg = QueryMsg::DailyLeaderboard {
            leaderboard_type: LeaderboardType::TotalWagered,
            limit: Some(10),
        };
        let res = query(deps.as_ref(), env.clone(), query_daily_msg.clone()).unwrap();
        let daily_lb: LeaderboardResponse = from_json(&res).unwrap();
        assert_eq!(daily_lb.entries.len(), 1);
        assert_eq!(daily_lb.entries[0].player, player1);
        assert_eq!(daily_lb.entries[0].value, Uint128::new(150_000000000000000000)); // 100 + 50
    
        // --- Advance time by 1 day ---
        env.block.time = env.block.time.plus_seconds(86401); // 1 day + 1 second
    
        // Querying before a new play should show an empty board because the query itself checks for reset
        let res = query(deps.as_ref(), env.clone(), query_daily_msg.clone()).unwrap();
        let daily_lb: LeaderboardResponse = from_json(&res).unwrap();
        assert_eq!(daily_lb.entries.len(), 0);
    
        // --- Day 2 ---
        // Player 2 plays, wagering 200. This tx will trigger the state-changing reset.
        let info = message_info(&player2, &coins(200_000000000000000000, TOKEN_DENOM));
        execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();
    
        // Check Daily Leaderboard for Day 2
        let res = query(deps.as_ref(), env.clone(), query_daily_msg.clone()).unwrap();
        let daily_lb: LeaderboardResponse = from_json(&res).unwrap();
        assert_eq!(daily_lb.entries.len(), 1);
        assert_eq!(daily_lb.entries[0].player, player2);
        assert_eq!(daily_lb.entries[0].value, Uint128::new(200_000000000000000000)); // Only player2's score
    
        // Check that Global Leaderboard was NOT reset
        let query_global_msg = QueryMsg::GlobalLeaderboard {
            leaderboard_type: LeaderboardType::TotalWagered,
            limit: Some(10),
        };
        let res = query(deps.as_ref(), env.clone(), query_global_msg).unwrap();
        let global_lb: LeaderboardResponse = from_json(&res).unwrap();
        assert_eq!(global_lb.entries.len(), 2);
        assert_eq!(global_lb.entries[0].player, player2); // Player 2 has wagered more overall now
        assert_eq!(global_lb.entries[0].value, Uint128::new(200_000000000000000000));
        assert_eq!(global_lb.entries[1].player, player1); // Player 1 is second
        assert_eq!(global_lb.entries[1].value, Uint128::new(150_000000000000000000));
    }

    #[test]
    fn test_leaderboard_sorting_and_updates() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        setup_contract(deps.as_mut(), &admin).unwrap();
        fund_contract(deps.as_mut(), Uint128::new(1000_000000000000000000));

        let player1 = Addr::unchecked("player1");
        let player2 = Addr::unchecked("player2");

        // P1 wagers 100
        let msg = ExecuteMsg::Play { difficulty: Difficulty::Easy, risk_level: RiskLevel::Low };
        let info = message_info(&player1, &coins(100_000000000000000000, TOKEN_DENOM));
        execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

        // P2 wagers 200
        let info = message_info(&player2, &coins(200_000000000000000000, TOKEN_DENOM));
        execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

        // Check leaderboard - P2 should be first
        let query_msg = QueryMsg::GlobalLeaderboard { leaderboard_type: LeaderboardType::TotalWagered, limit: Some(10) };
        let res = query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap();
        let leaderboard: LeaderboardResponse = from_json(&res).unwrap();
        assert_eq!(leaderboard.entries.len(), 2);
        assert_eq!(leaderboard.entries[0].player, player2);
        assert_eq!(leaderboard.entries[1].player, player1);

        // P1 wagers another 150, for a total of 250
        let info = message_info(&player1, &coins(150_000000000000000000, TOKEN_DENOM));
        execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

        // Check leaderboard again - P1 should now be first
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let leaderboard: LeaderboardResponse = from_json(&res).unwrap();
        assert_eq!(leaderboard.entries.len(), 2);
        assert_eq!(leaderboard.entries[0].player, player1);
        assert_eq!(leaderboard.entries[0].value, Uint128::new(250_000000000000000000));
        assert_eq!(leaderboard.entries[1].player, player2);
        assert_eq!(leaderboard.entries[1].value, Uint128::new(200_000000000000000000));
    }
}
