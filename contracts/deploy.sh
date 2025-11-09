#!/bin/bash
set -euo pipefail

NODE="https://testnet.sentry.tm.injective.network:443"
CHAIN_ID="injective-888"
FEES="2000000000000000inj"
GAS="4000000"
KEY_NAME="testnet"
PASSWORD="12345678" 

TREASURY_ADDRESS="inj1q2m26a7jdzjyfdn545vqsude3zwwtfrdap5jgz"
DEPLOYER=$TREASURY_ADDRESS

INITIAL_HOUSE_FUND="1000000000000000000000000000" # 1,000,000 tokens with 18 decimals
DENOM_CREATION_FEE="1000000000000000000inj" # 1 INJ

store_contract() {
    local wasm_file="$1"
    echo "📦 Storing contract: $wasm_file..." >&2
    
    tx_output=$(yes "$PASSWORD" | injectived tx wasm store "$wasm_file" \
      --from="$KEY_NAME" \
      --chain-id="$CHAIN_ID" \
      --yes --fees="$FEES" --gas="$GAS" \
      --node="$NODE" 2>&1)
    
    if echo "$tx_output" | grep -q 'error:'; then
        echo "❌ ERROR storing contract: $tx_output" >&2
        exit 1
    fi
      
    txhash=$(echo "$tx_output" | grep -o 'txhash: [A-F0-9]*' | awk '{print $2}')
    echo "  - Transaction hash: $txhash" >&2
    
    sleep 6
    
    query_output=$(injectived query tx "$txhash" --node="$NODE")
    code_id=$(echo "$query_output" | grep -A 1 'key: code_id' | grep 'value:' | head -1 | sed 's/.*value: "\(.*\)".*/\1/')
    
    if [ -z "$code_id" ]; then
        echo "❌ ERROR: Failed to retrieve code_id for txhash: $txhash." >&2
        exit 1
    fi
    echo "$code_id"
}

instantiate_contract() {
    local code_id="$1"
    local init_msg="$2"
    local label="$3"
    local admin="${4:-}"
    local amount="${5:-}" # <-- Add amount as an optional 5th argument

    # Build the core command
    local cmd_args=(
        "tx" "wasm" "instantiate" "$code_id" "$init_msg"
        "--label=$label"
        "--from=$KEY_NAME"
        "--chain-id=$CHAIN_ID"
        "--yes" "--fees=$FEES" "--gas=$GAS"
        "--node=$NODE"
    )

    # Add admin flag if provided
    if [ -z "$admin" ]; then
        cmd_args+=("--no-admin")
    else
        cmd_args+=("--admin=$admin")
    fi

    # Add amount flag if provided
    if [ -n "$amount" ]; then
        cmd_args+=("--amount=$amount")
    fi

    tx_output=$(yes $PASSWORD | injectived "${cmd_args[@]}" 2>&1)

    # (The rest of the function remains the same)
    txhash=$(echo "$tx_output" | grep -o 'txhash: [A-F0-9]*' | awk '{print $2}')
    sleep 6 # Increased sleep to ensure tx is indexed
    query_output=$(injectived query tx "$txhash" --node="$NODE")
    contract_address=$(echo "$query_output" \
    | grep -A 1 'key: contract_address' \
    | grep 'value:' \
    | head -1 \
    | sed "s/.*value: //; s/['\"]//g")
    
    if [ -z "$contract_address" ]; then
        echo "❌ ERROR: Failed to retrieve contract_address for txhash: $txhash." >&2
        echo "Full query output: $query_output" >&2
        exit 1
    fi

    echo "$contract_address"
}

execute_contract() {
    local contract_address="$1"
    local exec_msg="$2"
    echo "🔧 Executing message on contract: $contract_address..." >&2

    tx_output=$(yes "$PASSWORD" | injectived tx wasm execute "$contract_address" "$exec_msg" \
    --from="$KEY_NAME" \
    --chain-id="$CHAIN_ID" \
    --yes --fees="$FEES" --gas="$GAS" \
    --node="$NODE" 2>&1) || true
    
    txhash=$(echo "$tx_output" | grep -o 'txhash: [A-F0-9]*' | awk '{print $2}')

    if echo "$tx_output" | grep -q 'error:'; then
        echo "❌ ERROR executing contract: $tx_output" >&2
        exit 1
    fi
    
    txhash=$(echo "$tx_output" | grep -o 'txhash: [A-F0-9]*' | awk '{print $2}')
    echo "  - Transaction hash: $txhash" >&2
    sleep 6
}


# --- Main Deployment Logic ---

echo "🚀 Starting Plinko Game Deployment (Token Factory Version)"
echo "========================================================"
echo "Chain ID:         $CHAIN_ID"
echo "Node:             $NODE"
echo "Deployer Key:     $KEY_NAME"
echo "Deployer Address: $DEPLOYER"
echo "Treasury Address: $TREASURY_ADDRESS"
echo ""

# 1. Store the Purchase and Game contract code
PURCHASE_CODE_ID=$(store_contract "artifacts/purchase_contract.wasm")
echo "✅ Purchase Contract Code ID: $PURCHASE_CODE_ID"
echo ""
GAME_CODE_ID=$(store_contract "artifacts/plinko_game.wasm")
echo "✅ Plinko Game Code ID: $GAME_CODE_ID"
echo ""

# 2. Instantiate the Purchase Contract (which creates the token)
INIT_PURCHASE=$(cat <<EOF
{
  "treasury_address": "$TREASURY_ADDRESS",
  "exchange_rate": "100",
  "token_name": "Plink Token",
  "token_symbol": "PLINK",
  "token_decimals": 18,
  "subdenom": "plink"
}
EOF
)
PURCHASE_CONTRACT_ADDRESS=$(instantiate_contract "$PURCHASE_CODE_ID" "$INIT_PURCHASE" "purchase-contract" "$DEPLOYER" "$DENOM_CREATION_FEE")
echo "✅ Purchase Contract Address: $PURCHASE_CONTRACT_ADDRESS"
echo ""

# The token denom is now factory/{purchase_contract_address}/plink
TOKEN_DENOM="factory/$PURCHASE_CONTRACT_ADDRESS/plink"
echo "ℹ️  Token Denom: $TOKEN_DENOM"
echo ""

# 3. Instantiate the Game Contract
INIT_GAME=$(cat <<EOF
{
  "token_denom": "$TOKEN_DENOM",
  "funder_address": "$PURCHASE_CONTRACT_ADDRESS"
}
EOF
)
GAME_CONTRACT_ADDRESS=$(instantiate_contract "$GAME_CODE_ID" "$INIT_GAME" "plinko-game" "$DEPLOYER")
echo "✅ Plinko Game Address: $GAME_CONTRACT_ADDRESS"
echo ""

# 4. Fund the Game Contract (House) from the Purchase Contract
FUND_HOUSE_MSG=$(cat <<EOF
{
  "fund_house": {
    "game_contract": "$GAME_CONTRACT_ADDRESS",
    "amount": "$INITIAL_HOUSE_FUND"
  }
}
EOF
)
execute_contract "$PURCHASE_CONTRACT_ADDRESS" "$FUND_HOUSE_MSG"
echo "✅ Successfully funded the Plinko Game contract with $INITIAL_HOUSE_FUND $TOKEN_DENOM"
echo ""


echo "✅ Deployment complete!"
echo ""
echo "📋 Summary:"
echo "==========="
echo "Token Denom:       $TOKEN_DENOM"
echo "Purchase Contract: $PURCHASE_CONTRACT_ADDRESS"
echo "Plinko Game:       $GAME_CONTRACT_ADDRESS"