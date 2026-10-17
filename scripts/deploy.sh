#!/usr/bin/env bash
#
# Build and deploy the stream contract to a Stellar network.
#
# Usage:
#   ./scripts/deploy.sh <identity-name>
#
# Requires the Stellar CLI (https://developers.stellar.org/docs/tools/cli) and a
# funded identity created with `stellar keys generate`. The network defaults to
# testnet and can be overridden with the NETWORK environment variable.

set -euo pipefail

IDENTITY="${1:-}"
if [ -z "$IDENTITY" ]; then
  echo "usage: $0 <identity-name>" >&2
  exit 1
fi

NETWORK="${NETWORK:-testnet}"
# soroban-sdk requires wasm32v1-none on Rust 1.82+; wasm32-unknown-unknown
# enables wasm features the Soroban environment does not support.
WASM_TARGET="wasm32v1-none"
WASM="target/${WASM_TARGET}/release/tricklepay_stream.wasm"

echo "Building optimized WASM..."
cargo build --release --target "$WASM_TARGET"

echo "Deploying to ${NETWORK} as '${IDENTITY}'..."
stellar contract deploy \
  --wasm "$WASM" \
  --source "$IDENTITY" \
  --network "$NETWORK"

#!/usr/bin/env bash
set -e

# --- Existing build and deployment steps ---
# (Ensure your WASM file path and stellar deploy command are properly set here)

WASM_PATH="target/wasm32-unknown-unknown/release/tricklepay.wasm" # Adjust path as configured in your project

# Example deployment command capturing contract ID
# CONTRACT_ID=$(stellar contract deploy ...)

# Compute SHA-256 hash of the deployed wasm file
if command -v sha256sum &> /dev/null; then
    WASM_HASH=$(sha256sum "$WASM_PATH" | awk '{print $1}')
elif command -v shasum &> /dev/null; then
    WASM_HASH=$(shasum -a 256 "$WASM_PATH" | awk '{print $1}')
else
    echo "Error: Neither sha256sum nor shasum is available." >&2
    exit 1
fi

echo "=========================================="
echo "Deployment Successful!"
echo "WASM File   : $WASM_PATH"
echo "WASM Hash   : sha256:$WASM_HASH"
echo "Contract ID : $CONTRACT_ID"
echo "=========================================="