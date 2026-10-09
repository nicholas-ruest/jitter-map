# Specification

## Thesis

JitterMap evaluates retries as fleet-level admission. It prevents independently reasonable clients from amplifying a correlated outage while preserving bounded recovery and auditable evidence.

## Input and output

A versioned JSON Scenario contains requests, tenants, capacity/outage windows, horizon, seed, failure class, and event ceiling. The CLI selects local jitter, AWS standard, adaptive shared-budget (drop-on-exhaustion), or adaptive deferral policy. Output includes successes, failures, attempts, RAF, p95, denials, deferrals, nearest RuVector outcomes, exact provenance, RVF root, and authority none.

## Acceptance criteria

1. Identical input and seed produce byte-stable domain results.
2. Real AWS RetryConfig is translated in integration tests.
3. Real RuVector insert/search/read-back and RVF verification execute.
4. Invalid, bounded, timeout, persistence, and witness paths are tested.
5. One frozen corpus compares local jitter, AWS standard, adaptive shared-budget, and adaptive deferral against the same scenario digest.
6. No evaluator or memory component can promote a policy.
