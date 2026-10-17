#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(git rev-parse --show-toplevel)"
cd "$ROOT_DIR"
if [[ $# -ne 2 ]]; then echo "Usage: $0 <contract-id> <network>" >&2; exit 2; fi
WASM="target/wasm32v1-none/release/tricklepay_stream.wasm"
TMP_WASM="$(mktemp)"
trap 'rm -f "$TMP_WASM"' EXIT
cargo build --release --target wasm32v1-none
stellar contract fetch --id "$1" --network "$2" --out-file "$TMP_WASM"
local_hash="$(sha256sum "$WASM" | awk '{print $1}')"
deployed_hash="$(sha256sum "$TMP_WASM" | awk '{print $1}')"
printf 'local:    %s\ndeployed: %s\n' "$local_hash" "$deployed_hash"
if [[ "$local_hash" != "$deployed_hash" ]]; then echo "Deployment does not match the local build." >&2; exit 1; fi
echo "Deployment matches the local build."
