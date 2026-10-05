# Specification

## Thesis

JitterMap evaluates retries as fleet-level admission. It prevents independently reasonable clients from amplifying a correlated outage while preserving bounded recovery and auditable evidence.

## Input and output

A versioned JSON Scenario contains requests, tenants, capacity/outage windows, horizon, seed, failure class, and event ceiling. The CLI selects local jitter, AWS standard, or adaptive shared-budget policy. Output includes successes, failures, attempts, RAF, p95, denials, nearest RuVector outcomes, exact provenance, RVF root, and authority none.

## Acceptance criteria

1. Identical input and seed produce byte-stable domain results.
2. Real AWS RetryConfig is translated in integration tests.
3. Real RuVector insert/search/read-back and RVF verification execute.
4. Invalid, bounded, timeout, persistence, and witness paths are tested.
5. One frozen corpus compares local, AWS, adaptive, Ruvnet-only, and external-only slices.
6. No evaluator or memory component can promote a policy.
