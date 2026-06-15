# PhenoCompose ↔ nanovms Consolidation

## What was removed (2026-06-08)

The PhenoCompose Go tree was a 91% verbatim fork of
[`KooshaPari/nanovms`](https://github.com/KooshaPari/nanovms) with ~256 LOC of
features dropped (journalctl log streaming, nsenter exec, real landlock
detection, metrics) and 100% of the test coverage dropped (500 LOC of
`*_test.go`). The module was also mis-named: the Go module declared
`github.com/kooshapari/phenocompose` but every Go file imported
`github.com/kooshapari/nanovms/internal/...`, so the tree could not build
standalone.

Removed in this commit:

| Path                               |      LOC | Reason                                                 |
| ---------------------------------- | -------: | ------------------------------------------------------ |
| `cmd/nanovms/main.go`              |      109 | Duplicate of `nanovms/cmd/nanovms/main.go`             |
| `internal/adapters/linux/`         |      281 | Byte-identical to nanovms (whitespace-only diff)       |
| `internal/adapters/mac/`           |      401 | **md5 byte-identical** to nanovms                      |
| `internal/adapters/sandbox/`       |      979 | 256 LOC of features dropped (see `sandbox.go:1-30`)    |
| `internal/adapters/wasm/`          |      212 | **md5 byte-identical** to nanovms                      |
| `internal/adapters/windows/`       |      325 | **md5 byte-identical** to nanovms                      |
| `internal/domain/`                 |      363 | Whitespace-only diff vs nanovms                        |
| `internal/ports/`                  |      176 | **md5 byte-identical** to nanovms                      |
| `go.mod`, `go.sum`                 |        3 | Module declared but no local imports                   |
| `tests/bdd/tier-isolation.feature` |      182 | Byte-identical to `nanovms/tests/bdd/...`              |
| `tests/playwright/`                |      371 | `index.ts` byte-identical; `package.json` version-only |
| **Total removed**                  | **3402** |                                                        |

## What stays

| Path                                  | Status | Notes                                                    |
| ------------------------------------- | ------ | -------------------------------------------------------- |
| `bindings/rust-ffi/`                  | Kept   | Manual `extern "C"` declarations matching the C ABI      |
| `bindings/go-c-export/nvms_core.go`   | Kept   | C-export shim that should move to nanovms in a follow-up |
| `bindings/mojo/`, `bindings/zig/`     | Kept   | Unrelated language bindings                              |
| `pheno-compose-driver/`               | Kept   | High-level Rust wrapper around `nvms-ffi`                |
| `docs/`, `integrations/`, `worklogs/` | Kept   | PhenoCompose-specific content                            |

## Recommended follow-up

1. **Move `bindings/go-c-export/nvms_core.go` into nanovms** as
   `cmd/nanovms-cgo/main.go` and add a CGo build target in nanovms's
   `Makefile`/`Taskfile.yml` (e.g. `make nvms-c-archive` →
   `libnvms_core_$(GOOS)_$(GOARCH).a`).
2. **Wire `bindings/rust-ffi/build.rs`** to call
   `cargo:rustc-link-lib=static=nvms_core` and
   `cargo:rustc-link-search=native=../nanovms/build` so the `staticlib`
   and `cdylib` crate-types actually resolve their C symbols.
3. **Delete `cargo check` warning** in
   `bindings/rust-ffi/src/lib.rs:9` (`c_int` and `c_ulonglong` are
   imported in the outer `use` but only used inside `pub mod sys`).
4. **Decide on the dead `[features] cuda` flag** in
   `bindings/rust-ffi/Cargo.toml:27` (no gate anywhere in code).

After (1)+(2) PhenoCompose becomes a 1,178-LOC pure-Rust crate
(FFI bindings + high-level driver) that links against a single
artifact built from nanovms — no Go, no C, no duplication.
