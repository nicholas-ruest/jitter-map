# Frozen implementation contract

## Current API contract (base `3db7fd2`, implemented today)

`simulate(&Scenario, &RetryPolicy, &RunControl) -> Result<SimulationReport, DomainError>`
is the deterministic core. `evaluate(EvaluateRequest, ports)` validates, resolves policy,
simulates, seals, appends memory, and returns an advisory receipt.

All candidates receive the same scenario digest, seed, capacity trace, horizon, ceiling,
and metric code. `AdaptiveBudget` drops a retry when shared budget cannot cover its cost.
`AdaptiveDeferral` reschedules until projected refill would cover the cost, bounded by
the horizon and `MAX_EVENTS`. `TenantFairBudget` preserves drop semantics but orders a
same-tick batch through rotating per-tenant FIFO rounds. Optional tenant weights must be
positive, cardinality-matched, overflow-safe, and included in the scenario digest.
Reports expose per-tenant outcomes and optional Jain success fairness. Eligibility
requires publishing the fairness/throughput tradeoff and valid evidence; eligibility is
not promotion.

### Weighted fairness extension

`WeightedTenantFairBudget` is opt-in. With `tenant_weights`, each same-tick round admits
at most the tenant's weight; without weights, every quantum is one. Inputs must pass the
shared scenario validator. Equal-share, adaptive drop and adaptive deferral remain
executable ablations. All receipts retain `authority: none`.

## Planned API contract (22-crate target, docs/architecture.md §5 — not yet built)

Each row below is the typed public API a crate must expose once it is extracted; a crate
is not compliant if its API drifts from this table without a new ADR. Full detail lives
in docs/architecture.md §5; this contract restates only the function/type signatures and
their failure semantics.

| Crate | Entry point | Failure contract |
|---|---|---|
| `jitter-scenario` | `validate(Scenario) -> Result<ValidatedScenario, ScenarioError>` | Rejects invalid input before any allocation; never panics on malformed input |
| `jitter-workload` | `generate(&ValidatedScenario, seed: u64) -> WorkloadTrace` | Infallible given a `ValidatedScenario`; determinism is the contract, not error handling |
| `jitter-policy` | `RetryPolicy::next_delay(&self, &AttemptHistory) -> Decision` | Pure function; no I/O, no error variant — a policy that cannot decide is a bug, not a `Result` |
| `jitter-budget` | `Budget::try_consume(&mut self, Weight) -> Result<(), BudgetExhausted>` | `BudgetExhausted` is the only failure mode; budget state never goes negative |
| `jitter-scheduler` | `schedule_tick(Tick, &mut TenantQueues, &mut Budget, &dyn RetryPolicy) -> TickResult` | Infallible given valid state; invalid state is a caller contract violation, not a runtime error |
| `jitter-outcome` | `summarize(&[OutcomeEvent]) -> OutcomeSummary` | Infallible; serde round-trip must be exact |
| `jitter-simulation` | `run(&ValidatedScenario, seed: u64) -> SimulationRun` | Infallible given valid input; zero I/O, clock, or thread dependency — structurally enforced |
| `jitter-recovery` | `replay(&Checkpoint, remaining: WorkloadTrace) -> SimulationRun` | Must reach the same output digest as an uninterrupted run; any divergence is a correctness failure, not an acceptable approximation |
| `jitter-ports` | 5 traits: `RetryConfigSource`, `WitnessSink`, `MemorySink`, `HarnessSink`, `FlywheelSink` | Trait-only; zero implementations in this crate; all methods return `Result<_, PortError>` |
| `jitter-application` | `RunFleetRetryWorkload<R,W,M>::execute(&self, ValidatedScenario) -> Result<RunReceipt, AppError>` | `AppError` wraps any port failure; never swallows a port error into a success |
| `jitter-adapter-aws` | `impl RetryConfigSource for AwsRetryConfigAdapter` | Must pass the shared `RetryConfigSource` contract suite before being wired at a composition root |
| `jitter-adapter-ruvector` | `impl MemorySink for RuVectorMemoryAdapter` | Same contract-suite gate; IAM-scoped Cloud SQL path only |
| `jitter-adapter-rvf` | `impl WitnessSink for RvfWitnessAdapter` | Same contract-suite gate; digest-stable across runs |
| `jitter-adapter-ruflo` | `impl HarnessSink for RufloSignalAdapter` | Same contract-suite gate; this adapter has no code today (docs/integration-matrix.md) |
| `jitter-adapter-metaharness` | `impl FlywheelSink for MetaharnessAdapter` | Same contract-suite gate |
| `jitter-evidence` | `bind(..) -> Receipt` | Pure function; a `Receipt` once bound is never mutated, only superseded by a new one |
| `jitter-config` | `load() -> Result<AppConfig, ConfigError>` | Fails fast on any missing/invalid required field; no silent defaults |
| `jitter-observability` | `init_tracing(&AppConfig) -> Guard`, `record_tick_metrics(&TickResult)` | Never panics when tracing is disabled |
| `jitter-service` | binary; `build_app(AppConfig) -> RunFleetRetryWorkload<..>` | Composition root; permitted to depend on everything |
| `jitter-cli` | binary; `fn main()` | Composition root; preserves the current CLI JSON contract; adds `--replay <checkpoint>` |
| `jitter-evaluation` | `CandidateEvaluator::score(&self, &SimulationRun) -> Fitness` | Never returns a `Fitness` that implies promotion authority (ADR-0010, ADR-0022) |
| `jitter-contract-tests` | one `assert_<port>` fn per port trait | Dev-dependency only; must never appear in a runtime dependency graph (checked, §8 below) |

