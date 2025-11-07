#!/bin/bash

# Optimized Release Build Script for Plinko Game Contracts
# Uses cosmwasm/workspace-optimizer for production-ready WASM files

set -e

echo "🚀 Building Optimized Release Contracts"
echo "========================================"
echo ""

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Error: Docker is not running"
    echo "Please start Docker and try again"
    exit 1
fi

# Create artifacts directory if it doesn't exist
mkdir -p artifacts

echo "📦 Running workspace optimizer..."
echo "This may take several minutes..."
echo ""

# Run the optimizer
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.17.0

echo ""
echo "✅ Build complete!"
echo ""
echo "📋 Optimized artifacts:"
echo "======================="

# List the generated artifacts with sizes
if [ -d "artifacts" ]; then
    for file in artifacts/*.wasm; do
        if [ -f "$file" ]; then
            size=$(ls -lh "$file" | awk '{print $5}')
            echo "  - $(basename "$file") ($size)"
        fi
    done
else
    echo "  ⚠️  No artifacts directory found"
fi

echo ""
echo "📍 Location: $(pwd)/artifacts/"
echo ""

# Optional: Run cosmwasm-check if available
if command -v cosmwasm-check &> /dev/null; then
    echo "🔍 Validating contracts with cosmwasm-check..."
    echo ""
    for file in artifacts/*.wasm; do
        if [ -f "$file" ]; then
            echo "Checking $(basename "$file")..."
            cosmwasm-check "$file"
        fi
    done
    echo ""
    echo "✅ All contracts validated!"
else
    echo "ℹ️  Tip: Install cosmwasm-check to validate contracts:"
    echo "   cargo install cosmwasm-check"
fi

echo ""
echo "🎉 Ready for deployment!"
echo ""
echo "Next steps:"
echo "  1. Review artifacts in ./artifacts/"
echo "  2. Deploy to testnet using deploy.sh"
echo "  3. Update frontend .env with contract addresses"
