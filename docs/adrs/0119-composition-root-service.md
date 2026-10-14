# ADR-0119: Composition root — service (`jitter-service`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13

## Context

ADR-0027 planned `jitter-service` as a long-running executable wiring every adapter
into `jitter-application` over a service boundary, with `build_app(AppConfig) ->
RunFleetRetryWorkload<AwsRetryConfigAdapter, RvfWitnessAdapter, RuVectorMemoryAdapter>`
as its composition function, and a `/run` endpoint.

## Viable alternatives

- Pull in a full async web framework (axum/actix) for the HTTP boundary. Rejected for
  this slice: this crate's job is to prove the composition wiring is real and
  black-box-testable, not to be a production-grade HTTP server; a framework adds a
  large dependency surface (and an async runtime) disproportionate to one `/run` and
  one `/evaluate` endpoint.
- Hand-roll a minimal HTTP/1.1 request/response cycle over `std::net::TcpListener`,
  parsing just the request line, `Content-Length`, and body. Chosen.
- Expose only `/run`, leaving `jitter-evaluation` and `jitter-adapter-metaharness`
  wired into `jitter-cli` only. Rejected: docs/architecture.md §3's diagram draws
  `SVC --> AAWS & ARUV & ARVF & ARUFLO & AMETA` — the service composition root is
  meant to reach every adapter, including MetaHarness; a second endpoint,
  `/evaluate`, was added specifically so this wiring has a real caller inside this
  crate rather than being declared but unexercised.

## Decision

`build_app` wires `AwsRetryConfigAdapter`, `RvfWitnessAdapter`, and
`RuVectorMemoryAdapter` into one `RunFleetRetryWorkload`. `serve(&TcpListener,
&AppConfig, max_requests)` drives a bounded (production: `usize::MAX`) request loop;
`POST /run` executes a scenario and returns its `Receipt`; `POST /evaluate` scores a
candidate against the unweighted baseline via `jitter-evaluation` and submits the
result through `MetaharnessAdapter`.

## Tradeoffs

The HTTP layer has no keep-alive, no chunked transfer encoding, and no concurrency
beyond one request at a time — correct for proving the composition wiring under test,
not suitable as shipped production infrastructure without a real framework swap.

## Consequences

`jitter-service` is, along with `jitter-cli`, one of exactly two crates in the
workspace permitted to depend on every adapter.

## Validation

- `cargo test -p jitter-service` (3 tests: a black-box `/run` request over a real
  `TcpStream` returning a `Receipt` with `authority: none`, an unknown path returning
  404, and `/evaluate` submitting to `MetaharnessAdapter` and reporting that the
  weighted strategy outranks the baseline)
