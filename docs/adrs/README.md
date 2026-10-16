# Architecture decision records

51 accepted decisions. 26 (ADR-0001–ADR-0026) freeze the bounded MVP that originally
shipped in the base six-package workspace (`3db7fd2`). ADR-0027 planned the 22-crate
migration described in docs/architecture.md. ADR-0101–ADR-0122 are the 22 decisions
that implement that plan, one per planned crate. ADR-0123 and ADR-0124 define the
bounded live-process and read-only federation-sync increments. Each decision records the real
extraction/composition decision made while building it, including every documented
deviation from the planning table's exact signatures (dependency-direction
constraints took priority over literal signature matches in a handful of cases; each
deviation is recorded in both the relevant ADR and the crate's own doc comment). See
docs/adrs/index.md for the plain link list; this file adds a one-line summary of what
each decision actually settles, so a reader does not have to open all 51 to find the
relevant one.

| ADR | Settles |
|---|---|
| [0001](./0001-system-level-retry-admission.md) System-level retry admission | Admission is a fleet-level concern, not a per-client policy choice |
| [0002](./0002-deterministic-discrete-event-engine.md) Deterministic discrete-event engine | Simulation ordering is time/tenant-round/request-ID/attempt, with no wall-clock dependency |
| [0003](./0003-retry-amplification-factor.md) Retry amplification factor | RAF is the metric comparing candidate policies on the frozen corpus |
| [0004](./0004-typed-failure-classes.md) Typed failure classes | Failure classes are a closed, typed enum, not strings |
| [0005](./0005-aws-anti-corruption-layer.md) AWS anti-corruption layer | AWS RetryConfig is translated behind a typed boundary, never used raw in the domain |
| [0006](./0006-adaptive-shared-budget.md) Adaptive shared budget | The shared budget drops retries it cannot cover |
| [0007](./0007-seeded-full-jitter.md) Seeded full jitter | Jitter is seeded and reproducible, never `rand::thread_rng` |
| [0008](./0008-ruvector-outcome-memory.md) RuVector outcome memory | RuVector is the sole outcome-memory store |
| [0009](./0009-rvf-witness-chain.md) RVF witness chain | Every sealed receipt carries an RVF witness root |
| [0010](./0010-advisory-authority.md) Advisory authority | No evaluator or memory component may promote a policy; authority is always `none` |
| [0011](./0011-evidence-bound-configuration.md) Evidence-bound configuration | Configuration is part of the evidence digest, not out-of-band |
| [0012](./0012-bounded-scenario-input.md) Bounded scenario input | Scenario input is validated and bounded before allocation |
| [0013](./0013-cancellation-and-deadlines.md) Cancellation and deadlines | Runs respect a horizon and `MAX_EVENTS`; nothing runs unbounded |
| [0014](./0014-append-only-evidence.md) Append-only evidence | Evidence corrections append; they never rewrite history |
| [0015](./0015-six-crate-dependency-direction.md) Six-crate dependency direction | The original MVP's dependency-direction rule — superseded for crate *structure* by ADR-0027, otherwise still binding |
| [0016](./0016-no-service-owned-database.md) No service-owned database | The service owns no database; RuVector/Cloud SQL is the only persistence path |
| [0017](./0017-frozen-trace-comparison.md) Frozen trace comparison | Candidates are compared against a frozen, checked-in trace, not a live re-roll |
| [0018](./0018-aws-standard-baseline.md) AWS standard baseline | AWS-standard backoff is a reference ablation, not the system's own invention |
| [0019](./0019-ruvnet-only-ablation.md) Ruvnet-only ablation | Local policy runs with the same Ruvnet evidence path, isolating AWS's contribution |
| [0020](./0020-failure-class-cost.md) Failure-class cost | Each failure class carries an explicit budget cost |
| [0021](./0021-tenant-fairness.md) Deterministic tenant-fair admission | Equal-share and demand-weighted tenant-fair admission are both explicit, executable policies |
| [0022](./0022-promotion-gate.md) Promotion gate | Promotion requires published tradeoffs and valid evidence; eligibility is not promotion |
| [0023](./0023-darwin-safety-envelope.md) Darwin safety envelope | Darwin/Flywheel evaluation cannot widen its own SafetyEnvelope |
| [0024](./0024-cli-json-contract.md) CLI JSON contract | The CLI's stdin/stdout JSON contract is frozen and tested byte-for-byte |
| [0025](./0025-conservative-repository-recovery.md) Conservative repository recovery | The project name is retained; the single-client thesis is replaced before 1.0 |
| [0026](./0026-adaptive-deferral-admission.md) Adaptive deferral admission | `AdaptiveDeferral` reschedules instead of dropping when refill is only temporarily insufficient |
| [0027](./0027-twenty-two-crate-planned-migration.md) Twenty-two-crate planned migration | Plans the 22-crate workspace; ADR-0101–ADR-0122 below are its implementation |
| [0101](./0101-domain-purity-boundary.md) Domain purity boundary | `jitter-scenario`: `ValidatedScenario` makes "validated" a type, not a convention |
| [0102](./0102-deterministic-generation.md) Deterministic generation | `jitter-workload`: tick-0 arrival trace, seed carried forward for downstream jitter |
| [0103](./0103-policy-strategy-pattern.md) Policy strategy pattern | `jitter-policy`: pure delay decision, split from budget-gating and ordering |
| [0104](./0104-budget-value-object.md) Budget value object | `jitter-budget`: consumable balance, refill, and the deferral full-exhaust special case |
| [0105](./0105-deterministic-scheduler.md) Deterministic scheduler | `jitter-scheduler`: `TenantQueues` aggregate, one unified weighted-round-robin ordering fn |
| [0106](./0106-domain-event-log.md) Domain event log | `jitter-outcome`: six-variant `OutcomeEvent` (adds `Retried`, uncounted in the old report) |
| [0107](./0107-pure-simulation-kernel.md) Pure simulation kernel | `jitter-simulation`: composes Domain Core; golden-value regression vs. the deleted monolith |
| [0108](./0108-checkpoint-and-replay.md) Checkpoint and replay | `jitter-recovery`: interrupted-vs-uninterrupted digest match at tick 6 and tick 9 |
| [0109](./0109-ports-and-adapters-boundary.md) Ports and adapters boundary | `jitter-ports`: 5 traits + port-local DTOs; records the dev-dependency-cycle pitfall |
| [0110](./0110-application-layer-independence.md) Application layer independence | `jitter-application`: mocked E2E use case; recovery as a method-level generic |
| [0111](./0111-aws-adapter-isolation.md) AWS adapter isolation | `jitter-adapter-aws`: real `RetryConfig` translation behind `RetryConfigSource` |
| [0112](./0112-ruvector-adapter-isolation.md) RuVector adapter isolation | `jitter-adapter-ruvector`: `MemorySink`-only, split from the old combined ruvnet package |
| [0113](./0113-witness-binding.md) Witness binding | `jitter-adapter-rvf`: `WitnessSink`-only, split from the old combined ruvnet package |
| [0114](./0114-orchestration-signal-boundary.md) Orchestration signal boundary | `jitter-adapter-ruflo`: real atomic signal primitive; honestly no live Ruflo transport |
| [0115](./0115-evaluation-feedback-boundary.md) Evaluation feedback boundary | `jitter-adapter-metaharness`: append-only JSON-Lines feed, honest MetaHarness provenance gap |
| [0116](./0116-evidence-receipt-model.md) Evidence receipt model | `jitter-evidence`: pure `bind`, independently re-verifiable `outcome_digest` |
| [0117](./0117-typed-config-boundary.md) Typed config boundary | `jitter-config`: fail-fast required field, testable without touching real env vars |
| [0118](./0118-cross-cutting-observability.md) Cross-cutting observability | `jitter-observability`: span-per-tick, verified with a real test subscriber |
| [0119](./0119-composition-root-service.md) Composition root — service | `jitter-service`: hand-rolled HTTP `/run` + `/evaluate`, both black-box tested |
| [0120](./0120-composition-root-cli.md) Composition root — CLI | `jitter-cli`: same CLI contract, `--interrupt-after`/`--replay` added |
| [0121](./0121-evaluation-boundary.md) Evaluation boundary | `jitter-evaluation`: re-pointed at the new kernel; committed fitness-delta regression |
| [0122](./0122-shared-contract-test-suite.md) Shared contract test suite | `jitter-contract-tests`: 5 `assert_*` fns, invoked by all 5 real adapters |
| [0123](./0123-live-process-integration-boundaries.md) Live process integration boundaries | Pinned Ruflo and MetaHarness processes run behind bounded Rust APIs with typed append-only receipts and authority `none` |
| [0124](./0124-read-only-federation-sync.md) Read-only federation sync boundary | Ruflo sync preserves the untrusted envelope and emits only bounded, hashed metadata with no command or membership authority |
