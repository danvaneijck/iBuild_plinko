#!/bin/bash

# Plinko Game Deployment Script for Injective Testnet
# This script deploys all three contracts and configures them

set -e

# Configuration
CHAIN_ID="injective-888"
NODE="https://testnet.sentry.tm.injective.network:443"
KEY_NAME="your-key-name"  # Change this to your key name
TREASURY_ADDRESS="inj1..."  # Change this to your treasury address
EXCHANGE_RATE="100"  # 1 INJ = 100 PLINK

echo "🚀 Starting Plinko Game Deployment"
echo "=================================="
echo "Chain ID: $CHAIN_ID"
echo "Node: $NODE"
echo "Key: $KEY_NAME"
echo "Treasury: $TREASURY_ADDRESS"
echo ""

# Get deployer address
DEPLOYER=$(injectived keys show $KEY_NAME -a)
echo "Deployer: $DEPLOYER"
echo ""

# 1. Deploy PLINK Token
echo "📦 Deploying PLINK Token Contract..."
PLINK_TX=$(injectived tx wasm store contracts/plink-token/target/wasm32-unknown-unknown/release/plink_token.wasm \
  --from $KEY_NAME \
  --gas auto \
  --gas-adjustment 1.3 \
  --node $NODE \
  --chain-id $CHAIN_ID \
  --yes \
  --output json)

PLINK_CODE_ID=$(echo $PLINK_TX | jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
echo "✅ PLINK Token Code ID: $PLINK_CODE_ID"

# 2. Instantiate PLINK Token
echo "🎯 Instantiating PLINK Token..."
PLINK_INIT_TX=$(injectived tx wasm instantiate $PLINK_CODE_ID \
  "{\"name\":\"Plink Token\",\"symbol\":\"PLINK\",\"decimals\":18,\"initial_balances\":[],\"mint\":{\"minter\":\"$DEPLOYER\",\"cap\":null}}" \
  --label "plink-token" \
  --from $KEY_NAME \
  --admin $DEPLOYER \
  --gas auto \
  --gas-adjustment 1.3 \
  --node $NODE \
  --chain-id $CHAIN_ID \
  --yes \
  --output json)

PLINK_TOKEN_ADDRESS=$(echo $PLINK_INIT_TX | jq -r '.logs[0].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')
echo "✅ PLINK Token Address: $PLINK_TOKEN_ADDRESS"
echo ""

# 3. Deploy Purchase Contract
echo "📦 Deploying Purchase Contract..."
PURCHASE_TX=$(injectived tx wasm store contracts/purchase-contract/target/wasm32-unknown-unknown/release/purchase_contract.wasm \
  --from $KEY_NAME \
  --gas auto \
  --gas-adjustment 1.3 \
  --node $NODE \
  --chain-id $CHAIN_ID \
  --yes \
  --output json)

PURCHASE_CODE_ID=$(echo $PURCHASE_TX | jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
echo "✅ Purchase Contract Code ID: $PURCHASE_CODE_ID"

# 4. Instantiate Purchase Contract
echo "🎯 Instantiating Purchase Contract..."
PURCHASE_INIT_TX=$(injectived tx wasm instantiate $PURCHASE_CODE_ID \
  "{\"plink_token_address\":\"$PLINK_TOKEN_ADDRESS\",\"treasury_address\":\"$TREASURY_ADDRESS\",\"exchange_rate\":\"$EXCHANGE_RATE\"}" \
  --label "purchase-contract" \
  --from $KEY_NAME \
  --admin $DEPLOYER \
  --gas auto \
  --gas-adjustment 1.3 \
  --node $NODE \
  --chain-id $CHAIN_ID \
  --yes \
  --output json)

PURCHASE_CONTRACT_ADDRESS=$(echo $PURCHASE_INIT_TX | jq -r '.logs[0].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')
echo "✅ Purchase Contract Address: $PURCHASE_CONTRACT_ADDRESS"
echo ""

# 5. Update PLINK minter to Purchase Contract
echo "🔧 Setting Purchase Contract as PLINK minter..."
injectived tx wasm execute $PLINK_TOKEN_ADDRESS \
  "{\"update_minter\":{\"new_minter\":\"$PURCHASE_CONTRACT_ADDRESS\"}}" \
  --from $KEY_NAME \
  --gas auto \
  --gas-adjustment 1.3 \
  --node $NODE \
  --chain-id $CHAIN_ID \
  --yes
echo "✅ Minter updated"
echo ""

# 6. Deploy Plinko Game Contract
echo "📦 Deploying Plinko Game Contract..."
GAME_TX=$(injectived tx wasm store contracts/plinko-game/target/wasm32-unknown-unknown/release/plinko_game.wasm \
  --from $KEY_NAME \
  --gas auto \
  --gas-adjustment 1.3 \
  --node $NODE \
  --chain-id $CHAIN_ID \
  --yes \
  --output json)

GAME_CODE_ID=$(echo $GAME_TX | jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
echo "✅ Plinko Game Code ID: $GAME_CODE_ID"

# 7. Instantiate Plinko Game
echo "🎯 Instantiating Plinko Game..."
GAME_INIT_TX=$(injectived tx wasm instantiate $GAME_CODE_ID \
  "{\"plink_token_address\":\"$PLINK_TOKEN_ADDRESS\",\"house_address\":\"$DEPLOYER\"}" \
  --label "plinko-game" \
  --from $KEY_NAME \
  --admin $DEPLOYER \
  --gas auto \
  --gas-adjustment 1.3 \
  --node $NODE \
  --chain-id $CHAIN_ID \
  --yes \
  --output json)

GAME_CONTRACT_ADDRESS=$(echo $GAME_INIT_TX | jq -r '.logs[0].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')
echo "✅ Plinko Game Address: $GAME_CONTRACT_ADDRESS"
echo ""

# 8. Generate .env file
echo "📝 Generating .env file..."
cat > .env.deployed << EOF
# Plinko Game Contract Addresses - Injective Testnet
VITE_PLINK_TOKEN_ADDRESS=$PLINK_TOKEN_ADDRESS
VITE_PURCHASE_CONTRACT_ADDRESS=$PURCHASE_CONTRACT_ADDRESS
VITE_GAME_CONTRACT_ADDRESS=$GAME_CONTRACT_ADDRESS
VITE_TREASURY_ADDRESS=$TREASURY_ADDRESS
VITE_CHAIN_ID=$CHAIN_ID
VITE_RPC_URL=$NODE
VITE_EXCHANGE_RATE=$EXCHANGE_RATE
EOF

echo "✅ Deployment complete!"
echo ""
echo "📋 Summary:"
echo "==========="
echo "PLINK Token: $PLINK_TOKEN_ADDRESS"
echo "Purchase Contract: $PURCHASE_CONTRACT_ADDRESS"
echo "Plinko Game: $GAME_CONTRACT_ADDRESS"
echo ""
echo "Environment variables saved to .env.deployed"
echo "Copy these to your frontend .env file"
