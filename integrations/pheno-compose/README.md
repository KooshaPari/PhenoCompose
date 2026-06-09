# PhenoCompose NVMS Driver

Integration layer between PhenoCompose and NVMS.

## Architecture

```
PhenoCompose (Rust) → NvmsDriver → nvms_ffi (Rust FFI bindings)
                              ↓
                    ┌─────────┴─────────┐
                    │   3-Tier Isolation │
                    ├─────────┬─────────┤
                    │ WASM    │ ~1ms    │
                    │ gVisor  │ ~90ms   │
                    │ Firecracker│~125ms│
                    └─────────┴─────────┘
```

## Usage

```rust
use pheno_compose_driver::{NvmsDriver, Tier, NvmsConfig};

let driver = NvmsDriver::new()?;

// Create an instance at a specific isolation tier
let mut instance = driver.create_instance(Tier::Wasm, "my-service")?;
instance.start()?;

// Or build with explicit config (CPUs, memory, image, env)
let config = NvmsConfig::for_tier(Tier::Firecracker)
    .with_cpus(2)
    .with_memory_gb(4)
    .with_image("alpine:3.20")
    .with_env("RUST_LOG", "info");
let instance = driver.create_instance_with_config(&config)?;
```

## Status

This crate wraps the upstream `nanovms` Go core via the `nvms_ffi` raw FFI
bindings. The PhenoCompose Go tree (`cmd/nanovms/`, `internal/`) is a
near-verbatim copy of the sibling `nanovms` repo and is in the process of
being consolidated — once that work lands, only this Rust driver plus
`bindings/{rust-ffi,go-c-export,zig,mojo}` will remain in the PhenoCompose
tree, and the upstream `nanovms` module path will be the single source of
truth for the runtime.