## Exact crate-counting rule

A crate counts toward the 22-crate target, and may be reported as implemented in
PROJECT_STATUS.md, **only if all five of the following hold simultaneously**:

1. **Workspace member.** It appears in `[workspace] members` in the root `Cargo.toml` and
   `cargo metadata` reports it as a package.
2. **Substantive behavior.** Its `lib.rs` (or `main.rs`) contains real logic matching its
   row in docs/architecture.md §5's "Implementation tasks" column — not an empty module,
   not a `todo!()`, not a re-export of another crate's type with no added behavior.
3. **Real caller or executable.** At least one other real crate in the workspace calls it
   (for a library crate), or it is itself a real, runnable binary (for `jitter-cli` /
   `jitter-service`). A crate with zero callers and no binary target does not count, no
   matter how much code it contains.
4. **Tests.** It has its own `#[test]` suite (unit and/or property tests per
   docs/architecture.md §5's "Tests" column) that passes, and — for adapters — it invokes
   and passes the shared `jitter-contract-tests` suite for its port.
5. **E2E participation.** It is exercised, directly or transitively, by at least one
   end-to-end run of `fleet-retry-storm-22` (docs/architecture.md §4) or, for
   `jitter-contract-tests` specifically, by every adapter's dev-dependency test target.

A crate satisfying only (1) is a skeleton and must be reported as such, not as migrated.
A crate satisfying (1)-(4) but never exercised by the E2E fixture is "built but
unproven" and must not be marked compliant. "20+ meaningful crates" in any status
document means crates passing all five checks above, counted individually — not
`Cargo.toml` files, not workspace members alone.

## Staged migration order (docs/architecture.md §6, summarized)

0. ADRs + port contracts: author ADR-0101–ADR-0122, build `jitter-ports` + mocks +
   `jitter-contract-tests` skeleton.
1. Domain Core (crates 1-6): extracted one at a time, each reproducing the current base's
   demand-weighted admission output bit-for-bit before the next starts; old code deleted
   as each lands.
2. Domain Orchestration (`jitter-simulation`, then `jitter-recovery`): the full-stack
   domain-only E2E test must pass before `jitter-recovery` starts.
3. Application (`jitter-application`): built and proven against `jitter-ports` mocks only,
   before any real adapter exists.
4. Adapters, one at a time: `jitter-adapter-aws` first, then `jitter-adapter-rvf`,
   `jitter-adapter-ruvector`, `jitter-adapter-metaharness`, then the new
   `jitter-adapter-ruflo` — never stubbed in parallel.
5. Cross-cutting + roots: `jitter-evidence`, `jitter-config`, `jitter-observability`, then
   `jitter-cli` and `jitter-service`.
6. Evaluation: `jitter-evaluation`, proving the fitness regression against the unweighted
   baseline.

No phase may begin by creating all remaining `Cargo.toml` files up front (docs/architecture.md §6).

## Per-stage acceptance

Each phase above is "done" only when every crate it introduces passes **both** gates
defined in docs/architecture.md §8:

- **Gate 1 (structural).** The dependency-graph lint passes: no inward-to-outward edge
  from Domain Core/Orchestration, no adapter in `jitter-application`'s or any domain
  crate's graph, `jitter-contract-tests` only under `[dev-dependencies]`, no dependency
  cycle.
- **Gate 2 (semantic).** The crate's own acceptance criterion from docs/architecture.md
  §5 passes: a golden-digest comparison, a fairness-regression replay, the
  interrupted-vs-uninterrupted digest match (§7), or the evaluation ranking (row 21), as
  applicable.

A phase that ships Gate-1-passing, Gate-2-pending code must be reported as "structurally
staged," never as "complete" or "migrated" (docs/architecture.md §8; ADR-0027
Consequences). No empty skeletons: a crate's first commit must contain the real behavior
from its docs/architecture.md §5 row, not a placeholder to be filled in later.

## Live process implementation contract

The next increment modifies only the existing Ruflo and MetaHarness adapters plus their
callers, tests, bridge script, documentation, and receipts. It must not add a crate.

- `RufloProcessTransport` and `MetaharnessProcessConfig` carry canonical paths,
  immutable argv, working directory, timeout, 64-KiB output cap, and receipt path.
- `RufloProcessTransport::probe_registry()`,
  `RufloProcessTransport::sync_federation()`, and
  `MetaharnessProcessRunner::evaluate()` return typed receipts; raw
  JSON never crosses into domain crates.
- Federation sync validates gateway bounds and the complete untrusted provenance
  envelope. Payloads remain opaque data; only bounded type frequencies and hashes may
  enter the receipt. The operation is read-only and has authority `none`.
- Processes are spawned with `std::process::Command`, never `sh -c`; inputs are
  serialized to files or individual argv values, never interpolated command text.
- Timeout, missing executable/module, non-zero exit, malformed schema, oversized
  output, replay verification failure, and receipt I/O failure are required tests.
- Real integration tests are opt-in through explicit environment variables and must
  run on RuOS against exact installed artifacts. Fixture-process tests run in locked CI.
- Receipts are append-only JSON Lines with tool/version, argv/output fingerprints,
  duration, exit status, and authority `none`; Darwin adds its genome input digest.
  Environment values remain redacted.
- The unchanged local adapters are executable ablations. Live process removal may
  remove provenance or advisory evidence only; it must not change domain results.
