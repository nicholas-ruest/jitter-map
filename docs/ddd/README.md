# Domain-driven design documents

12 documents describing one coherent domain — JitterMap does not invent twelve bounded
contexts to match the count. See docs/ddd/index.md for the plain link list; this file
adds what each document actually owns and how it relates to the planned 22-crate map in
docs/architecture.md, so a reader can find the right document without opening all 12.
Every document in this set now carries explicit `## Context`, `## Language`, and
`## Invariants` headings (plus `## Ownership and interfaces`, `## Domain events`, and
`## Failure and validation`), so structure is consistent across all 12.

| Doc | Owns | Planned crate(s) |
|---|---|---|
| [01 — Ubiquitous language](./01-ubiquitous-language.md) | The canonical glossary every other document and every crate's public API must reuse unchanged | N/A — glossary, not a crate |
| [02 — Context map](./02-context-map.md) | How the 8 bounded contexts in docs/architecture.md §2 relate and which direction dependencies point | Spans all 22 planned crates |
| [03 — Scenario aggregate](./03-scenario-model.md) | Scenario validation, tenant demand weights, capacity/outage windows | `jitter-scenario` |
| [04 — Policy aggregate](./04-policy-model.md) | `RetryPolicy` strategies: local jitter, AWS-standard, adaptive budget/deferral, tenant-fair | `jitter-policy` |
| [05 — Simulation context](./05-simulation-engine.md) | The deterministic tick loop and event ordering | `jitter-simulation` |
| [06 — Admission context](./06-admission-budget.md) | Budget exhaustion/refill and same-tick FIFO scheduling | `jitter-budget`, `jitter-scheduler` |
| [07 — Evidence context](./07-evidence-receipts.md) | Receipt binding, witness roots, authority-none | `jitter-evidence` |
| [08 — Outcome memory](./08-memory-persistence.md) | RuVector-backed outcome recall | `jitter-adapter-ruvector` |
| [09 — Upstream integration](./09-upstream-integrations.md) | AWS, RuVector, RVF, Ruflo, Darwin/Flywheel adapter boundaries | `jitter-adapter-aws`, `jitter-adapter-ruvector`, `jitter-adapter-rvf`, `jitter-adapter-ruflo`, `jitter-adapter-metaharness`, `jitter-ports` |
| [10 — Application workflows](./10-application-workflows.md) | The `RunFleetRetryWorkload` use case and composition-root wiring | `jitter-application`, `jitter-cli`, `jitter-service` |
| [11 — Security and policy](./11-security-policy.md) | Untrusted-input handling, SafetyEnvelope, no deployment authority | `jitter-config` |
| [12 — Operations and failure](./12-operations-failure.md) | Run states, metrics, checkpoint/interruption/replay | `jitter-recovery`, `jitter-adapter-ruflo` |

No document above claims a planned crate is implemented. Each "Planned crate(s)" column
entry is a forward reference into docs/architecture.md §5 and is gated by
docs/implementation-contract.md's exact crate-counting rule before it may be reported as
built.
