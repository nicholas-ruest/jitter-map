# CORE_MEMORY_READ receipt

Default branch commit: `d9c4b5750e011aa807fbae8ddad3e78f8baf7249`.

| File | Blob SHA | Applied decision |
|---|---|---|
| AGENTS.md | 4afe6f9008076140486d2a1e5424ddc6e40a0671 | coordination is a ledger, executor performs work |
| CLAUDE.md | 0178167276c981ff0ba503e2e564eee8d347e3f7 | follow recall-to-publish loop and exact evidence |
| docs/architecture-overview.md | 8ba7f76674e59555dbcd5f971554884c54085944 | typed boundaries and stateless service |
| docs/adr/0002-storage-engine-anti-corruption-adapter.md | 9a8bdfe08dd41abc4402ab46a4ad359356815b6e | typed anti-corruption boundaries |
| docs/ddd/bounded-context-map.md | 0b560da87a55b01d5559eeb57b4015feaa77bc30 | explicit ownership and context mapping |
| docs/ddd/governance.md | 1146df6918ae704bc06f4b5c9cf8a37139eedb9a | evaluators advisory, authority none |
| docs/ddd/memory-store.md | b00275577f9ee60b192b4aa2712632f2e2439ef7 | append-only auditable memory |

Applied public-safe guidance: typed anti-corruption boundaries; deterministic tests without timing mocks; evaluators advisory with authority none; source-bound evidence; append-only witnesses; no self-promotion.
