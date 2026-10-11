# Specification

## Thesis

JitterMap evaluates retries as fleet-level admission. It prevents independently reasonable clients from amplifying a correlated outage while preserving bounded recovery and auditable evidence.

## Input and output

A versioned JSON Scenario contains requests, tenants, optional validated tenant weights, capacity/outage windows, horizon, seed, failure class, and event ceiling. The CLI selects local jitter, AWS standard, adaptive shared-budget, adaptive deferral, or tenant-fair shared-budget policy. Output includes aggregate and per-tenant outcomes, optional Jain success fairness, nearest RuVector outcomes, exact provenance, RVF root, and authority none.

## Acceptance criteria

1. Identical input and seed produce byte-stable domain results.
2. Real AWS RetryConfig is translated in integration tests.
3. Real RuVector insert/search/read-back and RVF verification execute.
4. Invalid, bounded, timeout, persistence, and witness paths are tested.
5. Frozen corpora compare local jitter, AWS standard, adaptive shared-budget, adaptive deferral, and tenant-fair admission against exact scenario digests.
6. No evaluator or memory component can promote a policy.

## 2026-10-10 proportional-service acceptance criterion

For a validated skewed workload, the system shall expose equal-share and demand-weighted tenant admission as separate executable policies. The weighted policy must be deterministic, must not change other policies, and must improve Jain fairness of per-tenant success ratios over equal-share admission without reducing total successes on the frozen fixture. Promotion still requires preserving the adaptive-drop success baseline.
