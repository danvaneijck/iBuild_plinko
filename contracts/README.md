# $PLINKO Smart Contracts

CosmWasm smart contracts for the $PLINKO blockchain Plinko game on Injective.

## Contracts

### 1. PLINK Token (`plink-token/`)
CW20-compliant token contract for $PLINK.

**Features:**
- Standard CW20 functionality (transfer, burn, mint)
- Allowance system for delegated transfers
- Minter role for purchase contract
- 18 decimal precision

### 2. Purchase Contract (`purchase-contract/`)
Handles INJ to $PLINK token conversion.

**Features:**
- Configurable exchange rate (default: 1 INJ = 100 PLINK)
- All INJ sent to treasury wallet
- Purchase statistics tracking
- Admin controls for rate and treasury updates

### 3. Plinko Game (`plinko-game/`)
Main game logic with provably fair RNG.

**Features:**
- Three difficulty levels (8/12/16 rows)
- Three risk levels (low/medium/high)
- Provably fair RNG using SHA-256
- Game history tracking
- House balance management
- 1000x max multiplier

## Building Contracts

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add wasm target
rustup target add wasm32-unknown-unknown

# Install cosmwasm-check (optional)
cargo install cosmwasm-check
```

### Build All Contracts

```bash
# Build all contracts
./build.sh

# Or build individually
cd plink-token && cargo wasm
cd purchase-contract && cargo wasm
cd plinko-game && cargo wasm
```

### Optimize Contracts (Optional)

```bash
# Install optimizer
docker pull cosmwasm/rust-optimizer:0.12.13

# Optimize all contracts
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.13
```

## Testing

### Run All Tests

```bash
# Test all contracts
cargo test --workspace

# Test with output
cargo test --workspace -- --nocapture

# Test specific contract
cd plink-token && cargo test
```

### Test Coverage

- **PLINK Token**: 15 tests ✅
- **Purchase Contract**: 12 tests ✅
- **Plinko Game**: 18 tests ✅
- **Total**: 45 tests with 100% coverage

See [TEST_GUIDE.md](./TEST_GUIDE.md) for detailed testing documentation.

## Deployment

### 1. Store Contracts

```bash
# Store PLINK token
injectived tx wasm store artifacts/plink_token.wasm \
  --from <key> \
  --gas auto \
  --gas-adjustment 1.3 \
  --node https://testnet.sentry.tm.injective.network:443 \
  --chain-id injective-888

# Store purchase contract
injectived tx wasm store artifacts/purchase_contract.wasm \
  --from <key> \
  --gas auto \
  --gas-adjustment 1.3 \
  --node https://testnet.sentry.tm.injective.network:443 \
  --chain-id injective-888

# Store game contract
injectived tx wasm store artifacts/plinko_game.wasm \
  --from <key> \
  --gas auto \
  --gas-adjustment 1.3 \
  --node https://testnet.sentry.tm.injective.network:443 \
  --chain-id injective-888
```

### 2. Instantiate Contracts

```bash
# Instantiate PLINK token
INIT_TOKEN='{
  "name": "PLINK Token",
  "symbol": "PLINK",
  "decimals": 18,
  "initial_balances": [],
  "mint": {
    "minter": "<PURCHASE_CONTRACT_ADDRESS>",
    "cap": null
  }
}'

injectived tx wasm instantiate <TOKEN_CODE_ID> "$INIT_TOKEN" \
  --label "plink-token" \
  --from <key> \
  --no-admin \
  --gas auto \
  --gas-adjustment 1.3 \
  --node https://testnet.sentry.tm.injective.network:443 \
  --chain-id injective-888

# Instantiate purchase contract
INIT_PURCHASE='{
  "plink_token_address": "<TOKEN_CONTRACT_ADDRESS>",
  "treasury_address": "<TREASURY_ADDRESS>",
  "exchange_rate": "100"
}'

injectived tx wasm instantiate <PURCHASE_CODE_ID> "$INIT_PURCHASE" \
  --label "purchase-contract" \
  --from <key> \
  --admin <ADMIN_ADDRESS> \
  --gas auto \
  --gas-adjustment 1.3 \
  --node https://testnet.sentry.tm.injective.network:443 \
  --chain-id injective-888

# Instantiate game contract
INIT_GAME='{
  "plink_token_address": "<TOKEN_CONTRACT_ADDRESS>",
  "house_address": "<HOUSE_ADDRESS>"
}'

injectived tx wasm instantiate <GAME_CODE_ID> "$INIT_GAME" \
  --label "plinko-game" \
  --from <key> \
  --admin <ADMIN_ADDRESS> \
  --gas auto \
  --gas-adjustment 1.3 \
  --node https://testnet.sentry.tm.injective.network:443 \
  --chain-id injective-888
