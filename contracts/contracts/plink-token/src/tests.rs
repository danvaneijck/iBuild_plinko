#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_json, Uint128};
    use cw20::{BalanceResponse, Cw20Coin, MinterResponse, TokenInfoResponse};
    use cw20_base::ContractError as Cw20ContractError;

    const CREATOR: &str = "creator";
    const MINTER: &str = "minter";
    const USER1: &str = "user1";
    const USER2: &str = "user2";

    fn setup_contract(deps: cosmwasm_std::DepsMut) -> Result<cosmwasm_std::Response, Cw20ContractError> {
        let msg = InstantiateMsg {
            name: "PLINK Token".to_string(),
            symbol: "PLINK".to_string(),
            decimals: 18,
            initial_balances: vec![Cw20Coin {
                address: CREATOR.to_string(),
                amount: Uint128::new(1000000),
            }],
            mint: Some(MinterResponse {
                minter: MINTER.to_string(),
                cap: None,
            }),
        };

        let info = mock_info(CREATOR, &[]);
        instantiate(deps, mock_env(), info, msg)
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let res = setup_contract(deps.as_mut()).unwrap();

        assert_eq!(res.messages.len(), 0);

        // Check token info
        let query_msg = QueryMsg::TokenInfo {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let token_info: TokenInfoResponse = from_json(&res).unwrap();

        assert_eq!(token_info.name, "PLINK Token");
        assert_eq!(token_info.symbol, "PLINK");
        assert_eq!(token_info.decimals, 18);
        assert_eq!(token_info.total_supply, Uint128::new(1000000));
    }

    #[test]
    fn test_minter_set() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        // Check minter
        let query_msg = QueryMsg::Minter {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let minter_response: MinterResponse = from_json(&res).unwrap();

        assert_eq!(minter_response.minter, MINTER);
        assert_eq!(minter_response.cap, None);
    }

    #[test]
    fn test_transfer() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        // Transfer tokens
        let msg = ExecuteMsg::Transfer {
            recipient: USER1.to_string(),
            amount: Uint128::new(100000),
        };
        let info = mock_info(CREATOR, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(res.messages.len(), 0);

        // Check balances
        let query_msg = QueryMsg::Balance {
            address: CREATOR.to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let balance: BalanceResponse = from_json(&res).unwrap();
        assert_eq!(balance.balance, Uint128::new(900000));

        let query_msg = QueryMsg::Balance {
            address: USER1.to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let balance: BalanceResponse = from_json(&res).unwrap();
        assert_eq!(balance.balance, Uint128::new(100000));
    }

    #[test]
    fn test_transfer_insufficient_balance() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        // Try to transfer more than balance
        let msg = ExecuteMsg::Transfer {
            recipient: USER1.to_string(),
            amount: Uint128::new(2000000),
        };
        let info = mock_info(CREATOR, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, Cw20ContractError::Std(_)));
    }

    #[test]
    fn test_mint() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        // Mint tokens
        let msg = ExecuteMsg::Mint {
            recipient: USER1.to_string(),
            amount: Uint128::new(500000),
        };
        let info = mock_info(MINTER, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(res.messages.len(), 0);

        // Check balance
        let query_msg = QueryMsg::Balance {
            address: USER1.to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let balance: BalanceResponse = from_json(&res).unwrap();
        assert_eq!(balance.balance, Uint128::new(500000));

        // Check total supply
        let query_msg = QueryMsg::TokenInfo {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let token_info: TokenInfoResponse = from_json(&res).unwrap();
        assert_eq!(token_info.total_supply, Uint128::new(1500000));
    }

    #[test]
    fn test_mint_unauthorized() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        // Try to mint from non-minter
        let msg = ExecuteMsg::Mint {
            recipient: USER1.to_string(),
            amount: Uint128::new(500000),
        };
        let info = mock_info(USER1, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert!(matches!(err, Cw20ContractError::Unauthorized {}));
    }

    #[test]
    fn test_burn() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        // Burn tokens
        let msg = ExecuteMsg::Burn {
            amount: Uint128::new(100000),
        };
        let info = mock_info(CREATOR, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(res.messages.len(), 0);

        // Check balance
        let query_msg = QueryMsg::Balance {
            address: CREATOR.to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let balance: BalanceResponse = from_json(&res).unwrap();
        assert_eq!(balance.balance, Uint128::new(900000));

        // Check total supply
        let query_msg = QueryMsg::TokenInfo {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let token_info: TokenInfoResponse = from_json(&res).unwrap();
        assert_eq!(token_info.total_supply, Uint128::new(900000));
    }

    #[test]
    fn test_allowance_flow() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        // Increase allowance
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: USER1.to_string(),
            amount: Uint128::new(50000),
            expires: None,
        };
        let info = mock_info(CREATOR, &[]);
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Check allowance
        let query_msg = QueryMsg::Allowance {
            owner: CREATOR.to_string(),
            spender: USER1.to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let allowance: cw20::AllowanceResponse = from_json(&res).unwrap();
        assert_eq!(allowance.allowance, Uint128::new(50000));

        // Transfer from
        let msg = ExecuteMsg::TransferFrom {
            owner: CREATOR.to_string(),
            recipient: USER2.to_string(),
            amount: Uint128::new(30000),
        };
        let info = mock_info(USER1, &[]);
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Check balances
        let query_msg = QueryMsg::Balance {
            address: CREATOR.to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let balance: BalanceResponse = from_json(&res).unwrap();
        assert_eq!(balance.balance, Uint128::new(970000));

        let query_msg = QueryMsg::Balance {
            address: USER2.to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let balance: BalanceResponse = from_json(&res).unwrap();
        assert_eq!(balance.balance, Uint128::new(30000));

        // Check remaining allowance
        let query_msg = QueryMsg::Allowance {
            owner: CREATOR.to_string(),
            spender: USER1.to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let allowance: cw20::AllowanceResponse = from_json(&res).unwrap();
        assert_eq!(allowance.allowance, Uint128::new(20000));
    }

    #[test]
    fn test_decrease_allowance() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut()).unwrap();

        // Increase allowance
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: USER1.to_string(),
            amount: Uint128::new(50000),
            expires: None,
        };
        let info = mock_info(CREATOR, &[]);
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Decrease allowance
        let msg = ExecuteMsg::DecreaseAllowance {
            spender: USER1.to_string(),
            amount: Uint128::new(20000),
            expires: None,
        };
        let info = mock_info(CREATOR, &[]);
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Check allowance
        let query_msg = QueryMsg::Allowance {
            owner: CREATOR.to_string(),
            spender: USER1.to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let allowance: cw20::AllowanceResponse = from_json(&res).unwrap();
        assert_eq!(allowance.allowance, Uint128::new(30000));
    }
}
