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

INITIAL_HOUSE_FUND="1000000000000000000000000"


store_contract() {
    local wasm_file="$1"
    echo "📦 Storing contract: $wasm_file..." >&2
    
    tx_output=$(yes "$PASSWORD" | injectived tx wasm store "$wasm_file" \
      --from="$KEY_NAME" \
      --chain-id="$CHAIN_ID" \
      --yes --fees="$FEES" --gas="$GAS" \
      --node="$NODE" 2>&1)
    
    # **IMPROVED ERROR HANDLING**
    if echo "$tx_output" | grep -q 'error:'; then
        echo "❌ ERROR storing contract: $tx_output" >&2
        exit 1
    fi
      
    # **CORRECTED REGEX**
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
    if [ -z "$admin" ]; then
        tx_output=$(yes $PASSWORD | injectived tx wasm instantiate "$code_id" "$init_msg" \
          --label="$label" \
          --no-admin \
          --from="$KEY_NAME" \
          --chain-id="$CHAIN_ID" \
          --yes --fees="$FEES" --gas="$GAS" \
          --node="$NODE" 2>&1)
    else
        tx_output=$(yes $PASSWORD | injectived tx wasm instantiate "$code_id" "$init_msg" \
          --label="$label" \
          --admin="$admin" \
          --from="$KEY_NAME" \
          --chain-id="$CHAIN_ID" \
          --yes --fees="$FEES" --gas="$GAS" \
          --node="$NODE" 2>&1)
    fi
    # Extract the txhash from the tx output.
    txhash=$(echo "$tx_output" | grep -o 'txhash: [A-F0-9]*' | awk '{print $2}')
    sleep 1
    query_output=$(injectived query tx "$txhash" --node="$NODE")
    # Extract the contract address from the query output.
    contract_address=$(echo "$query_output" \
    | grep -A 1 'key: contract_address' \
    | grep 'value:' \
    | head -1 \
    | sed "s/.*value: //; s/['\"]//g")
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
    
    # **CORRECTED REGEX**
    txhash=$(echo "$tx_output" | grep -o 'txhash: [A-F0-9]*' | awk '{print $2}')
    echo "  - Transaction hash: $txhash" >&2
    sleep 6
}


# --- Main Deployment Logic ---
# (This part remains the same)

echo "🚀 Starting Plinko Game Deployment"
echo "=================================="
echo "Chain ID:         $CHAIN_ID"
echo "Node:             $NODE"
echo "Deployer Key:     $KEY_NAME"
echo "Deployer Address: $DEPLOYER"
echo "Treasury Address: $TREASURY_ADDRESS"
echo ""

PLINK_CODE_ID=$(store_contract "artifacts/plink_token.wasm")
echo "✅ PLINK Token Code ID: $PLINK_CODE_ID"
echo ""

INIT_PLINK=$(cat <<EOF
{
  "name": "Plink Token",
  "symbol": "PLINK",
  "decimals": 18,
  "initial_balances": [],
  "mint": {
    "minter": "$DEPLOYER",
    "cap": null
  }
}
EOF
)
PLINK_TOKEN_ADDRESS=$(instantiate_contract "$PLINK_CODE_ID" "$INIT_PLINK" "plink-token" "$DEPLOYER")
echo "✅ PLINK Token Address: $PLINK_TOKEN_ADDRESS"
echo ""

MINT_FOR_HOUSE_MSG="{\"mint\":{\"recipient\":\"$DEPLOYER\",\"amount\":\"$INITIAL_HOUSE_FUND\"}}"
execute_contract "$PLINK_TOKEN_ADDRESS" "$MINT_FOR_HOUSE_MSG"
echo "✅ Minted $INITIAL_HOUSE_FUND PLINK to deployer account"
echo ""

PURCHASE_CODE_ID=$(store_contract "artifacts/purchase_contract.wasm")
echo "✅ Purchase Contract Code ID: $PURCHASE_CODE_ID"
echo ""

INIT_PURCHASE=$(cat <<EOF
{
  "plink_token_address": "$PLINK_TOKEN_ADDRESS",
  "treasury_address": "$TREASURY_ADDRESS",
  "exchange_rate": "100"
}
EOF
)
PURCHASE_CONTRACT_ADDRESS=$(instantiate_contract "$PURCHASE_CODE_ID" "$INIT_PURCHASE" "purchase-contract" "$DEPLOYER")
echo "✅ Purchase Contract Address: $PURCHASE_CONTRACT_ADDRESS"
echo ""

UPDATE_MINTER_MSG="{\"update_minter\":{\"new_minter\":\"$PURCHASE_CONTRACT_ADDRESS\"}}"
execute_contract "$PLINK_TOKEN_ADDRESS" "$UPDATE_MINTER_MSG"
echo "✅ PLINK Token minter updated to Purchase Contract"
echo ""

GAME_CODE_ID=$(store_contract "artifacts/plinko_game.wasm")
echo "✅ Plinko Game Code ID: $GAME_CODE_ID"
echo ""

INIT_GAME=$(cat <<EOF
{
  "plink_token_address": "$PLINK_TOKEN_ADDRESS",
  "house_address": "$DEPLOYER"
}
EOF
)
GAME_CONTRACT_ADDRESS=$(instantiate_contract "$GAME_CODE_ID" "$INIT_GAME" "plinko-game" "$DEPLOYER")
echo "✅ Plinko Game Address: $GAME_CONTRACT_ADDRESS"
echo ""

INCREASE_ALLOWANCE_MSG="{\"increase_allowance\":{\"spender\":\"$GAME_CONTRACT_ADDRESS\",\"amount\":\"$INITIAL_HOUSE_FUND\"}}"
execute_contract "$PLINK_TOKEN_ADDRESS" "$INCREASE_ALLOWANCE_MSG"
echo "✅ Approved Plinko Game to spend $INITIAL_HOUSE_FUND PLINK"
echo ""

## ==> Step 3: Execute the FundHouse message on the Game Contract to transfer the tokens.
FUND_HOUSE_MSG="{\"fund_house\":{\"amount\":\"$INITIAL_HOUSE_FUND\"}}"
execute_contract "$GAME_CONTRACT_ADDRESS" "$FUND_HOUSE_MSG"
echo "✅ Successfully funded the Plinko Game contract with $INITIAL_HOUSE_FUND PLINK"
echo ""

echo "✅ Deployment complete!"
echo ""
echo "📋 Summary:"
echo "==========="
echo "PLINK Token:       $PLINK_TOKEN_ADDRESS"
echo "Purchase Contract: $PURCHASE_CONTRACT_ADDRESS"
echo "Plinko Game:       $GAME_CONTRACT_ADDRESS"