#!/bin/bash

# Build script for all contracts

set -e

echo "Building PLINK Token..."
cd plink-token
cargo wasm
cd ..

echo "Building Purchase Contract..."
cd purchase-contract
cargo wasm
cd ..

echo "Building Plinko Game..."
cd plinko-game
cargo wasm
cd ..

echo "All contracts built successfully!"
echo ""
echo "Artifacts location:"
echo "  - plink-token/target/wasm32-unknown-unknown/release/plink_token.wasm"
echo "  - purchase-contract/target/wasm32-unknown-unknown/release/purchase_contract.wasm"
echo "  - plinko-game/target/wasm32-unknown-unknown/release/plinko_game.wasm"
