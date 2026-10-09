# Project status

- IMPLEMENTED: six-crate Rust vertical slice with AWS RetryConfig, RuVector and RVF paths
- IMPLEMENTED: added `AdaptiveDeferral` (ADR-0026) as a second, distinct admission strategy alongside the unchanged `AdaptiveBudget` ablation
- VALIDATED: governed executor gates pass; frozen benchmark does not clear promotion criteria for either adaptive candidate
- REPO_PUBLISHED: architecture and integration recovery PRs merged; validation PR records final evidence
- RESEARCH_GIST: BLOCKED_GIST_PUBLISH
- BUILD_GIST: prior recovery post exists, but the required build announcement remains BLOCKED_GIST_PUBLISH
- BUILT: no

The project is source-grounded and executable but both adaptive candidates require revision: on the frozen fixture, deferral trades immediate denial for more attempts that mostly still fail, yielding a lower success count and higher RAF than the drop ablation. It cannot be called built until both dated public Gist files are published and independently read back.
