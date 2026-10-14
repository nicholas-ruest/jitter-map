# ADR-0118: Cross-cutting observability (`jitter-observability`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13

## Context

ADR-0027 planned `jitter-observability` to wrap tracing/metrics instrumentation,
never panicking when tracing is disabled, with a planning-table signature
`record_tick_metrics(&TickResult)` — a `jitter-scheduler` type.

## Viable alternatives

- Depend on `jitter-scheduler` for `TickResult` to match the planning table's
  signature exactly. Rejected: a cross-cutting crate whose only reason to depend on a
  specific domain crate is one struct's shape is a narrower, more fragile dependency
  than necessary — `jitter-outcome`'s `OutcomeEvent` already carries everything worth
  logging (tenant, request, tick, outcome kind).
- Take `&[OutcomeEvent]` directly, depending only on `jitter-outcome`. Chosen,
  documented as a deviation in the crate's own doc comment.

## Decision

`init_tracing(&AppConfig) -> Guard` installs a global `tracing_subscriber::fmt`
subscriber filtered by `RUST_LOG` (defaulting to `info`), swallowing a "subscriber
already installed" error rather than panicking (`try_init`, not `init`).
`record_tick_metrics(tick, &[OutcomeEvent])` emits one `tracing::info!` per event
inside a `tick` span, with `tenant_id`/`request_id`/`admitted`/`outcome` fields.

## Tradeoffs

Verifying this crate's own test subscriber required discovering a real pitfall:
`tracing_subscriber::fmt()`'s default ANSI color codes inject escape sequences around
the `=` in `key=value` output, breaking naive `str::contains("key=value")` assertions
in a test buffer. The fix (`.with_ansi(false)` for the test subscriber) is now the
documented pattern this crate's own tests use.

## Consequences

Running any scenario through `jitter-cli` or `jitter-service` emits one span per
tick's outcome batch with `tenant_id`/`admitted` fields, verifiable by any subscriber
— exactly the planning table's acceptance criterion, verified here with a test
subscriber rather than only asserted.

## Validation

- `cargo test -p jitter-observability` (3 tests: span/field emission verified via a
  test subscriber, `init_tracing` safe to call more than once, an empty event list
  never panics)
