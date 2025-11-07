# $PLINKO - Blockchain Plinko Game

A decentralized Plinko game built on Injective blockchain with provably fair gameplay and native $PLINK token.

## Features

- 🎮 **Provably Fair Gameplay**: RNG using blockchain data (block height, timestamp, player nonce)
- 🪙 **Native Token**: $PLINK token (CW20-compliant)
- 💰 **1000x Max Multiplier**: On Hard difficulty, High risk edges
- 🎯 **Three Difficulty Levels**: Easy (8 rows), Medium (12 rows), Hard (16 rows)
- ⚡ **Three Risk Levels**: Low, Medium, High with different multiplier distributions
- 🔗 **Multi-Wallet Support**: Keplr, Leap, Metamask, Rabby
- 📊 **Game History**: Track all your plays on-chain
- 🏦 **Treasury Integration**: All INJ from purchases goes to treasury wallet

## Tech Stack

- **Frontend**: React, TypeScript, Vite, Tailwind CSS
- **Blockchain**: Injective (CosmWasm)
- **Smart Contracts**: Rust, CosmWasm 1.5
- **Wallet Integration**: @injectivelabs SDK

## Project Structure

```
├── contracts/              # CosmWasm smart contracts
│   ├── plink-token/       # CW20 token contract
│   ├── purchase-contract/ # INJ to PLINK conversion
│   └── plinko-game/       # Main game logic
├── src/
│   ├── components/        # React components
│   ├── hooks/            # Custom React hooks
│   ├── services/         # Contract interaction services
│   ├── config/           # Configuration files
│   └── types/            # TypeScript types
└── public/               # Static assets
```

## Getting Started

### Prerequisites

- Node.js 18+
- pnpm
- Rust toolchain (for building contracts)
- Injective wallet (Keplr, Leap, Metamask, or Rabby)

### Installation

1. **Clone and install dependencies**:
```bash
pnpm install
```

2. **Build smart contracts** (requires Rust):
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Build contracts
cd contracts/plink-token && cargo wasm
cd ../purchase-contract && cargo wasm
cd ../plinko-game && cargo wasm
```

3. **Deploy contracts** (see `contracts/README.md` for detailed instructions):
```bash
chmod +x contracts/deploy.sh
./contracts/deploy.sh
```

4. **Configure environment**:
```bash
cp .env.example .env
# Update .env with deployed contract addresses
```

5. **Start development server**:
```bash
pnpm dev
```

## Smart Contracts

### PLINK Token
- **Type**: CW20-compliant token
- **Decimals**: 18
- **Minter**: Purchase contract
- **Features**: Transfer, burn, mint, allowances

### Purchase Contract
- **Exchange Rate**: 1 INJ = 100 PLINK (configurable)
- **Treasury**: All INJ sent to treasury wallet
- **Features**: Purchase tracking, admin controls

### Plinko Game
- **RNG**: SHA-256(block_height + timestamp + player_address + nonce)
- **Multipliers**: Stored as fractions for precision
- **Features**: Game history, house balance, provably fair

## Multipliers

### Easy (8 rows)
- **Low Risk**: 5.6x, 2.1x, 1.1x, 1.0x, 0.5x, 1.0x, 1.1x, 2.1x, 5.6x
- **Medium Risk**: 13x, 3x, 1.3x, 0.7x, 0.4x, 0.7x, 1.3x, 3x, 13x
- **High Risk**: 29x, 4x, 1.5x, 0.3x, 0.2x, 0.3x, 1.5x, 4x, 29x

### Medium (12 rows)
- **Low Risk**: 8.9x, 3x, 1.4x, 1.1x, 1.0x, 0.5x, 1.0x, 1.1x, 1.4x, 3x, 8.9x
- **Medium Risk**: 18x, 4x, 1.7x, 0.9x, 0.5x, 0.3x, 0.5x, 0.9x, 1.7x, 4x, 18x
- **High Risk**: 43x, 7x, 2x, 0.6x, 0.2x, 0.2x, 0.2x, 0.6x, 2x, 7x, 43x

### Hard (16 rows)
- **Low Risk**: 16x, 9x, 2x, 1.4x, 1.1x, 1.0x, 0.5x, 1.0x, 1.1x, 1.4x, 2x, 9x, 16x
- **Medium Risk**: 110x, 41x, 10x, 5x, 3x, 1.5x, 1.0x, 0.5x, 1.0x, 1.5x, 3x, 5x, 10x, 41x, 110x
- **High Risk**: **1000x**, 130x, 26x, 9x, 4x, 2x, 0.2x, 0.2x, 0.2x, 2x, 4x, 9x, 26x, 130x, **1000x**

## How to Play

1. **Connect Wallet**: Click "Connect Wallet" and select your wallet
2. **Buy $PLINK**: Click "Buy $PLINK" and purchase tokens with INJ
3. **Configure Game**: Select difficulty and risk level
4. **Place Bet**: Enter bet amount in $PLINK
5. **Drop Ball**: Click "Drop Ball" to play
6. **Watch Result**: Ball drops through pegs to final multiplier
7. **Collect Winnings**: Winnings automatically sent to your wallet

## Provably Fair Verification

Every game result can be verified on-chain:

1. Check transaction on Injective Explorer
2. Verify RNG inputs: block height, timestamp, player address, nonce
3. Recalculate SHA-256 hash to verify ball path
4. Confirm multiplier matches bucket position

## Security

- **Audited Patterns**: Uses OpenZeppelin-style patterns
- **Checked Math**: All calculations use overflow protection
- **Access Control**: Admin functions protected
- **Transparent RNG**: Verifiable on-chain randomness

## Development

```bash
# Run development server
pnpm dev

# Build for production
pnpm build

# Run tests
pnpm test

# Lint code
pnpm lint
```

## Deployment

### Testnet
- Network: Injective Testnet
- Chain ID: injective-888
- RPC: https://testnet.sentry.tm.injective.network:443
- Faucet: https://testnet.faucet.injective.network/

### Mainnet
- Network: Injective Mainnet
- Chain ID: injective-1
- RPC: https://sentry.tm.injective.network:443

## Environment Variables

```env
VITE_NETWORK=testnet
VITE_CHAIN_ID=injective-888
VITE_RPC_URL=https://testnet.sentry.tm.injective.network:443
VITE_REST_URL=https://testnet.sentry.lcd.injective.network:443
VITE_PLINK_TOKEN_ADDRESS=inj1...
VITE_PURCHASE_CONTRACT_ADDRESS=inj1...
VITE_GAME_CONTRACT_ADDRESS=inj1...
VITE_TREASURY_ADDRESS=inj1...
VITE_EXCHANGE_RATE=100
```

## Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

## License

MIT License - see LICENSE file for details

## Support

- Documentation: See `contracts/README.md`
- Issues: GitHub Issues
- Discord: [Join our community]

## Acknowledgments

- Built on Injective blockchain
- Uses CosmWasm smart contracts
- Inspired by classic Plinko game
- Community-driven development
