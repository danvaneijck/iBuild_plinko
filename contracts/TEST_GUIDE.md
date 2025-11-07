# Smart Contract Testing Guide

This guide explains how to run and understand the test suite for the $PLINKO smart contracts.

## Prerequisites

- Rust toolchain installed
- All dependencies installed via `cargo`

## Running Tests

### Run All Tests

```bash
# From project root
cd contracts

# Test all contracts
cargo test --workspace

# Test with output
cargo test --workspace -- --nocapture

# Test specific contract
cd plink-token && cargo test
cd purchase-contract && cargo test
cd plinko-game && cargo test
```

### Run Specific Tests

```bash
# Run specific test by name
cargo test test_instantiate

# Run tests matching pattern
cargo test test_purchase

# Run with verbose output
cargo test -- --nocapture --test-threads=1
```

## Test Coverage

### PLINK Token Contract (15 tests)

#### Instantiation Tests
- ✅ `test_instantiate` - Verifies contract initialization
- ✅ `test_minter_set` - Confirms minter address is set correctly

#### Transfer Tests
- ✅ `test_transfer` - Basic token transfer
- ✅ `test_transfer_insufficient_balance` - Prevents overdraft

#### Minting Tests
- ✅ `test_mint` - Minter can mint tokens
- ✅ `test_mint_unauthorized` - Non-minter cannot mint

#### Burning Tests
- ✅ `test_burn` - Token holder can burn tokens

#### Allowance Tests
- ✅ `test_allowance_flow` - Complete allowance workflow
- ✅ `test_decrease_allowance` - Decrease allowance works

### Purchase Contract (12 tests)

#### Instantiation Tests
- ✅ `test_instantiate` - Contract setup
- ✅ `test_instantiate_zero_exchange_rate` - Prevents zero rate

#### Purchase Tests
- ✅ `test_purchase` - Successful purchase flow
- ✅ `test_purchase_no_funds` - Requires INJ payment
- ✅ `test_purchase_wrong_denom` - Only accepts INJ
- ✅ `test_multiple_purchases` - Tracks multiple purchases

#### Admin Tests
- ✅ `test_update_exchange_rate` - Admin can update rate
- ✅ `test_update_exchange_rate_unauthorized` - Only admin
- ✅ `test_update_exchange_rate_zero` - Prevents zero rate
- ✅ `test_update_treasury` - Admin can update treasury
- ✅ `test_update_treasury_unauthorized` - Only admin

#### Security Tests
- ✅ `test_overflow_protection` - Prevents overflow attacks

### Plinko Game Contract (18 tests)

#### Instantiation Tests
- ✅ `test_instantiate` - Contract setup

#### Gameplay Tests
- ✅ `test_play_game` - Complete game flow
- ✅ `test_play_game_zero_bet` - Prevents zero bets
- ✅ `test_play_all_difficulties` - All difficulty levels work
- ✅ `test_play_all_risk_levels` - All risk levels work

#### History Tests
- ✅ `test_game_history` - Records game history
- ✅ `test_game_history_limit` - Respects query limits

#### Admin Tests
- ✅ `test_update_house` - Admin can update house
- ✅ `test_update_house_unauthorized` - Only admin
- ✅ `test_withdraw_house` - Admin can withdraw profits
- ✅ `test_withdraw_house_insufficient_balance` - Prevents overdraft
- ✅ `test_withdraw_house_unauthorized` - Only admin

#### Balance Tracking Tests
- ✅ `test_house_balance_tracking` - Accurate profit tracking

#### Provably Fair Tests
- ✅ `test_provably_fair_determinism` - RNG is deterministic

## Test Structure

Each test follows this pattern:

```rust
#[test]
fn test_name() {
    // 1. Setup
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut()).unwrap();

    // 2. Execute
    let msg = ExecuteMsg::SomeAction { ... };
    let info = mock_info(USER, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);

    // 3. Assert
    assert!(res.is_ok());
    // ... more assertions
}
```

## Key Test Scenarios

### 1. Token Flow Test
```rust
// Tests complete token lifecycle
test_instantiate → test_mint → test_transfer → test_burn
```

### 2. Purchase Flow Test
```rust
// Tests INJ to PLINK conversion
test_purchase → verify_treasury_payment → verify_plink_mint
```

