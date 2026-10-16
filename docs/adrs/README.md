# Architecture decision records

51 accepted decisions. 26 (ADR-0001â€“ADR-0026) freeze the bounded MVP that originally
shipped in the base six-package workspace (`3db7fd2`). ADR-0027 planned the 22-crate
migration described in docs/architecture.md. ADR-0101â€“ADR-0122 are the 22 decisions
that actually implement that plan, one per planned crate â€” each one record the real
extraction/composition decision made while building it, including every documented
deviation from the planning table's exact signatures (dependency-direction
constraints took priority over literal signature matches in a handful of cases; each
deviation is recorded in both the relevant ADR and the crate's own doc comment). See
docs/adrs/index.md for the plain link list; this file adds a one-line summary of what
each decision actually settles, so a reader does not have to open all 49 to find the
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
| [0015](./0015-six-crate-dependency-direction.md) Six-crate dependency direction | The original MVP's dependency-direction rule â€” superseded for crate *stz.¶›­¶»§q«^