# Worklog: PhenoCompose hygiene (DAG unit PC-001..002)

**Branch:** `fix/pc-001-dead-features`
**Plan ref:** `plans/2026-06-22-compute-infra-dag-v1.md` → track T-PC.1

## What was done

| ID     | Action                                                                |
|--------|-----------------------------------------------------------------------|
| PC-001 | Delete dead `[features] cuda = []` from `bindings/rust-ffi/Cargo.toml` |
| PC-002 | Add historical note explaining the removal and how to re-introduce    |

## What was NOT done (and why)

- **CONSOLIDATION.md #1** (move `bindings/go-c-export/nvms_core.go` to
  `nanovms/cmd/nanovms-cgo/main.go`): requires coordinated change in
  nanovms repo (new build target, Makefile, CI matrix). Tracked as
  PC-010 for a follow-up PR after the nanovms go.mod cleanup lands.
- **CONSOLIDATION.md #2** (wire `bindings/rust-ffi/build.rs` to link
  against the nanovms CGo archive): requires PC-010 to be in place
  first. Tracked as PC-011.
- **CONSOLIDATION.md #3** (delete `c_int`/`c_ulonglong` warning): the
  claim is incorrect. Both types ARE used in `pub mod sys` (see
  `bindings/rust-ffi/src/lib.rs:48,54,61,62,68,69,76,77,…`). The
  `use std::os::raw::{c_char, c_int, c_ulonglong}` at line 8 lives
  inside `pub mod sys`, not the outer scope. No warning exists.
- **CONSOLIDATION.md #4** (cuda flag) — addressed in this PR.

## Verification

```
$ cargo check -p nvms-ffi --offline
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 44s
```

The full workspace `cargo check` (8 crates, all with `#![forbid(unsafe_code)]`
or `#![deny(missing_docs)]` and bindgen build scripts) takes >5 minutes on
this Windows host due to repeated pheno-tracing / pheno-otel resolution
attempts. Verification on the `nvms-ffi` crate alone (the only crate we
touched) is GREEN. The other 7 crates were not modified in this PR and
their last known state was GREEN per `worklog-L2-029-2026-06-11.json`.

## Follow-ups

- PC-010: migrate `bindings/go-c-export/nvms_core.go` into nanovms
- PC-011: wire `build.rs` to link against the nanovms CGo archive
- PC-012: add a `LICENSE-MIT` and `LICENSE-APACHE` (currently only
  referenced in source headers)
- PC-020: rebuild the port-types crate as `no_std`-compatible
- PC-021: collapse `crates/port-di` and `crates/pheno-config` (they
  overlap significantly)
