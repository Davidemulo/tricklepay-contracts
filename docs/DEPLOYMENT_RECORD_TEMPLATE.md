# Deployment Record Template

Use this standardized template to record all smart contract deployments to Staging, Testnet, or Mainnet environments. Completed deployment records should be committed to the repository under a `deployments/` directory or archived securely for audit and verification purposes.

---

## Deployment Metadata Record

- **Deployment Date & Time (UTC)**: `YYYY-MM-DD HH:MM:SS`
  - *Hint*: Timestamp of when the transaction was submitted and confirmed.
- **Target Network**: `[Testnet / Mainnet / Futurenet]`
  - *Hint*: The Stellar network environment where the contract was deployed.
- **Git Tag**: `[e.g., v1.0.0]`
  - *Hint*: Exact git release tag associated with the build (`git describe --tags`).
- **Build Reference / Commit SHA**: `[e.g., a1b2c3d]`
  - *Hint*: Full or short commit hash of the compiled source code (`git rev-parse HEAD`).
- **WASM Hash**: `sha256:[hash]`
  - *Hint*: SHA-256 hash of the deployed `.wasm` binary (printed by `scripts/deploy.sh`).
- **Contract ID / Address**: `[C...]`
  - *Hint*: The deployed Stellar contract identifier returned by the network.
- **Deploying Identity (Public Address)**: `[G...]`
  - *Hint*: Stellar public key / account address that signed the deployment transaction.
- **Transaction Hash**: `[hex hash]`
  - *Hint*: Stellar ledger transaction hash for the deployment invocation.
- **Ledger Number**: `[integer]`
  - *Hint*: The ledger sequence number in which the deployment transaction was included.
- **Verification Result**: `[Passed / Pending / Failed]`
  - *Hint*: Status of source code verification against the on-chain WASM hash using Soroban developer tooling.
- **Sign-off / Auditor Notes**:
  - *Hint*: Any special configuration parameters, initialization arguments, or manual verification notes.