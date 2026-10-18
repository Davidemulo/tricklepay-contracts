# Decision Records

This file records design decisions that were deliberate trade-offs, not
oversights, so a reviewer who notices the gap does not have to re-litigate it
from scratch. Each record is numbered and, once accepted, is not edited in
place — a changed decision gets a new record that supersedes it and says so.

## ADR 1: Enumeration is off-chain, not an entry point

**Status:** Accepted

### Context

There is no entry point that answers "which streams does address X have?".
`get_stream(id)` returns one stream by id; `stream_count()` returns how many
ids exist in total. Nothing returns the set of ids for a given `sender` or
`recipient`. The only on-chain way to find a party's streams today is to call
`get_stream` for every id from `0` to `stream_count() - 1` and filter
client-side, which is O(every stream ever created on the deployment) and gets
slower with every new stream, regardless of how many belong to the party
asking.

This gap is deliberate actually, not an oversight. `Created`, `Withdrawn`, and
`Cancelled` all index `sender` and/or `recipient` as event topics specifically
so an off-chain consumer can answer this query without the contract doing it
(see [README.md § Events](../README.md#events)); the indexer and web client
that consume those events live in separate repositories.

### Decision

Enumeration by party stays off-chain, answered by an indexer that subscribes
to `Created`, `Withdrawn`, and `Cancelled` and reconstructs the
address-to-stream-id mapping from their topics. The contract exposes no
`streams_of(address)` or equivalent entry point, and none is planned.

### Alternatives considered

1. **A reverse index stored on-chain** — e.g. a `Vec<u64>` of stream ids kept
   per address, appended to on every `create_stream`. Rejected: every
   creation would have to read and rewrite a collection that only grows, for
   both `sender` and `recipient`, so the cost of `create_stream` increases
   without bound as a party accumulates streams. This is exactly the
   unbounded-storage-growth cost Soroban's fee model is designed to price
   out, and it would eventually make creating or reading a heavy user's
   streams prohibitively expensive or too large to read back in one call.

2. **A paginated view entry point** — `list_streams(address, offset, limit) ->
   Vec<u64>` — backed by the same kind of on-chain index. Rejected for the
   same reason as (1): pagination only changes how the index is read, not the
   fact that the contract would still have to maintain and mutate a per-address
   list on every `create_stream`, `withdraw`, `withdraw_amount`, and `cancel`.

3. **Brute-force client iteration over `stream_count`.** Already possible
   today with no contract change, but it is O(total streams on the
   deployment) per query, paid in RPC round-trips by whoever is asking. Fine
   for a local script or a contract with a handful of streams; not something
   a production client should do for "show me my streams" at scale.

4. **Off-chain indexer on event topics (chosen).** The topics the contract
   already publishes for other reasons make `sender`/`recipient` natively
   filterable by any indexer, with no additional contract state or logic.
   This is the standard Soroban/Stellar pattern: keep the contract minimal
   and emit enough signal for indexers to reconstruct whatever view is
   needed. It also lets independent indexers build different views — by
   sender, by recipient, by token, by status — from the same event stream
   without the contract committing to any one of them.

### Consequences

- A client cannot derive "my streams" from the contract alone; it needs an
  indexer that has processed the event history (or a query to one). There is
  no on-chain fallback beyond the slow brute-force iteration in alternative 3.
- `create_stream`, `withdraw`, `withdraw_amount`, and `cancel` have storage
  and fee costs that do not grow as a party accumulates streams, because no
  per-address collection is read or mutated by any of them.
- Adding enumeration later, if it ever becomes necessary, is additive — a new
  entry point — rather than a retrofit, since no existing stream record needs
  to change shape to support it.

### What would have to change to revisit this

- A concrete case where off-chain indexing is not viable: for example, a
  deployment that must be fully self-contained with no indexer
  infrastructure, or a client environment that cannot run or trust a
  third-party indexer and must verify "my streams" from the contract alone.
- If that case materializes, the shape most consistent with this contract's
  low-footprint design is the paginated view in alternative 2, not an
  unbounded `Vec`, with the storage-growth cost accepted explicitly as a new,
  deliberate trade-off — and the resulting entry point treated as a new
  public interface surface per
  [docs/INTEGRATOR_OPERATIONS.md § Interface Stability Policy](INTEGRATOR_OPERATIONS.md#interface-stability-policy).
- Until one of those is true, this decision stands: enumeration by party is a
  client/indexer concern, not a contract concern.


## ADR: Immutable Architecture with No Admin or Initializer

- **Status**: Accepted
- **Context**: Smart contract protocols often introduce administrative keys, upgrade proxies, or initialization functions to allow pausing, parameter tuning, or code updates post-deployment. However, administrative privileges introduce critical centralization risks, single points of failure, and high-value target attack surfaces for private key compromise or malicious governance takeovers.
- **Decision**: The TricklePay smart contract architecture is strictly immutable, featuring **no admin role, no owner account, and no `initialize` function**. All contract behavior is fully deterministic and hardcoded upon deployment.
- **Consequences**:
  - **Upgrades**: The deployed contract cannot be upgraded in place. Any logic updates require deploying a brand-new contract instance and handling client-side migration.
  - **Pausing**: There is no emergency pause mechanism. Risk mitigation relies entirely on pre-deployment audit rigor and robust client-side validation.
  - **Recovery**: Administrative fund rescue or asset recovery functions do not exist; misdirected or locked funds cannot be arbitrarily withdrawn by any party.
- **Related References**: 
  - See the [Audit Readiness Threat Model](../docs/AUDIT_READINESS.md#threat-model) for central privilege risk assessments and trust assumption boundaries.

  ## ADR: Linear-Only Vesting Release Curve

- **Status**: Accepted
- **Context**: Vesting and token streaming protocols require a mathematical formula to calculate unlocked token amounts over time. While complex release curves (such as exponential, logarithmic, or S-curves) are used in some financial contexts, TricklePay's initial design requires a predictable, transparent, and computationally lightweight mechanism.
- **Decision**: Restrict the smart contract vesting logic to strictly **linear release curves** following an optional cliff period.
- **Rationale**:
  - **Simplicity & Predictability**: Linear vesting is easily understood by beneficiaries and integrators without requiring complex mathematical tooling.
  - **Gas & Compute Efficiency**: Linear calculations require minimal arithmetic operations, optimizing Soroban CPU and memory footprint.
  - **Auditability**: Straightforward formulas significantly reduce potential edge cases and rounding bugs during audits.
- **Impact of Introducing Alternative Curves**:
  If future development introduces non-linear curves (as noted in the [Roadmap](../README.md#roadmap)), the following changes will be required:
  - **Interface / ABI**: Stream creation functions must be refactored to accept curve type enums and curve-specific parameters/coefficients.
  - **Events**: Stream initialization events must emit curve metadata so off-chain indexers know which mathematical model applies.
  - **View Functions**: Calculations in `vested_amount` and `claimable_amount` will require dynamic branching or modular curve evaluation functions rather than direct linear interpolation.
- **Related References**:
  - See the roadmap item for future vesting curve flexibility.

  ## ADR: On-Demand Derived Stream Status

- **Status**: Accepted
- **Context**: Streaming and payment contracts track state transitions across a stream's lifecycle (e.g., `Pending`, `Active`, `Completed`, or `Cancelled`). Storing explicit state values on-chain requires state-transition transactions (or cron/keeper bots) to push state changes as time advances past start/end thresholds, introducing unnecessary gas costs and transaction overhead.
- **Decision**: Stream status is **dynamically derived on every call** directly from the current ledger timestamp (`env.ledger().timestamp()`) relative to the stream's `start_time` and `end_time`. Stream status is never stored as persistent state on-chain.
- **Consequences**:
  - **Zero Transaction Overhead**: Streams transition between lifecycle states (e.g., from `Pending` to `Active` to `Completed`) seamlessly without requiring explicit state-transition transactions or maintenance calls.
  - **Storage & Gas Optimization**: Eliminates on-chain storage reads/writes for status updates, reducing overall ledger footprint and transaction gas fees.
  - **Impact on Indexers and Clients**: Off-chain indexers, UI frontends, and client SDKs cannot simply read a static status field from contract storage. Instead, they must mirror the contract's status calculation logic using the stream's temporal parameters and the latest ledger header timestamp.
- **Related References**:
  - See the status calculation logic in the [Status Module](../contracts/tricklepay/src/types.rs).x

  ## ADR: Ledger Timestamp as the Canonical Clock

- **Status**: Accepted
- **Context**: Time-dependent financial vesting and token streaming require a reliable, tamper-proof source of time within the smart contract environment. Smart contract platforms generally offer two primary mechanisms: block/ledger sequence numbers or network ledger timestamps.
- **Decision**: Use `env.ledger().timestamp()` as the canonical source of time for all vesting calculations, stream start/end thresholds, and rate evaluations.
- **Alternative Considered & Rejected**: 
  - *Ledger Sequence Numbers*: Using discrete ledger count increments as a proxy for time passing.
  - *Why it was rejected*: Ledger sequences do not map intuitively to real-world calendar time or human expectations (e.g., scheduling a stream to unlock on a specific date versus calculating exact ledger heights). Furthermore, ledger close times on Stellar are highly stable (~5 seconds per ledger), making wall-clock timestamps superior for predictability. While ledger timestamps are bounded by ledger close resolution rather than continuous sub-second precision, they provide an ideal balance of auditability and real-world utility.
- **Consequences**:
  - **Intuitive Scheduling**: Vesting cliffs and durations align directly with Unix epoch timestamps, making client integration and user comprehension straightforward.
  - **Granularity Limits**: Time calculations are precise to the ledger close interval (~5 seconds) rather than wall-clock milliseconds.
- **Related References**:
  - See the detailed discussion on clock precision and assumptions in the [Time and Ledger Clock section of the README](../README.md#time-and-ledger-clock).


  ## ADR: `i128` Amount Type and `i64::MAX` Stream Amount Ceiling

- **Status**: Accepted
- **Context**: Soroban token interfaces utilize `i128` data types for token balances and transfers to support high-precision assets. However, calculating linear vesting streams requires multiplying total amounts by elapsed time and duration intervals, which creates potential integer overflow vectors during arithmetic operations.
- **Decision**: Token amounts are stored and processed as `i128` for complete compatibility with the Stellar / Soroban token standard, but `create_stream` strictly enforces an upper limit capped at `i64::MAX` for individual stream allocations.
- **Consequences**:
  - **Arithmetic Safety**: Capping individual streams at `i64::MAX` guarantees that intermediate multiplication steps during vesting calculations cannot overflow standard integer boundaries.
  - **Largest Streamable Amount**: The maximum amount streamable in a single contract instance is `i64::MAX` (approximately $9.22 \times 10^{18}$ base units).
  - **Exclusions**: Assets with astronomical token supplies or extreme decimal precision exceeding `i64::MAX` cannot be streamed in a single stream instance and must be split across multiple streams if necessary.
- **Related References**:
  - See the detailed discussion in the [Amount Ceiling section of the README](../README.md#amount-ceiling).


## ADR: Accept Any Token Address Without an Allowlist

**Status:** Accepted

### Context

`create_stream` accepts a caller-supplied `token: Address` with no
restrictions. The contract performs no allowlist check, no registry lookup, and
no verification of the token's implementation beyond confirming that it responds
to a `transfer` call. Every stream stores the token address it was created with
and invokes that same address — with the contract's own authority — for
`withdraw`, `withdraw_amount`, and `cancel` for the entire life of the stream.

This means a sender can open a stream with any Soroban contract address,
including one that does not conform to SEP-41, charges an undisclosed fee,
reenters the stream contract from within a transfer hook, or always fails and
makes the stream permanently illiquid. The [Trust assumptions about the token
contract](../THREAT_MODEL.md#trust-assumptions-about-the-token-contract) section
of the threat model spends a full page documenting what a hostile or defective
token can do: silent short-transfers, reentrancy, permanent locking, and
balance-accounting disagreements. That is a major security consequence, and it
is a deliberate choice.

### Decision

The stream contract accepts any token address and performs no allowlist,
registry, or conformance checks. Token quality is entirely the sender's
responsibility at creation time. The contract offers no protection against a
malicious or broken token, and neither party can change the token on an
existing stream.

### Alternatives considered

1. **A hardcoded allowlist of known-good tokens in the contract.** Rejected:
   requires the contract to know every legitimate token in advance, which
   cannot scale across every asset on Stellar or every custom token a project
   might issue. An allowlist would have to be updated with every new token
   deployment, which conflicts with the immutability design (see [ADR:
   Immutable Architecture with No Admin or
   Initializer](#adr-immutable-architecture-with-no-admin-or-initializer)).
   Updating the allowlist would require redeployment — at which point the old
   contract still exists with the old list, so streams on the old deployment
   cannot benefit from the expanded coverage. A hardcoded list introduces a
   central gatekeeper role this contract was designed not to have.

2. **A registry contract that the stream contract queries.** Rejected for the
   same central-authority reason as (1), but with an additional failure mode:
   if the registry contract itself is upgraded, paused, or compromised, every
   stream contract that depends on it inherits that single point of failure.
   The registry becomes a shared choke point, and its trust assumptions
   propagate to every deployment that calls it. A registry also adds a
   cross-contract call to every `create_stream` invocation, which costs fees
   and latency, and introduces a new class of failure (registry unavailable,
   registry returns stale data) that the sender cannot control.

3. **A token conformance check run at `create_stream` time** — for example,
   attempting a zero-value transfer and verifying the result matches
   expectations. Rejected: a well-behaved zero-value dry run does not prove the
   token will behave correctly under load, with real amounts, or in the future.
   A token that passes a creation-time check can still fail later, charge an
   undisclosed fee, or reenter maliciously during an actual withdrawal. The
   check gives false confidence without eliminating the real risk. Additionally,
   Soroban's environment has no try-catch or safe sub-invocation primitive that
   would let the contract attempt a transfer and recover cleanly from a trap or
   reentrancy, so running a test transfer means accepting whatever side effects
   it triggers — which is itself a security risk if the token being tested is
   hostile.

4. **Accept any token address (chosen).** The sender chooses the token at
   `create_stream` time and is responsible for verifying it beforehand. The
   contract trusts the token's transfer call to do exactly what SEP-41 specifies
   and nothing more. If the token violates that, the stream's behavior is
   undefined and the contract offers no remedy. This is explicit: token quality
   is not the contract's concern, and the threat model documents exactly what
   can go wrong when a sender picks a bad one. The contract stays minimal, has
   no privileged decision-maker, and does not add cross-contract dependencies or
   a false-security check.

### Consequences

- **Sender due diligence is mandatory.** Before creating a stream, a sender
  must verify the token conforms to SEP-41, does not charge a transfer fee,
  does not reenter from transfer hooks, and has no issuer controls (freeze,
  claw back, mint-on-transfer) that could break the stream's accounting. The
  contract does none of this checking and provides no safety net if the sender
  skips it.
- **Token choice is irreversible.** Once a stream is created with a given
  token address, neither party can change it. If the token is later discovered
  to be defective, the only recourse is for the sender to cancel (returning the
  unvested portion) or for the recipient to withdraw whatever has already
  vested, but the stream itself cannot be migrated to a different token
  contract.
- **No central gatekeeper.** The contract never consults an allowlist,
  registry, or admin before accepting a token. Anyone can stream any Soroban
  contract address. This is consistent with the immutability and no-admin
  design (see [ADR: Immutable Architecture with No Admin or
  Initializer](#adr-immutable-architecture-with-no-admin-or-initializer)), but
  it means the contract cannot protect users from their own token choices.
- **The threat model is the sender's responsibility to read.** The [Trust
  assumptions about the token
  contract](../THREAT_MODEL.md#trust-assumptions-about-the-token-contract)
  section lists every failure mode a non-conforming token can cause. A sender
  who does not read it and streams a hostile token has no claim the contract
  failed to warn them.

### Related references

- [THREAT_MODEL.md § Trust assumptions about the token
  contract](../THREAT_MODEL.md#trust-assumptions-about-the-token-contract) —
  full explanation of what a hostile or defective token can do.
- [INTEGRATOR_OPERATIONS.md § Token due diligence
  checklist](#) (to be added) — short list of checks a sender should perform
  before choosing a token.

  