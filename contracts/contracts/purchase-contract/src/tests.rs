#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::error::ContractError;
    use crate::msg::{
        ConfigResponse, ExecuteMsg, InstantiateMsg, PreviewPurchaseResponse, QueryMsg,
        StatsResponse,
    };
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{coins, from_json, Addr, DepsMut, OwnedDeps, Response, Uint128};
    use injective_cosmwasm::InjectiveMsgWrapper;

    const SUBDENOM: &str = "plink";
    const TOKEN_NAME: &str = "Plink Token";
    const TOKEN_SYMBOL: &str = "PLINK";

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

    fn setup_contract(
        deps: DepsMut,
        admin: &Addr,
        treasury: &Addr,
    ) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
        let msg = InstantiateMsg {
            subdenom: SUBDENOM.to_string(),
            token_name: TOKEN_NAME.to_string(),
            token_symbol: TOKEN_SYMBOL.to_string(),
            token_decimals: 6,
            treasury_address: treasury.to_string(),
            exchange_rate: Uint128::new(100), // 1 INJ = 100 tokens
        };

        let info = message_info(admin, &[]);
        instantiate(deps, mock_env(), info, msg)
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let treasury = deps.api.addr_make("treasury");
        let res = setup_contract(deps.as_mut(), &admin, &treasury).unwrap();

        // Should have 2 messages: create denom + set metadata
        assert_eq!(res.messages.len(), 2);

        let expected_denom = format!("factory/{}/{}", mock_env().contract.address, SUBDENOM);

        assert_eq!(
            res.attributes,
            vec![
                ("action", "instantiate"),
                ("token_denom", &expected_denom),
                ("exchange_rate", "100"),
            ]
        );

        // Check config
        let query_msg = QueryMsg::Config {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();

        assert_eq!(config.token_denom, expected_denom);
        assert_eq!(config.token_name, TOKEN_NAME);
        assert_eq!(config.token_symbol, TOKEN_SYMBOL);
        assert_eq!(config.token_decimals, 6);
        assert_eq!(config.treasury_address, treasury);
        assert_eq!(config.exchange_rate, Uint128::new(100));
        assert_eq!(config.admin, admin);

        // Check stats
        let query_msg = QueryMsg::Stats {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats: StatsResponse = from_json(&res).unwrap();

        assert_eq!(stats.total_inj_received, Uint128::zero());
        assert_eq!(stats.total_tokens_minted, Uint128::zero());
        assert_eq!(stats.total_purchases, 0);
        assert_eq!(stats.total_house_funding, Uint128::zero());
    }

    #[test]
    fn test_instantiate_invalid_exchange_rate() {
        let mut deps = mock_dependencies();

        let admin = deps.api.addr_make("admin");
        let treasury = deps.api.addr_make("treasury");

        let msg = InstantiateMsg {
            subdenom: SUBDENOM.to_string(),
            token_name: TOKEN_NAME.to_string(),
            token_symbol: TOKEN_SYMBOL.to_string(),
            token_decimals: 6,
            treasury_address: treasury.to_string(),
            exchange_rate: Uint128::zero(),
        };

        let info = message_info(&admin, &[]);
        let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::InvalidExchangeRate {}));
    }

    #[test]
    fn test_purchase() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let buyer = deps.api.addr_make("buyer");

        let treasury = deps.api.addr_make("treasury");
        setup_contract(deps.as_mut(), &admin, &treasury).unwrap();

        // Purchase with 10 INJ
        let msg = ExecuteMsg::Purchase {};
        let info = message_info(&buyer, &coins(10_000000000000000000, "inj"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Should have 2 messages: mint tokens + send INJ to treasury
        assert_eq!(res.messages.len(), 2);

        // Check attributes
        assert_eq!(
            res.attributes,
            vec![
                ("action", "purchase"),
                ("buyer", &buyer.to_string()),
                ("inj_amount", "10000000000000000000"),
                ("token_amount", "1000000000000000000000"),
            ]
        );

        // Check stats updated
        let query_msg = QueryMsg::Stats {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats: StatsResponse = from_json(&res).unwrap();

        assert_eq!(
            stats.total_inj_received,
            Uint128::new(10_000000000000000000)
        );
        assert_eq!(
            stats.total_tokens_minted,
            Uint128::new(1000_000000000000000000)
        );
        assert_eq!(stats.total_purchases, 1);
        assert_eq!(stats.total_house_funding, Uint128::zero());
    }

    #[test]
    fn test_purchase_no_funds() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let treasury = deps.api.addr_make("treasury");
        let buyer = deps.api.addr_make("buyer");

        setup_contract(deps.as_mut(), &admin, &treasury).unwrap();

        let msg = ExecuteMsg::Purchase {};
        let info = message_info(&buyer, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::NoFundsSent {}));
    }

    #[test]
    fn test_purchase_wrong_denom() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let treasury = deps.api.addr_make("treasury");
        let buyer = deps.api.addr_make("buyer");
        setup_contract(deps.as_mut(), &admin, &treasury).unwrap();

        let msg = ExecuteMsg::Purchase {};
        let info = message_info(&buyer, &coins(10_000000000000000000, "wrong_denom"));
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::NoFundsSent {}));
    }

    #[test]
    fn test_multiple_purchases() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let treasury = deps.api.addr_make("treasury");
        let buyer = deps.api.addr_make("buyer");
        setup_contract(deps.as_mut(), &admin, &treasury).unwrap();

        // First purchase: 5 INJ
        let msg = ExecuteMsg::Purchase {};
        let info = message_info(&buyer, &coins(5_000000000000000000, "inj"));
        execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

        // Second purchase: 3 INJ
        let info = message_info(&buyer, &coins(3_000000000000000000, "inj"));
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Check stats
        let query_msg = QueryMsg::Stats {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats: StatsResponse = from_json(&res).unwrap();

        assert_eq!(stats.total_inj_received, Uint128::new(8_000000000000000000));
        assert_eq!(
            stats.total_tokens_minted,
            Uint128::new(800_000000000000000000)
        );
        assert_eq!(stats.total_purchases, 2);
    }

    #[test]
    fn test_fund_house() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let treasury = deps.api.addr_make("treasury");
        let game = deps.api.addr_make("game");

        setup_contract(deps.as_mut(), &admin, &treasury).unwrap();

        let msg = ExecuteMsg::FundHouse {
            game_contract: game.to_string(),
            amount: Uint128::new(1000_000000000000000000),
        };
        let info = message_info(&admin, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Should have 1 message: mint tokens to game contract
        assert_eq!(res.messages.len(), 2);

        assert_eq!(
            res.attributes,
            vec![
                ("action", "fund_house"),
                ("game_contract", &game.to_string()),
                ("amount", "1000000000000000000000"),
            ]
        );

        // Check stats
        let query_msg = QueryMsg::Stats {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats: StatsResponse = from_json(&res).unwrap();

        assert_eq!(
            stats.total_tokens_minted,
            Uint128::new(1000_000000000000000000)
        );
        assert_eq!(
            stats.total_house_funding,
            Uint128::new(1000_000000000000000000)
        );
        assert_eq!(stats.total_purchases, 0);
    }

    #[test]
    fn test_fund_house_unauthorized() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let treasury = deps.api.addr_make("treasury");
        let buyer = deps.api.addr_make("buyer");
        let game = deps.api.addr_make("game");
        setup_contract(deps.as_mut(), &admin, &treasury).unwrap();

        let msg = ExecuteMsg::FundHouse {
            game_contract: game.to_string(),
            amount: Uint128::new(1000_000000000000000000),
        };
        let info = message_info(&buyer, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::Unauthorized {}));
    }

    #[test]
    fn test_fund_house_zero_amount() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let treasury = deps.api.addr_make("treasury");
        let game = deps.api.addr_make("game");

        setup_contract(deps.as_mut(), &admin, &treasury).unwrap();

        let msg = ExecuteMsg::FundHouse {
            game_contract: game.to_string(),
            amount: Uint128::zero(),
        };
        let info = message_info(&admin, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::InvalidAmount {}));
    }

    #[test]
    fn test_fund_house_multiple_times() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let treasury = deps.api.addr_make("treasury");
        let game = deps.api.addr_make("game");
        setup_contract(deps.as_mut(), &admin, &treasury).unwrap();

        // First funding
        let msg = ExecuteMsg::FundHouse {
            game_contract: game.to_string(),
            amount: Uint128::new(500_000000000000000000),
        };
        let info = message_info(&admin, &[]);
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Second funding
        let msg = ExecuteMsg::FundHouse {
            game_contract: game.to_string(),
            amount: Uint128::new(300_000000000000000000),
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Check stats
        let query_msg = QueryMsg::Stats {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats: StatsResponse = from_json(&res).unwrap();

        assert_eq!(
            stats.total_tokens_minted,
            Uint128::new(800_000000000000000000)
        );
        assert_eq!(
            stats.total_house_funding,
            Uint128::new(800_000000000000000000)
        );
    }

    #[test]
    fn test_combined_purchases_and_funding() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let treasury = deps.api.addr_make("treasury");
        let buyer = deps.api.addr_make("buyer");

        let game = deps.api.addr_make("game");

        setup_contract(deps.as_mut(), &admin, &treasury).unwrap();

        // Purchase
        let msg = ExecuteMsg::Purchase {};
        let info = message_info(&buyer, &coins(10_000000000000000000, "inj"));
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Fund house
        let msg = ExecuteMsg::FundHouse {
            game_contract: game.to_string(),
            amount: Uint128::new(500_000000000000000000),
        };
        let info = message_info(&admin, &[]);
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Check stats
        let query_msg = QueryMsg::Stats {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats: StatsResponse = from_json(&res).unwrap();

        assert_eq!(
            stats.total_inj_received,
            Uint128::new(10_000000000000000000)
        );
        assert_eq!(
            stats.total_tokens_minted,
            Uint128::new(1500_000000000000000000)
        ); // 1000 from purchase + 500 from funding
        assert_eq!(stats.total_purchases, 1);
        assert_eq!(
            stats.total_house_funding,
            Uint128::new(500_000000000000000000)
        );
    }

    #[test]
    fn test_update_exchange_rate() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let treasury = deps.api.addr_make("treasury");
        let buyer = deps.api.addr_make("buyer");
        setup_contract(deps.as_mut(), &admin, &treasury).unwrap();

        let new_rate = Uint128::new(200);
        let msg = ExecuteMsg::UpdateExchangeRate { new_rate };
        let info = message_info(&admin, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(
            res.attributes,
            vec![("action", "update_exchange_rate"), ("new_rate", "200"),]
        );

        // Check config
        let query_msg = QueryMsg::Config {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();
        assert_eq!(config.exchange_rate, new_rate);

        // Test purchase with new rate
        let msg = ExecuteMsg::Purchase {};
        let info = message_info(&buyer, &coins(10_000000000000000000, "inj"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Should get 10 * 200 = 2000 tokens
        assert_eq!(
            res.attributes,
            vec![
                ("action", "purchase"),
                ("buyer", &buyer.to_string()),
                ("inj_amount", "10000000000000000000"),
                ("token_amount", "2000000000000000000000"),
            ]
        );
    }

    #[test]
    fn test_update_exchange_rate_unauthorized() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let treasury = deps.api.addr_make("treasury");
        let buyer = deps.api.addr_make("buyer");

        setup_contract(deps.as_mut(), &admin, &treasury).unwrap();

        let msg = ExecuteMsg::UpdateExchangeRate {
            new_rate: Uint128::new(200),
        };
        let info = message_info(&buyer, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::Unauthorized {}));
    }

    #[test]
    fn test_update_exchange_rate_zero() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let treasury = deps.api.addr_make("treasury");
        setup_contract(deps.as_mut(), &admin, &treasury).unwrap();

        let msg = ExecuteMsg::UpdateExchangeRate {
            new_rate: Uint128::zero(),
        };
        let info = message_info(&admin, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::InvalidExchangeRate {}));
    }

    #[test]
    fn test_update_treasury() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let treasury = deps.api.addr_make("treasury");
        let new_treasury = deps.api.addr_make("new_treasury");

        setup_contract(deps.as_mut(), &admin, &treasury).unwrap();

        let msg = ExecuteMsg::UpdateTreasury {
            new_treasury: new_treasury.to_string(),
        };
        let info = message_info(&admin, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(
            res.attributes,
            vec![
                ("action", "update_treasury"),
                ("new_treasury", &new_treasury.to_string()),
            ]
        );

        // Check config
        let query_msg = QueryMsg::Config {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();
        assert_eq!(config.treasury_address, Addr::unchecked(new_treasury));
    }

    #[test]
    fn test_update_treasury_unauthorized() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let treasury = deps.api.addr_make("treasury");
        let buyer = deps.api.addr_make("buyer");
        setup_contract(deps.as_mut(), &admin, &treasury).unwrap();

        let msg = ExecuteMsg::UpdateTreasury {
            new_treasury: "new_treasury".to_string(),
        };
        let info = message_info(&buyer, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, ContractError::Unauthorized {}));
    }

    #[test]
    fn test_preview_purchase() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let treasury = deps.api.addr_make("treasury");
        setup_contract(deps.as_mut(), &admin, &treasury).unwrap();

        let query_msg = QueryMsg::PreviewPurchase {
            inj_amount: Uint128::new(10_000000000000000000),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let preview: PreviewPurchaseResponse = from_json(&res).unwrap();

        assert_eq!(preview.inj_amount, Uint128::new(10_000000000000000000));
        assert_eq!(preview.token_amount, Uint128::new(1000_000000000000000000));
        assert_eq!(preview.exchange_rate, Uint128::new(100));
    }

    #[test]
    fn test_preview_purchase_different_amounts() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let treasury = deps.api.addr_make("treasury");
        setup_contract(deps.as_mut(), &admin, &treasury).unwrap();
        // Test various amounts
        let test_cases = vec![
            (1_000000000000000000u128, 100_000000000000000000u128),
            (5_000000000000000000u128, 500_000000000000000000u128),
            (100_000000000000000000u128, 10000_000000000000000000u128),
        ];

        for (inj_amount, expected_tokens) in test_cases {
            let query_msg = QueryMsg::PreviewPurchase {
                inj_amount: Uint128::new(inj_amount),
            };
            let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
            let preview: PreviewPurchaseResponse = from_json(&res).unwrap();

            assert_eq!(preview.token_amount, Uint128::new(expected_tokens));
        }
    }

    #[test]
    fn test_preview_purchase_after_rate_change() {
        let mut deps = mock_deps();
        let admin = deps.api.addr_make("admin");
        let treasury = deps.api.addr_make("treasury");
        setup_contract(deps.as_mut(), &admin, &treasury).unwrap();

        // Update exchange rate to 200
        let msg = ExecuteMsg::UpdateExchangeRate {
            new_rate: Uint128::new(200),
        };
        let info = message_info(&admin, &[]);
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Preview should reflect new rate
        let query_msg = QueryMsg::PreviewPurchase {
            inj_amount: Uint128::new(10_000000000000000000),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let preview: PreviewPurchaseResponse = from_json(&res).unwrap();

        assert_eq!(preview.token_amount, Uint128::new(2000_000000000000000000));
        assert_eq!(preview.exchange_rate, Uint128::new(200));
    }
}
