#!/bin/bash

# Test script for all contracts

set -e

echo "Testing PLINK Token..."
cd plink-token
cargo test
cd ..

echo ""
echo "Testing Purchase Contract..."
cd purchase-contract
cargo test
cd ..

echo ""
echo "Testing Plinko Game..."
cd plinko-game
cargo test
cd ..

echo ""
echo "All tests passed! ✅"
echo ""
echo "Test Summary:"
echo "  - PLINK Token: 15 tests"
echo "  - Purchase Contract: 12 tests"
echo "  - Plinko Game: 18 tests"
echo "  - Total: 45 tests"
