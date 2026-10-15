# ADR-0123: Live process integration boundaries

Status: Accepted

## Context

The 22-crate system has real typed Ruflo and MetaHarness ports, but the published
implementation stops at an in-process interruption signal and an append-only JSONL
feed. Those are useful seams, not proof that the external tools consumed or produced
anything. The final recovery increment must exercise actual installed processes while
keeping evaluation advisory and preserving deterministic replay.

The available pinned interfaces are `@claude-flow/cli` 3.56.10 (MIT, npm integrity
`sha512-jcl7i1gwnb1vQRlGwmXU/FkBulxf3nKhYPC6cZiDPn0fptw6OJgSSsgAQf229axUcqsmrA3pd3V+ma0vO7Xv+A==`),
Darwin 0.10.3, and Flywheel 0.1.12. MetaHarness source is pinned to
`9ce8b8dd89045c3b9a1f809ae58f3589029db4a4`; npm omits `gitHead`, so source and
registry provenance remain separate facts. A ChatGPT Ruflo MCP identity probe is
external execution evidence, but it is not the JitterMap process transport.

## Alternatives considered

1. Continue reporting the in-process adapters as partial. Safe, but it leaves the two
   named completion gaps unresolved.
2. Link Node packages into Rust through FFI. This couples allocators and runtimes and
   makes process isolation, timeouts, and evidence capture harder.
3. Execute pinned CLIs/modules as child processes behind typed Rust APIs, with bounded
   I/O and append-only receipts.
4. Call the public relay directly from domain crates. This leaks transport concerns
   into the domain and duplicates Ruflo protocol behavior.

## Decision

Choose option 3. `jitter-adapter-ruflo` owns a `RufloProcessTransport` configured by
`JITTER_RUFLO_BIN`, an immutable argv prefix, a working directory, and a timeout.
It executes only a fixed, allowlisted provenance operation; caller data is passed as
a value argument, never a shell fragment. `jitter-adapter-metaharness` owns a
`MetaharnessProcessRunner` configured by `JITTER_NODE_BIN`,
`JITTER_DARWIN_CLI`, `JITTER_DARWIN_GENOME`, `JITTER_CANDIDATE_BIN`,
`METAHARNESS_FLYWHEEL_ENTRY`, and a timeout. It runs a bounded Darwin numeric
suite and invokes a repository-owned ESM bridge that imports Flywheel, reads the
adapter JSONL, runs a bounded generation, and verifies the replay bundle.

Both runners use `std::process::Command` without a shell, clear or explicitly set the
working directory, cap stdout and stderr at 64 KiB, reject missing/non-file
executables, enforce a wall-clock timeout, require exit code zero, validate structured
JSON output, and append a receipt before returning success. Secrets and environment
dumps are never captured. Receipts record tool/version, argv and output fingerprints,
duration, exit status, and authority `none`; Darwin additionally records its genome
input digest.

## Tradeoffs

Process startup adds latency and depends on installed artifacts. The isolation is
intentional: crashes and package-runtime state cannot corrupt the Rust process.
Absolute executable/module paths improve auditability but require explicit runtime
configuration. A timeout helper thread is acceptable for this bounded evaluator path;
it is not used in the deterministic simulation kernel.

## Consequences

The existing in-process `HarnessSink` and `FlywheelSink` behavior remains available as
an ablation. Live process success is a separate acceptance gate and cannot be inferred
from contract tests. Missing tools, malformed output, timeout, non-zero exit, oversized
output, or receipt-write failure return `PortError` and cannot promote a candidate.
Darwin/Flywheel retain authority `none`; successful execution is evidence, not release
authorization.

## Validation

- Code review plus typed fixed-argv construction proves no shell interpolation;
  child-process fixtures exercise the direct-process boundary.
- Failure fixtures cover missing executable, timeout, non-zero exit, malformed JSON,
  oversized output, and unwritable receipt path.
- The installed Ruflo CLI performs the allowlisted provenance operation and produces
  a schema-valid receipt.
