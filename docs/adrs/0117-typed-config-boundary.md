# ADR-0117: Typed config boundary (`jitter-config`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13

## Context

ADR-0027 planned `jitter-config` with fail-fast validation and no silent defaults for
required fields, used only by `jitter-cli` and `jitter-service`. The base monolith had
no typed config crate — CLI flags and their `clap` defaults were the only
configuration surface, with no single place that could fail fast before any adapter
was constructed.

## Viable alternatives

- Make every field optional with a built-in default. Rejected: this is exactly the
  "silent defaults for required fields" the planning table's acceptance criterion
  forbids — a misconfigured memory path would silently fall back to a default rather
  than surfacing an error.
- Read directly from `std::env::vars()` inside `load`, with no testable seam.
  Rejected: every test would then have to mutate real process environment variables,
  which is unsafe under parallel test execution.
- `load()` reads real process environment variables and delegates to `load_from(&
  HashMap<String, String>)`, which is what every test calls directly. `memory_path` is
  the one required field (`JITTER_MEMORY_PATH`); `aws_max_attempts`, `timeout_ms`, and
  `flywheel_feed_path` have documented defaults. Chosen.

## Decision

`ConfigError::Missing` and `ConfigError::Invalid` are the only two failure modes.
`jitter-cli` builds its environment map from already-parsed CLI flags (so the CLI's
existing `--memory` flag flows through this same typed validator rather than
requiring users to set environment variables directly); `jitter-service` reads the
real process environment, since it has no CLI flags of its own.

## Tradeoffs

Routing CLI flags through a synthetic environment map (rather than parsing them
directly into `AppConfig`) adds one layer of indirection, in exchange for both
composition roots sharing exactly one validation path with exactly one set of
fail-fast rules.

## Consequences

A malformed config value fails before `AwsRetryConfigAdapter`, `RvfWitnessAdapter`, or
`RuVectorMemoryAdapter` is ever constructed — verified directly in this crate's own
test suite, not just asserted in prose.

## Validation

- `cargo test -p jitter-config` (6 tests: missing required field, documented
  defaults, invalid/zero `aws_max_attempts`, invalid `timeout_ms`, explicit overrides
  honored)
