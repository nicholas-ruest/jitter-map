# ADR-0027: Twenty-two-crate planned migration

- Status: Accepted for design-only planning; supersedes ADR-0015 for crate structure only
- Date: 2026-10-11

## Context

Nick now requires design before implementation and a minimum of 20 meaningful implemented crates before the build counts as compliant. The base commit (`3db7fd2`) carries five workspace members plus the root CLI package — six packages total — covering domain, application, two adapters, evaluation, and CLI. ADR-0015 fixed that six-crate shape for the bounded MVP and remains correct about ownership, typed boundaries, and the replay-before-reversal invariant; it did not anticipate a 20+ crate target, a typed service-API boundary, a Ruflo coordination adapter, or separated cross-cutting crates for evidence, config, and observability. The decision must fit a deterministic Rust vertical slice, exact-source evidence, and advisory-only governance, and it must not implement any of the planned crates — this ADR is a plan, not a build.

## Viable alternatives

- Keep the six-package shape and add internal modules instead of crates. Rejected: modules cannot be independently versioned, typed-API-tested, or counted as compliant crates under the new >=20 requirement.
- Explode into 30+ micro-crates by splitting every value type into its own package. Rejected: ADR-0015 already rejected "microcrate explosion"; it would multiply manifests without adding a bounded context or a typed boundary, and most of the resulting packages would have no real caller.
- Grow to exactly 22 crates, one per distinct responsibility already named in the domain model, the upstream integration list, and the operational surface. Chosen.

## Decision

We will plan a 22-crate workspace: eight domain crates (`jitter-scenario`, `jitter-workload`, `jitter-policy`, `jitter-budget`, `jitter-scheduler`, `jitter-outcome`, `jitter-sim-kernel`, `jitter-recovery`), two application-core crates (`jitter-ports`, `jitter-application`), five adapters (`jitter-adapter-aws`, `jitter-adapter-ruvector`, `jitter-adapter-rvf`, `jitter-adapter-ruflo`, `jitter-adapter-metaharness`), three cross-cutting crates (`jitter-evidence`, `jitter-config`, `jitter-observability`), two composition roots (`jitter-cli`, `jitter-service-api`), and two verification/comparison crates (`jitter-evaluation`, `jitter-contract-tests`). Full responsibility, bounded-context, typed-API, caller, ADR/DDD reference, task, test, and acceptance detail for each crate is recorded in docs/architecture.md and must not drift from it. No crate in this list exists yet; `cargo metadata` must continue to report exactly the base six packages until an implementation PR lands a slice, per docs/implementation-contract.md's staging table.

## Rationale and tradeoffs

More manifests and more typed boundaries to keep consistent, in exchange for independently testable responsibilities, a crate-counting rule that cannot be gamed by hollow packages, and an explicit home for Ruflo coordination and a future service API that the six-crate shape had no room for. This is preferred because it preserves replayability and makes the twenty-crate requirement measurable against `cargo metadata` and real tests, instead of satisfied by renaming modules or adding empty packages.

## Consequences

ADR-0015 is superseded for crate *structure* only: the count and names of workspace members change from six to a staged path toward twenty-two. ADR-0015's ownership invariant, its requirement that a reversal needs a new ADR plus a migration note plus a replay of the frozen corpus, and its advisory-only governance all remain binding on every one of the 22 planned crates. Darwin, Flywheel, and Ruflo may propose or score an alternative crate shape but cannot promote one. PROJECT_STATUS.md must read `NONCOMPLIANT_BUILD/IN_PROGRESS` until the first planned crate clears every implementation-contract gate.

## Validation

The executable check is the same `cargo_metadata_direction` check ADR-0015 defined, re-run at every future commit against the planned dependency-direction table in docs/architecture.md; today it must report the unchanged base six packages, which is itself evidence that no crate from this ADR has been implemented. A missing or premature implementation is reported as blocked rather than mocked as passing.
