# ADR 0002: Decompose port-types lib.rs

## Status

Accepted. 2026-07-08.

## Context

`crates/port-types/src/lib.rs` had grown to 632 lines, exceeding the AGENTS.md hard limit of 500 lines per module.

## Decision

Decompose `port-types` into five cohesive sub-modules:

- `lib.rs` (186) — license + crate-level doc + mod decls + re-exports + tests
- `compose.rs` (163) — Manifest, ComposedArtifact, PublishTarget, PublishReceipt
- `runtime.rs` (168) — ImageRef, ContainerId, ContainerStatus
- `error.rs` (43) — PortError (shared port error type)
- `secret.rs` (116) — SecretRef, Secret

All public types remain re-exported at the crate root, so the public API is unchanged.

## Consequences

- `crates/port-types/src/oci.rs` (427 lines) remains as-is. It doesn't fit a natural image/manifest/runtime split (it's mostly image reference helpers). It is over the 350 target but under the 500 hard limit. Future work.
- File size cap (500 hard, 350 target) is now respected across the entire workspace.
- 34 unit tests + 2 doc tests continue to pass.
