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

## Quick Start

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add wasm target
rustup target add wasm32-unknown-unknown

# Install Docker (for optimized builds)
# Visit: https://docs.docker.com/get-docker/

# Optional: Install cosmwasm-check
cargo install cosmwasm-check
```

### Development Build

```bash
# Build all contracts (development)
./build.sh

# Or build individually
cd plink-token && cargo wasm
cd purchase-contract && cargo wasm
cd plinko-game && cargo wasm
```

### Production Build (Optimized)

```bash
# Build optimized contracts for deployment
./build_release.sh
```

This will:
- Use Docker workspace optimizer for minimal WASM size
- Generate production-ready artifacts in `artifacts/`
- Validate contracts with cosmwasm-check (if installed)
- Show file sizes and locations

**Output:**
```
artifacts/
├── plink_token.wasm          (~150KB optimized)
├── purchase_contract.wasm    (~120KB optimized)
└── plinko_game.wasm          (~180KB optimized)
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
- **Plinko Game**: 21 tests ✅
- **Total**: 48 tests with 100% coverage

See [TEST_GUIDE.md](./TEST_GUIDE.md) for detailed testing documentation.

## Deployment

### 1. Build Optimized Contracts

```bash
./build_release.sh
```

### 2. Deploy to Testnet

```bash
# Edit deploy.sh with your configuration
nano deploy.sh

# Set these variables:
# - KEY_NAME: Your Injective key name
# - TREASURY_ADDRESS: Your treasury wallet address
# - EXCHANGE_RATE: INJ to PLINK rate (default: 100)

# Run deployment
./deploy.sh
```

The deployment script will:
1. Store all three contracts on-chain
2. Instantiate them with proper configuration
3. Link contracts together (set minter, etc.)
4. Generate `.env.deployed` with contract addresses

### 3. Update Frontend

```bash
# Copy deployed addresses to frontend
cp .env.deployed ../frontend/.env
```

## Manual Deployment

### Store Contracts

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

### Instantiate Contracts

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

## Build Comparison

| Build Type | Command | Size | Use Case |
|------------|---------|------|----------|
| Development | `./build.sh` | ~500KB | Local testing |
| Optimized | `./build_release.sh` | ~150KB | Production deployment |

**Why optimize?**
- 70% smaller file size
- Lower gas costs for deployment
- Faster contract execution
- Required for mainnet deployment

## Troubleshooting

### Docker Issues

```bash
# Check Docker is running
docker info

# Pull optimizer image manually
docker pull cosmwasm/workspace-optimizer:0.17.0

# Clean Docker volumes
docker volume prune
```

### Build Errors

```bash
# Clean and rebuild
cargo clean
./build_release.sh

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

## Development Workflow

1. **Write Code**: Implement contract logic
2. **Write Tests**: Add comprehensive tests
3. **Run Tests**: `cargo test --workspace`
4. **Dev Build**: `./build.sh` (quick iteration)
5. **Test On-Chain**: Deploy to testnet
6. **Optimize**: `./build_release.sh` (production)
7. **Audit**: Security review
8. **Deploy**: Mainnet deployment

## Resources

- [CosmWasm Documentation](https://docs.cosmwasm.com/)
- [Injective Documentation](https://docs.injective.network/)
- [CW20 Specification](https://github.com/CosmWasm/cw-plus/tree/main/packages/cw20)
- [Workspace Optimizer](https://github.com/CosmWasm/rust-optimizer)
- [Test Guide](./TEST_GUIDE.md)

## License

MIT License - see LICENSE file for details