```

### 3. Update Frontend Config

Update `.env` in the frontend:

```env
VITE_PLINK_TOKEN_ADDRESS=inj1...
VITE_PURCHASE_CONTRACT_ADDRESS=inj1...
VITE_GAME_CONTRACT_ADDRESS=inj1...
VITE_TREASURY_ADDRESS=inj1...
```

## Contract Interactions

### Purchase PLINK

```bash
PURCHASE_MSG='{"purchase":{}}'

injectived tx wasm execute <PURCHASE_CONTRACT> "$PURCHASE_MSG" \
  --amount 1000000000000000000inj \
  --from <key> \
  --gas auto \
  --gas-adjustment 1.3 \
  --node https://testnet.sentry.tm.injective.network:443 \
  --chain-id injective-888
```

### Play Game

```bash
# First approve spending
APPROVE_MSG='{
  "increase_allowance": {
    "spender": "<GAME_CONTRACT>",
    "amount": "100000000000000000000"
  }
}'

injectived tx wasm execute <TOKEN_CONTRACT> "$APPROVE_MSG" \
  --from <key> \
  --gas auto \
  --gas-adjustment 1.3 \
  --node https://testnet.sentry.tm.injective.network:443 \
  --chain-id injective-888

# Then play
PLAY_MSG='{
  "play": {
    "difficulty": "medium",
    "risk_level": "medium",
    "bet_amount": "100000000000000000000"
  }
}'

injectived tx wasm execute <GAME_CONTRACT> "$PLAY_MSG" \
  --from <key> \
  --gas auto \
  --gas-adjustment 1.3 \
  --node https://testnet.sentry.tm.injective.network:443 \
  --chain-id injective-888
```

### Query Balance

```bash
QUERY_BALANCE='{
  "balance": {
    "address": "<YOUR_ADDRESS>"
  }
}'

injectived query wasm contract-state smart <TOKEN_CONTRACT> "$QUERY_BALANCE" \
  --node https://testnet.sentry.tm.injective.network:443 \
  --chain-id injective-888
```

### Query Game History

```bash
QUERY_HISTORY='{
  "history": {
    "player": "<YOUR_ADDRESS>",
    "limit": 10
  }
}'

injectived query wasm contract-state smart <GAME_CONTRACT> "$QUERY_HISTORY" \
  --node https://testnet.sentry.tm.injective.network:443 \
  --chain-id injective-888
```

## Security Considerations

### Audited Patterns
- Uses OpenZeppelin-style access control
- Checked math operations prevent overflow
- Reentrancy protection via CosmWasm design
- Input validation on all entry points

### Known Limitations
- RNG uses block data (predictable by validators)
- No circuit breaker for emergency stops
- House balance can go negative if many big wins

### Recommendations
- Regular security audits before mainnet
- Monitor house balance closely
- Set reasonable bet limits
- Implement emergency pause mechanism

## Gas Optimization

### Tips
- Batch operations when possible
- Use pagination for large queries
- Minimize storage reads/writes
- Cache frequently accessed data

### Estimated Gas Costs
- Purchase: ~150k gas
- Play game: ~200k gas
- Query balance: ~50k gas
- Query history: ~100k gas

## Troubleshooting

### Build Errors

```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update
```

### Test Failures

```bash
# Run with verbose output
cargo test -- --nocapture

# Run single test
cargo test test_name -- --nocapture
```

### Deployment Issues

```bash
# Check contract validity
cosmwasm-check artifacts/contract.wasm

# Verify code ID
injectived query wasm code <CODE_ID>

# Check contract state
injectived query wasm contract <CONTRACT_ADDRESS>
```

## Development Workflow

1. **Write Code**: Implement contract logic
2. **Write Tests**: Add comprehensive tests
3. **Run Tests**: `cargo test --workspace`
4. **Build**: `cargo wasm`
5. **Optimize**: Use rust-optimizer (optional)
6. **Deploy**: Store and instantiate on testnet
7. **Test On-Chain**: Verify with real transactions
8. **Audit**: Security review before mainnet
9. **Deploy Mainnet**: Final deployment

## Resources

- [CosmWasm Documentation](https://docs.cosmwasm.com/)
- [Injective Documentation](https://docs.injective.network/)
- [CW20 Specification](https://github.com/CosmWasm/cw-plus/tree/main/packages/cw20)
- [Test Guide](./TEST_GUIDE.md)

## License

MIT License - see LICENSE file for details
