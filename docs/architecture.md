# Architecture

> **CURRENT-STATE WARNING.** Base commit `3db7fd2` ships **6 packages** implementing a
> monolithic domain/application split (CLI, Application, Deterministic engine, AWS
> RetryConfig, RVF witness, RuVector memory, plus a Darwin/Flywheel evaluation
> boundary folded into Application). This base is **NONCOMPLIANT** with the bounded-context
> architecture defined below. Everything past this point β€” the 22-crate table, the
> dependency rules, the staged migration, the failure/replay path β€” is a **PLANNED
> design**, not a description of code that exists. No crate in the table below counts
> as compliant until it is implemented, passes the structural checker described at the
> end of this document, *and* passes the semantic acceptance test named in its row. A
> crate that exists only as a `Cargo.toml` and an empty `lib.rs` is not "done" and must
> not be reported as done.

## 1. System thesis

JitterMap exists to answer one question with evidence, not assertion: *when a fleet of
tenants retries against a shared backend after a correlated failure, which admission
policy keeps the system fair and bounded, and can we prove it deterministically and
replay it after a crash?*

The thesis has three load-bearing consequences for the architecture:

1. **The domain must be a pure, deterministic kernel.** Fairness and boundedness claims
   are only trustworthy if the scheduling math has zero dependency on wall-clock time,
   threads, or I/O. Every domain crate (Β§3) must be provable by unit/property test alone.
2. **Infrastructure must be swappable without touching the kernel.** AWS retry
   configuration, RuVector memory, RVF witnessing, RuFlo orchestration signals, and
   Metaharness evaluation feedback are all *adapters* to the same kernel β€” not forks of
   it. Ports (Β§3) are the only contract between the two worlds.
3. **Recovery is not an afterthought.** A fleet retry run that is interrupted mid-tick
   must be resumable to the *same* final outcome as an uninterrupted run. This is a
   first-class bounded context (`jitter-recovery`), not a script bolted onto the CLI.

## 2. Bounded contexts and the context map

| Bounded context | Owns | Crates |
|---|---|---|
| **Domain Core** | Scenario, workload generation, policy, budget, scheduling, outcome β€” pure value objects, aggregates, domain services | `jitter-scenario`, `jitter-workload`, `jitter-policy`, `jitter-budget`, `jitter-scheduler`, `jitter-outcome` |
| **Domain Orchestration** | Composing Domain Core into one deterministic run; replay from checkpoint | `jitter-simulation`, `jitter-recovery` |
| **Hexagonal Boundary** | Port traits; use-case orchestration generic over ports | `jitter-ports`, `jitter-application` |
| **Infrastructure Adapters** | Concrete implementations of ports against real external systems | `jitter-adapter-aws`, `jitter-adapter-ruvector`, `jitter-adapter-rvf`, `jitter-adapter-ruflo`, `jitter-adapter-metaharness` |
| **Cross-Cutting Support**‹­¦λm®ιάjΧ