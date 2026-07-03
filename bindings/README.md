# NVMS Bindings (Migrated)

> **All bindings have been consolidated as of 2026-06-14.**

> This directory previously hosted Go, Rust, Mojo, and Zig bindings.
> They now live in canonical locations to avoid duplication.

## New Canonical Homes

| Language      | Old Location                       | New Canonical Home                                                                                                               |
| ------------- | ---------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| Go (C-export) | `bindings/go-c-export/`            | [`nanovms/cmd/nvms-cgo/main.go`](https://github.com/KooshaPari/nanovms/blob/main/cmd/nvms-cgo/main.go) (upstream) — local copy at `nanovms/cgo-shim/nvms_export.{go,h}` for the C ABI contract (T09) |
| Rust (FFI)    | `bindings/rust-ffi/`               | [`thegent/crates/thegent-nvms/src/lib.rs`](https://github.com/KooshaPari/thegent/blob/main/crates/thegent-nvms/src/lib.rs)       |
| Rust (Driver) | `pheno-compose-driver/`            | [`nanovms/sdk/rust/src/driver.rs`](https://github.com/KooshaPari/nanovms/blob/main/sdk/rust/src/driver.rs)                       |
| Python (pyo3) | `bindings/build_cross_platform.py` | [`thegent/crates/thegent-nvms/`](https://github.com/KooshaPari/thegent/blob/main/crates/thegent-nvms/) (enable `python` feature) |
| Mojo          | `bindings/mojo/`                   | [`thegent/src/thegent/infra/mojo_bridge.py`](https://github.com/KooshaPari/thegent/blob/main/src/thegent/infra/mojo_bridge.py)   |
| Zig           | `bindings/zig/`                    | [`thegent/crates/thegent-wasm-tools/`](https://github.com/KooshaPari/thegent/blob/main/crates/thegent-wasm-tools/) (Wasm SDK)    |

## Rationale

See [`PhenoCompose/CONSOLIDATION.md`](../CONSOLIDATION.md) for the full migration rationale.

Key points:

- `thegent` already had a formal 4-tier sandbox ADR (bubblewrap/gVisor/Firecracker/Wasm) and 14+ pyo3-enabled Rust crates
- `nanovms` already had a canonical Rust SDK (`sdk/rust/`)
- PhenoCompose's Go tree was 91% byte-identical to `nanovms` and was deleted on 2026-06-08
- The remaining bindings were a strict subset of `thegent`'s polyglot infrastructure

## Building (New Locations)

### Go C-export

```bash
cd /path/to/nanovms
make build-cgo          # Produces build/cgo/libnvms_core.a + .h
```

### Rust FFI + Python

```bash
cd /path/to/thegent/crates/thegent-nvms
cargo build --features python   # Produces cdylib + rlib
maturin develop                  # Or: pip installable Python extension
```

### Rust Driver

```bash
cd /path/to/nanovms/sdk/rust
cargo build --features driver    # High-level async driver
```

## Migration History

| Date       | Event                                                             |
| ---------- | ----------------------------------------------------------------- |
| 2026-06-08 | PhenoCompose Go tree deleted (91% duplicate of nanovms)           |
| 2026-06-14 | Go C-export moved to `nanovms/cmd/nvms-cgo/`                      |
| 2026-06-14 | Rust FFI moved to `thegent/crates/thegent-nvms/`                  |
| 2026-06-14 | Rust driver merged into `nanovms/sdk/rust/src/driver.rs`          |
| 2026-06-14 | Mojo/Zig bindings redirected to thegent's existing infrastructure |
| 2026-07-02 | T09: local CGO shim extracted to `nanovms/cgo-shim/` with committed `nvms_export.h` ABI header |
