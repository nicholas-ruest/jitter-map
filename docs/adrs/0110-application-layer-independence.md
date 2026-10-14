# ADR-0110: Application layer independence (`jitter-application`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13

## Context

ADR-0027 planned `jitter-application` as use-case orchestration generic over ports,
never naming a concrete adapter crate, with dependency injection through generics
supplied by a composition root. The base monolith's `jitter-application` already used
this pattern for its three traits (`PolicySource`, `OutcomeMemory`, `WitnessSealer`),
but those traits predate `jitter-ports` and do not match its five-port, DTO-based
shape.

## Viable alternatives

- Keep the base monolith's three ad hoc traits, rename them to match `jitter-ports`'
  vocabulary. Rejected: `jitter-ports` already defines the authoritative port shapes
  (`RetryConfigSource`, `WitnessSink`, `MemorySink`, `HarnessSink`, `FlywheelSink`);
  duplicating trait definitions in two crates would reintroduce exactly the
  "which trait is the real one" ambiguity ADR-0109 exists to prevent.
- Give `RunFleetRetryWorkload` four or five generic parameters, one per port it might
  ever use. Rejected: the planning table's signature is explicitly three generics
  (`R`, `W`, `M`); a fourth (`HarnessSink`) is needed only for the recovery-aware
  execution path, not for a plain run, so it is added as a method-level generic
  instead of a struct-level one.
- `RunFleetRetryWorkload<R: RetryConfigSource, W: WitnessSink, M: MemorySink>` matching
  the planning table exactly, with `execute_with_recovery<H: HarnessSink>(&self,
  scenario, harness: &H)` as a separate method carrying its own extra generic. Chosen.

## Decision

`execute(&self, ValidatedScenario) -> Result<RunReceipt, AppError>` resolves an
`AdmissionConfig` from the injected `RetryConfigSource`, runs
`jitter_simulation::run`, and binds a receipt through `WitnessSink` and `MemorySink`.
`execute_with_recovery` additionally polls `HarnessSink::poll_interruption` before
running: if an interruption is reported, it routes through
`jitter_recovery::capture` + `replay` instead of a plain run, and reports the
resulting events to `HarnessSink::on_tick`. `AppError` has one variant per port
(`RetryConfig`, `Witness`, `Memory`) and never swallows a port failure into a success.

## Tradeoffs

Splitting plain `execute` from recovery-aware `execute_with_recovery` means a caller
must choose which one it wants, rather than recovery always being implicit. This is
deliberate: a composition root that never expects interruption (most test callers)
should not have to supply a `HarnessSink` it will never meaningfully drive.

## Consequences

`cargo tree -p jitter-application` names zero `jitter-adapter-*` crates — the mocked
end-to-end use-case test for a `fleet-retry-storm-22`-shaped scenario passes using
only `jitter-ports`' mocks, proving the application layer needs nothing concrete to
be correct.

## Validation

- `cargo test -p jitter-application` (5 tests: mocked E2E pass, memory-recall growth
  across repeated runs, a port failure surfacing as a typed `AppError` rather than a
  silent success, plain-vs-recovery-aware execution producing identical receipts with
  no interruption signaled, and interruption routing through checkpoint/replay
  producing the same receipt as the uninterrupted baseline)
