// SPDX-License-Identifier: MIT OR Apache-2.0
//
// nvms_export.h — C ABI contract for the Go → Rust bridge.
//
// This header is the **stable, manually-curated** declaration of every
// symbol exported by the NVMS Go CGO shim (`nvms_export.go`) and consumed
// by the Rust FFI crate (`bindings/rust-ffi`, crate name `nvms-ffi`).
//
// Two audiences:
//   1. CGO consumers (the `nvms_export.go` shim) — must keep the
//      C-side type definitions here in lock-step with the C block in the
//      Go source.
//   2. Rust consumers (`bindings/rust-ffi/src/lib.rs::sys`) — must keep
//      its `#[repr(C)]` types and `extern "C"` declarations in
//      lock-step with the prototypes below.
//
// When the shim is built with `go build -buildmode=c-archive`, `cgo`
// additionally emits a machine-generated `_cgo_export.h` that mirrors
// these prototypes; that file is treated as an internal build artifact
// and not committed. This committed header is the source of truth.

#ifndef NVMS_EXPORT_H
#define NVMS_EXPORT_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// ---------------------------------------------------------------------------
// Opaque instance handle.
// ---------------------------------------------------------------------------
typedef struct NvmsInstance NvmsInstance;

// ---------------------------------------------------------------------------
// Enumerations (must match `sys::NvmsTier`, `sys::NvmsStatus`, etc. in
// `bindings/rust-ffi/src/lib.rs`).
// ---------------------------------------------------------------------------
typedef enum {
    NVMS_TIER_WASM        = 1,
    NVMS_TIER_GVISOR      = 2,
    NVMS_TIER_FIRECRACKER = 3,
} NvmsTier;

typedef enum {
    NVMS_STATUS_STOPPED  = 0,
    NVMS_STATUS_STARTING = 1,
    NVMS_STATUS_RUNNING  = 2,
    NVMS_STATUS_STOPPING = 3,
    NVMS_STATUS_ERROR    = 4,
} NvmsStatus;

typedef enum {
    NVMS_GPU_NONE        = 0,
    NVMS_GPU_APPLE_METAL = 1,
    NVMS_GPU_NVIDIA_CUDA = 2,
    NVMS_GPU_AMD_ROCM    = 3,
    NVMS_GPU_INTEL_ONEAPI = 4,
} NvmsGpuBackend;

typedef enum {
    NVMS_MEMORY_CPU     = 0,
    NVMS_MEMORY_GPU     = 1,
    NVMS_MEMORY_UNIFIED = 2,
} NvmsMemoryType;

// ---------------------------------------------------------------------------
// Aggregate types (must match `#[repr(C)]` layouts in `sys`).
// ---------------------------------------------------------------------------
struct NvmsInstance {
    uint64_t        id;
    NvmsTier        tier;
    NvmsStatus      status;
    char           *name;
    NvmsGpuBackend  gpu_backend;
    NvmsMemoryType  memory_type;
    uint64_t        gpu_memory_bytes;
};

typedef struct {
    char           name[256];
    NvmsGpuBackend backend;
    uint64_t       memory_bytes;
    uint32_t       compute_units;
    bool           supports_unified_memory;
} NvmsGpuDevice;

typedef struct {
    uint64_t startup_time_ns;
    uint64_t memory_used_bytes;
    double   gpu_utilization;
} NvmsPerfStats;

// ---------------------------------------------------------------------------
// Library / platform.
// ---------------------------------------------------------------------------
const char *nvms_version(void);
const char *nvms_platform_info(void);
int32_t    nvms_init(void);
int32_t    nvms_init_gpu(NvmsGpuBackend backend);
NvmsGpuDevice nvms_gpu_info(void);
bool       nvms_supports_gpu(void);
bool       nvms_supports_unified_memory(void);

// ---------------------------------------------------------------------------
// Apple Silicon (Metal / ANE / Unified Memory).
// ---------------------------------------------------------------------------
int32_t nvms_apple_silicon_init(void);
bool    nvms_apple_ane_available(void);
void   *nvms_apple_unified_memory_alloc(uint64_t size);

// ---------------------------------------------------------------------------
// NVIDIA CUDA.
// ---------------------------------------------------------------------------
int32_t nvms_cuda_init(void);
int32_t nvms_cuda_device_count(void);
void   *nvms_cuda_alloc_unified(uint64_t size);

// ---------------------------------------------------------------------------
// AMD ROCm.
// ---------------------------------------------------------------------------
int32_t nvms_rocm_init(void);
int32_t nvms_rocm_device_count(void);

// ---------------------------------------------------------------------------
// ARM64 NEON.
// ---------------------------------------------------------------------------
bool nvms_neon_available(void);

// ---------------------------------------------------------------------------
// Instance lifecycle.
// ---------------------------------------------------------------------------
NvmsInstance *nvms_instance_create(NvmsTier tier, const char *name);
int32_t       nvms_instance_destroy(NvmsInstance *inst);
int32_t       nvms_instance_start(NvmsInstance *inst);
int32_t       nvms_instance_stop(NvmsInstance *inst);
NvmsStatus    nvms_instance_status(const NvmsInstance *inst);
NvmsPerfStats nvms_perf_stats(void);

#ifdef __cplusplus
} // extern "C"
#endif

#endif // NVMS_EXPORT_H