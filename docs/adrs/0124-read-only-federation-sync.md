# ADR-0124: Read-only federation sync boundary

Status: Accepted

## Context

ADR-0123 proved that Rust could execute the pinned Ruflo registry command, but a
registry document is configuration provenance rather than a live coordination read.
JitterMap still lacked source-bound evidence that its own Rust adapter could reach the
federation gateway and consume the gateway's provenance envelope. Federation messages
are third-party data and can contain arbitrary text; treating any payload as an
instruction would cross the project's authority boundary.

Ruflo 3.56.10 exposes `federation sync` through the signature-verifying gateway. The
gateway returns an envelope with `untrusted`, `provenance`, a TLS relay URL,
`retrievedAt`, and `data.count/messages`. The service bounds `sinceSeconds` to
1..=2,592,000 and `limit` to 1..=1,000.

## Alternatives considered

1. Keep registry discovery as the only live Ruflo evidence. This remains safe but does
   not demonstrate a live coordination read.
2. Join the relay and publish a synthetic interruption. This creates an external
   identity and authority that the build has not been granted.
3. Read the relay directly. This duplicates Ruflo's NIP-42 and signature-verification
   responsibilities and weakens provenance.
4. Invoke pinned `federation sync` read-only, preserve the complete untrusted envelope
   for validation, and expose only bounded metadata in the receipt.

## Decision

Choose option 4. `RufloProcessTransport::sync_federation` executes fixed argv without a
shell. Typed numeric inputs are validated before spawn. The child inherits the existing
30-second timeout and separate 64-KiB stdout/stderr caps.

The adapter requires `untrusted=true`, a non-empty provenance label and retrieval time,
a `wss://` relay, and equality between `data.count` and `messages.len()`. Message bodies
remain opaque `serde_json::Value` data. The receipt records only count, a sorted
frequency map of bounded message `type` labels, input bounds, argv/output SHA-256,
elapsed time, exit status, package pin, and `authority=none`. Older signed gateway
wrappers that expose only a non-empty `raw` field are classified as `opaque-raw`; the
adapter neither parses nor stores that field. Messages with neither a non-empty type nor
a valid raw wrapper fail closed. The receipt never records message payloads, author keys,
gateway credentials, or environment values.

## Tradeoffs

The adapter can prove reachability and provenance-preserving ingestion without joining
the relay, but it cannot prove remote interruption delivery or membership. A zero-count
response is valid. The live test therefore validates the typed envelope without making
availability depend on current federation traffic. Deterministic parser tests cover
non-empty messages and hostile schema cases.

## Consequences

Malformed JSON, `untrusted=false`, non-TLS relay metadata, missing provenance fields,
count mismatch, invalid message-type metadata, timeout, output overflow, non-zero exit,
or receipt I/O failure returns `PortError` and produces no successful receipt. Domain
results remain unchanged when the process integration is removed; the sync path is
read-only evidence with no promotion, deployment, membership, or interruption authority.

## Validation

- deterministic parser tests for valid non-empty and zero-message envelopes;
- a signed-raw-wrapper classification test plus rejection of unclassified messages;
- rejection tests for trusted-labelled, non-TLS, inconsistent-count, missing-field, and
  invalid-type responses;
- input-bound tests for both gateway-enforced ranges;
- existing child-process timeout, overflow, and non-zero-exit tests;
- an ignored RuOS test against pinned Ruflo 3.56.10 that performs fresh registry and
  seven-day federation-sync reads from an isolated checkout;
- locked workspace tests, strict Clippy, dependency-direction, and structural checker.
