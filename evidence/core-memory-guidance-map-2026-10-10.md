# Core-memory guidance map - 2026-10-10

Status: REPOSITORY_GUIDANCE_APPLIED; LIVE_MEMORY_RETRIEVED.

Repository guidance snapshot: `nicholas-ruest/core-memory@ca7723339bb8295da7f86999250051c68c13d54e`.

| Public-safe source | Applied guidance | Decision and evidence | Status |
|---|---|---|---|
| AGENTS.md (`4afe6f9`) | one writer; typed boundaries; real tests | isolated worktree, typed policy enum, boundary validation, locked tests | APPLIED |
| CLAUDE.md (`0178167`) | coordination is not implementation | Ruflo recorded identity; RuOS executed code and gates | APPLIED |
| architecture overview (`8ba7f76`) | ports/adapters and bounded contexts | policy stays in domain; AWS translation remains adapter-owned | APPLIED |
| anti-corruption ADR (`9a8bdfe`) | keep upstream semantics behind adapter | new policy reuses AWS retry configuration without leaking AWS types | APPLIED |
| context map (`0b560da`) | explicit ownership across contexts | domain owns admission; evaluation owns comparisons | APPLIED |
| governance DDD (`1146df6`) | evaluators have no promotion authority | every row and receipt retains `authority: none` | APPLIED |
| memory-store DDD (`b002755`) | append-only auditable outcomes | existing RuVector/RVF path retained; no shadow persistence | APPLIED |
| live guidance | inject dependencies; avoid timing mocks | deterministic `RunControl` and explicit fixtures, no wall-clock assertions | APPLIED |
| live guidance | unreproduced performance claims must be removed | README numbers come only from committed raw benchmark output | APPLIED |
| live guidance | undefined metrics must be explicit | zero-success fairness remains JSON `null` | APPLIED |
| live guidance | validate once at boundaries | tenant weights share the existing scenario validator | APPLIED |
| live guidance | verify hashes at use time | scenario SHA-256 is computed and emitted for every candidate | APPLIED |
| live guidance | evidence chains need external anchoring | current RVF local witness is documented as evaluation-only | DEVIATED: external anchor remains unavailable |

Private lesson text and identifiers are intentionally excluded.