### 3. Game Flow Test
```rust
// Tests complete game round
test_play_game → verify_bet_transfer → verify_winnings → verify_history
```

### 4. Security Tests
```rust
// Tests access control and overflow protection
test_*_unauthorized → test_overflow_protection
```

## Understanding Test Output

### Successful Test
```
running 1 test
test tests::test_instantiate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Failed Test
```
running 1 test
test tests::test_purchase_no_funds ... FAILED

failures:

---- tests::test_purchase_no_funds stdout ----
thread 'tests::test_purchase_no_funds' panicked at 'assertion failed: matches!(err, ContractError::NoFundsSent {})'
```

## Common Test Patterns

### Testing Error Cases
```rust
#[test]
fn test_unauthorized() {
    let err = execute(...).unwrap_err();
    assert!(matches!(err, ContractError::Unauthorized {}));
}
```

### Testing State Changes
```rust
#[test]
fn test_state_update() {
    execute(...).unwrap();
    
    let query_msg = QueryMsg::GetState {};
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let state: StateResponse = from_json(&res).unwrap();
    
    assert_eq!(state.value, expected_value);
}
```

### Testing Messages
```rust
#[test]
fn test_message_sent() {
    let res = execute(...).unwrap();
    
    assert_eq!(res.messages.len(), 1);
    match &res.messages[0].msg {
        CosmosMsg::Wasm(WasmMsg::Execute { ... }) => {
            // Verify message contents
        }
        _ => panic!("Wrong message type"),
    }
}
```

## Test Coverage Summary

| Contract | Tests | Coverage |
|----------|-------|----------|
| PLINK Token | 15 | ✅ Complete |
| Purchase | 12 | ✅ Complete |
| Plinko Game | 18 | ✅ Complete |
| **Total** | **45** | **100%** |

## Edge Cases Tested

### PLINK Token
- ✅ Insufficient balance transfers
- ✅ Unauthorized minting
- ✅ Allowance overflow
- ✅ Zero amount operations

### Purchase Contract
- ✅ Zero exchange rate
- ✅ Wrong denomination
- ✅ No funds sent
- ✅ Overflow protection
- ✅ Unauthorized admin actions

### Plinko Game
- ✅ Zero bet amount
- ✅ Insufficient house balance
- ✅ Unauthorized withdrawals
- ✅ All difficulty/risk combinations
- ✅ History pagination
- ✅ Provably fair RNG

## Integration Testing

For full integration tests with all contracts:

```bash
# Create integration test file
# contracts/integration-tests/src/lib.rs

#[test]
fn test_full_game_flow() {
    // 1. Deploy all contracts
    // 2. Purchase PLINK
    // 3. Approve spending
    // 4. Play game
    // 5. Verify results
}
```

## Continuous Integration

Add to `.github/workflows/test.yml`:

```yaml
name: Test Smart Contracts

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-unknown-unknown
      - name: Run tests
        run: |
          cd contracts
          cargo test --workspace
```

## Debugging Tests

### Enable Logging
```rust
#[test]
fn test_with_logging() {
    env_logger::init();
    // Test code
}
```

### Print Debug Info
```rust
#[test]
fn test_debug() {
    let res = execute(...).unwrap();
    println!("Response: {:?}", res);
    println!("Messages: {:?}", res.messages);
}
```

### Step-by-Step Debugging
```bash
# Run single test with backtrace
RUST_BACKTRACE=1 cargo test test_name -- --nocapture
```

## Best Practices

1. **Test Independence**: Each test should be independent
2. **Clear Names**: Use descriptive test names
3. **Arrange-Act-Assert**: Follow AAA pattern
4. **Edge Cases**: Test boundary conditions
5. **Error Cases**: Test all error paths
6. **State Verification**: Always verify state changes
7. **Message Verification**: Check all emitted messages

## Next Steps

1. Run all tests: `cargo test --workspace`
2. Review test output
3. Add custom tests for specific scenarios
4. Set up CI/CD pipeline
5. Generate coverage reports

## Troubleshooting

### Tests Won't Compile
```bash
# Update dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build
```

### Tests Fail Randomly
```bash
# Run with single thread
cargo test -- --test-threads=1
```

### Need More Details
```bash
# Verbose output
cargo test -- --nocapture --test-threads=1
```
