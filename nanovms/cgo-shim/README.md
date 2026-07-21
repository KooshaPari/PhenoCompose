# `nanovms/cgo-shim/` — Go CGO Export Shim (T09)

This directory holds the **C ABI contract** that bridges the Go side of
the NVMS core (`nanovms/cmd/nvms-cgo/`) to the Rust FFI consumer in
`bindings/rust-ffi/` (crate `nvms-ffi`).

## Layout

| File              | Role                                                                                                                                  |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| `nvms_export.go`  | CGO source: `import "C"` block + `//export nvms_*` directives. Compiled with `go build -buildmode=c-archive -o libnvms_core.a .`        |
| `nvms_export.h`   | Hand-written C ABI header declaring every prototype and type the Go shim and Rust FFI agree on. Source of truth for cross-language layout. |

`nvms_export.go` is tagged `//go:build ignore` so it is **not** part of
any Go package — it is compiled only as a c-archive on demand (see
[Building](#building)).

## Why it lives here

Prior to T09 the shim was a vestigial copy in `bindings/go-c-export/`
that the canonical migration (see [`bindings/README.md`](../../bindings/README.md))
had already declared obsolete. The remaining copy kept being referenced
indirectly from `bindings/rust-ffi/build.rs` (which watches for
`nvms_core.h`), so we extracted it to a single, canonical location
under a `nanovms/` crate-style directory.

## C ABI contract

The full ABI is declared in [`nvms_export.h`](./nvms_export.h). Three
rules keep Go, C, and Rust in lock-step:

1. **`#[repr(C)]` / `enum` ordering.** Every `NvmsTier`, `NvmsStatus`,
   `NvmsGpuBackend`, `NvmsMemoryType` enumerator must have the same
   numeric value in C, Go (the `typedef enum` block in the `import "C"`
   preamble of `nvms_export.go`), and Rust (`sys::NvmsTier`, etc. in
   `bindings/rust-ffi/src/lib.rs`).
2. **Struct layout.** `NvmsInstance`, `NvmsGpuDevice`, `NvmsPerfStats`
   are aggregate types whose field order, sizes, and alignment are
   fixed by C. Rust mirrors them as `#[repr(C)]` structs; the Go
   declarations in the C preamble mirror them as `typedef struct`.
3. **Symbol signatures.** Every `//export nvms_*` directive in
   `nvms_export.go` must have a matching prototype in `nvms_export.h`
   and a matching `unsafe extern "C" { pub fn nvms_* }` declaration in
   `bindings/rust-ffi/src/lib.rs::sys`.

A discrepancy in any of the three will manifest as either a Rust link
error (missing symbol), a layout mismatch at runtime, or — worst case —
silent data corruption. Treat this directory as ABI-frozen.

## Symbol groups

| Group                                  | Symbols                                                                                                                                                  |
| -------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Library / platform                     | `nvms_version`, `nvms_platform_info`, `nvms_init`, `nvms_init_gpu`, `nvms_gpu_info`, `nvms_supports_gpu`, `nvms_supports_unified_memory`                |
| Apple Silicon                          | `nvms_apple_silicon_init`, `nvms_apple_ane_available`, `nvms_apple_unified_memory_alloc`                                                                 |
| NVIDIA CUDA                            | `nvms_cuda_init`, `nvms_cuda_device_count`, `nvms_cuda_alloc_unified`                                                                                   |
| AMD ROCm                               | `nvms_rocm_init`, `nvms_rocm_device_count`                                                                                                               |
| ARM64 NEON                             | `nvms_neon_available`                                                                                                                                    |
| Instance lifecycle                     | `nvms_instance_create`, `nvms_instance_destroy`, `nvms_instance_start`, `nvms_instance_stop`, `nvms_instance_status`, `nvms_perf_stats`                  |

## Building

This shim is built by the upstream `nanovms` repository, not by
PhenoCompose's Cargo workspace. To produce `libnvms_core.a`:

```bash
cd /path/to/nanovms
make build-cgo          # produces build/cgo/libnvms_core.a + .h
```

`bindings/rust-ffi/build.rs` then discovers the static library via the
`NANOVMS_BUILD` environment variable or the default
`../../repos/nanovms/build/` sibling path, and links it in
automatically (gated by `nvms_real_ffi`). When the static library is
absent, the crate falls back to a pure-Rust `shim` module so the
workspace still compiles and tests pass offline.

## Adding a new symbol

1. Add the C prototype to `nvms_export.h`.
2. Add the matching `//export nvms_xxx` function to `nvms_export.go`,
   plus any new types in its `import "C"` preamble.
3. Add the matching `unsafe extern "C" { pub fn nvms_xxx }`
   declaration to `bindings/rust-ffi/src/lib.rs::sys`.
4. Bump the shim version comment in `nvms_export.go` if the change is
   not source-compatible.

## Migration history

| Date       | Event                                                                                  |
| ---------- | -------------------------------------------------------------------------------------- |
| 2026-06-14 | PhenoCompose Go tree deleted; shim declared canonical in nanovms repo (see `bindings/README.md`). |
| 2026-07-02 | **T09** — extracted the surviving local copy from `bindings/go-c-export/` to `nanovms/cgo-shim/`, added committed `nvms_export.h`, refreshed `build.rs` reference. |